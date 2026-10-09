#!/usr/bin/env pwsh
# Build the native-app plugin through the xcelerate CLI: it writes wit/plugin.wit
# (the canonical host ABI) and compiles the component next to plugin.json.
$ErrorActionPreference = "Stop"

xcelerate build --wasm-only
