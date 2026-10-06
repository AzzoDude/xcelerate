#!/usr/bin/env sh
# Build the xcelerate binaries (CLI + MCP server) and package them as a Linux tarball.
#
#   ./packaging/linux/build-tarball.sh [target-triple]
#
# With no argument it builds for the host. Examples:
#   ./packaging/linux/build-tarball.sh x86_64-unknown-linux-gnu
#   ./packaging/linux/build-tarball.sh x86_64-unknown-linux-musl
#
# Writes dist/xcelerate-<version>-<target>.tar.gz (and a .sha256 if available).
set -eu

# Repo root is two levels up from this script.
root=$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)
cd "$root"

version=$(grep -m1 '^version = ' Cargo.toml | cut -d '"' -f2)
target=${1:-$(rustc -vV | sed -n 's/^host: //p')}

echo "building xcelerate-cli + xcelerate-mcp $version for $target"
cargo build --release -p xcelerate-cli -p xcelerate-mcp --target "$target"

name="xcelerate-${version}-${target}"
stage="dist/${name}"

rm -rf "$stage"
mkdir -p "$stage"
cp "target/${target}/release/xcelerate" "$stage/"
cp "target/${target}/release/xcelerate-mcp" "$stage/"
cp LICENSE-MIT LICENSE-APACHE "$stage/" 2>/dev/null || true

tar -C dist -czf "dist/${name}.tar.gz" "$name"

if command -v sha256sum >/dev/null 2>&1; then
  ( cd dist && sha256sum "${name}.tar.gz" > "${name}.tar.gz.sha256" )
elif command -v shasum >/dev/null 2>&1; then
  ( cd dist && shasum -a 256 "${name}.tar.gz" > "${name}.tar.gz.sha256" )
fi

echo "wrote dist/${name}.tar.gz"
