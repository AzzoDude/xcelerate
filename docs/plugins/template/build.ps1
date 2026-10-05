#!/usr/bin/env pwsh
# Build the mod as a WebAssembly component and stage it next to plugin.json.
$ErrorActionPreference = "Stop"

$artifact = "{{crate}}" -replace '-', '_'

rustup target add wasm32-wasip2 2>$null
cargo build --release --target wasm32-wasip2
Copy-Item "target/wasm32-wasip2/release/$artifact.wasm" "{{entrypoint}}" -Force

Write-Host "built {{entrypoint}}"
