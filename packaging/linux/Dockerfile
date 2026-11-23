FROM rust:1.95-slim-bookworm AS toolchain
FROM ubuntu:22.04
ENV DEBIAN_FRONTEND=noninteractive
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential ca-certificates pkg-config python3 file \
    libasound2-dev libasound2-plugins libxkbcommon-dev libxkbcommon-x11-0 libwayland-dev libx11-dev libxcursor-dev \
    libxi-dev libxrandr-dev libxcb1-dev libvulkan1 libgl1-mesa-dri mesa-vulkan-drivers \
    flatpak appstream desktop-file-utils squashfs-tools patchelf \
    xvfb xauth x11-utils dbus-x11 && rm -rf /var/lib/apt/lists/*
COPY --from=toolchain /usr/local/cargo /usr/local/cargo
COPY --from=toolchain /usr/local/rustup /usr/local/rustup
ENV RUSTUP_HOME=/usr/local/rustup CARGO_HOME=/usr/local/cargo
ENV PATH=/usr/local/cargo/bin:$PATH
WORKDIR /work
