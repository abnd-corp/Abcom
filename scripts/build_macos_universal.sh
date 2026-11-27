#!/usr/bin/env bash
# Binaire macOS universel (Apple silicon et Intel) : deux builds release, réunis par lipo.
# Usage : build_macos_universal.sh <binaire de sortie>
set -euo pipefail

output=$1
targets=(aarch64-apple-darwin x86_64-apple-darwin)

rustup target add "${targets[@]}"
for target in "${targets[@]}"; do
    cargo build --release --locked --target "$target"
done
mkdir -p "$(dirname "$output")"
lipo -create -output "$output" \
    "target/aarch64-apple-darwin/release/abcom" \
    "target/x86_64-apple-darwin/release/abcom"
lipo -verify_arch "$output" arm64 x86_64
lipo -info "$output"
