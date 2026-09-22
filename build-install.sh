#! /bin/sh
distrobox enter builder -- just build-release
sudo install -m 755 target/release/flip-companion /usr/local/bin/
sudo restorecon -v /usr/local/bin/flip-companion
