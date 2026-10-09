#!/usr/bin/env sh
# Build the browser plugin through the xcelerate CLI: it writes wit/plugin.wit
# (the canonical host ABI) and compiles the component next to plugin.json.
set -eu

xcelerate build --wasm-only
