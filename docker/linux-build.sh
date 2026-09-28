#!/usr/bin/env bash
# Runs inside the container: lint, test, and bundle Peron for Linux.
# Mode: "test" (lint + tests only) or "build" (default: lint + tests + bundles + static peron-cli).
set -euo pipefail
mode="${1:-build}"
cd /src

# node_modules is a container volume (Windows node_modules has Windows-native binaries).
npm ci --no-audit --no-fund

cd src-tauri
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace -- --include-ignored
cd ..

if [ "$mode" = "build" ]; then
  # Static CLI first: the .deb/.rpm ship it as /usr/bin/peron-cli (tauri.linux.conf.json).
  cargo build --manifest-path src-tauri/Cargo.toml --release -p peron-cli --target x86_64-unknown-linux-musl
  mkdir -p src-tauri/bin
  cp /target/x86_64-unknown-linux-musl/release/peron-cli src-tauri/bin/peron-cli

  # The target volume persists between runs; drop old bundles so only this version is copied.
  rm -rf /target/release/bundle
  npm run tauri build
  rm -rf dist-linux && mkdir -p dist-linux
  cp /target/release/bundle/deb/*.deb /target/release/bundle/rpm/*.rpm \
     /target/release/bundle/appimage/*.AppImage dist-linux/
  cp src-tauri/bin/peron-cli dist-linux/peron-cli-x86_64-linux
  (cd dist-linux && sha256sum * > SHA256SUMS)
  ls -la dist-linux
fi
