# Linux build/test environment for Peron (Tauri 2).
# Ubuntu 22.04 = glibc 2.35, so the resulting .deb/.rpm/AppImage run on most current distros.
# Usage: see docker/linux-build.ps1.
FROM ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends \
      build-essential curl wget file ca-certificates pkg-config git \
      libwebkit2gtk-4.1-dev libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev \
      rpm python3 xdg-utils musl-tools \
    && curl -fsSL https://deb.nodesource.com/setup_22.x | bash - \
    && apt-get install -y --no-install-recommends nodejs \
    && rm -rf /var/lib/apt/lists/*

RUN curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable \
      --component clippy,rustfmt \
    && /root/.cargo/bin/rustup target add x86_64-unknown-linux-musl
# musl: peron-cli is built fully static so it runs on any x86_64 Linux server (old glibc included).

# APPIMAGE_EXTRACT_AND_RUN: linuxdeploy needs FUSE otherwise (not available in containers).
# NO_STRIP: linuxdeploy's bundled strip can't handle newer ELF sections.
ENV PATH=/root/.cargo/bin:$PATH \
    APPIMAGE_EXTRACT_AND_RUN=1 \
    NO_STRIP=true \
    CARGO_TARGET_DIR=/target

WORKDIR /src
