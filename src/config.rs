use clap::Parser;
use serde::Deserialize;

/// Bottom-screen companion app for AYANEO Flip DS on Bazzite.
#[derive(Parser, Debug)]
#[command(name = "flip-companion", version)]
pub struct Config {
    /// Run with mock backends (no Wayland, D-Bus, or hardware required).
    #[arg(long)]
    pub mock: bool,

    /// Override the bottom-screen output name (e.g. "eDP-2").
    /// If not set, auto-detection is used.
    #[arg(long)]
    pub output: Option<String>,

    /// Path to Gamescope's DRM lease socket (enables Game Mode).
    /// Connects and receives a DRM lease fd via SCM_RIGHTS.
    /// Default: /tmp/gamescope-lease.sock
    #[arg(long)]
    pub lease_socket: Option<String>,

    /// Path to apps.toml configuration file.
    /// Default: looks for apps.toml next to the executable, then ~/.config/flip-companion/apps.toml
    #[arg(long)]
    pub apps_config: Option<String>,

    /// UI font scale multiplier for small text (labels, keys, tab buttons).
    /// Default: 1.0. Use 2.0 to double the size.
    #[arg(long, default_value_t = 1.0)]
    pub ui_scale: f32,

    /// Path to theme.toml configuration file.
    /// Default: looks for theme.toml next to the executable, then ~/.config/flip-companion/theme.toml
    #[arg(long)]
    pub theme_config: Option<String>,
}

/// A single launchable app entry from apps.toml.
#[derive(Debug, Clone, Deserialize)]
pub struct AppEntry {
    pub name: String,
    pub icon: String,
    pub exec: String,
    pub url: Option<String>,
}

/// Top-level apps.toml structure.
#[derive(Debug, Deserialize)]
struct AppsFile {
    app: Vec<AppEntry>,
}

/// Theme color configuration loaded from theme.toml.
#[derive(serde::Deserialize, Debug, Clone)]
pub struct ThemeColors {
    #[serde(default = "defaults::background")]
    pub background: String,
    #[serde(default = "defaults::surface")]
    pub surface: String,
    #[serde(default = "defaults::interactive")]
    pub interactive: String,
    #[serde(default = "defaults::interactive_pressed")]
    pub interactive_pressed: String,
    #[serde(default = "defaults::text_primary")]
    pub text_primary: String,
    #[serde(default = "defaults::text_secondary")]
    pub text_secondary: String,
    #[serde(default = "defaults::accent")]
    pub accent: String,
    #[serde(default = "defaults::border")]
    pub border: String,
    #[serde(default = "defaults::running_app")]
    pub running_app: String,
    #[serde(default = "defaults::running_app_text")]
    pub running_app_text: String,
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self {
            background:          defaults::background(),
            surface:             defaults::surface(),
            interactive:         defaults::interactive(),
            interactive_pressed: defaults::interactive_pressed(),
            text_primary:        defaults::text_primary(),
            text_secondary:      defaults::text_secondary(),
            accent:              defaults::accent(),
            border:              defaults::border(),
            running_app:         defaults::running_app(),
            running_app_text:    defaults::running_app_text(),
        }
    }
}

mod defaults {
    pub fn background()          -> String { "#1a1a2e".into() }
    pub fn surface()             -> String { "#16213e".into() }
    pub fn interactive()         -> String { "#0f3460".into() }
    pub fn interactive_pressed() -> String { "#1a5276".into() }
    pub fn text_primary()        -> String { "#e0e0f0".into() }
    pub fn text_secondary()      -> String { "#a0a0b8".into() }
    pub fn accent()              -> String { "#e94560".into() }
    pub fn border()              -> String { "#2a2a4a".into() }
    pub fn running_app()         -> String { "#1a3a1a".into() }
    pub fn running_app_text()    -> String { "#90ee90".into() }
}

#[derive(serde::Deserialize, Default)]
struct ThemeFile {
    colors: ThemeColors,
}

/// Load theme colors from theme.toml.
/// Searches in order: --theme-config path, ./theme.toml, ~/.config/flip-companion/theme.toml
/// Falls back to built-in defaults if no file is found.
pub fn load_theme(config: &Config) -> ThemeColors {
    let candidates: Vec<std::path::PathBuf> = if let Some(ref path) = config.theme_config {
        vec![std::path::PathBuf::from(path)]
    } else {
        let mut paths = vec![std::path::PathBuf::from("theme.toml")];
        if let Some(config_dir) = dirs_fallback() {
            paths.push(config_dir.join("theme.toml"));
        }
        paths
    };

    for path in &candidates {
        if let Ok(contents) = std::fs::read_to_string(path) {
            match toml::from_str::<ThemeFile>(&contents) {
                Ok(file) => {
                    eprintln!("[config] loaded theme from {}", path.display());
                    return file.colors;
                }
                Err(e) => {
                    eprintln!("[config] failed to parse {}: {e}", path.display());
                }
            }
        }
    }

    eprintln!("[config] no theme.toml found, using defaults");
    ThemeColors::default()
}

/// Load the apps list from the config file.
/// Searches in order: --apps-config path, ./apps.toml, ~/.config/flip-companion/apps.toml
pub fn load_apps(config: &Config) -> Vec<AppEntry> {
    let candidates: Vec<std::path::PathBuf> = if let Some(ref path) = config.apps_config {
        vec![std::path::PathBuf::from(path)]
    } else {
        let mut paths = vec![std::path::PathBuf::from("apps.toml")];
        if let Some(config_dir) = dirs_fallback() {
            paths.push(config_dir.join("apps.toml"));
        }
        paths
    };

    for path in &candidates {
        if let Ok(contents) = std::fs::read_to_string(path) {
            match toml::from_str::<AppsFile>(&contents) {
                Ok(file) => {
                    eprintln!("[config] loaded {} apps from {}", file.app.len(), path.display());
                    return file.app;
                }
                Err(e) => {
                    eprintln!("[config] failed to parse {}: {e}", path.display());
                }
            }
        }
    }

    eprintln!("[config] no apps.toml found, using empty app list");
    Vec::new()
}

fn dirs_fallback() -> Option<std::path::PathBuf> {
    std::env::var_os("XDG_CONFIG_HOME")
        .map(std::path::PathBuf::from)
        .or_else(|| {
            std::env::var_os("HOME").map(|h| std::path::PathBuf::from(h).join(".config"))
        })
        .map(|p| p.join("flip-companion"))
}
