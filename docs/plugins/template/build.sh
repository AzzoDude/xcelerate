#!/usr/bin/env sh
# Build the mod as a WebAssembly component and stage it next to plugin.json.
set -eu

ARTIFACT=$(printf '%s' "{{crate}}" | tr '-' '_')

rustup target add wasm32-wasip2 >/dev/null 2>&1 || true
cargo build --release --target wasm32-wasip2
cp "target/wasm32-wasip2/release/${ARTIFACT}.wasm" "{{entrypoint}}"

echo "built {{entrypoint}}"
