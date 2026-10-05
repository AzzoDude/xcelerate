# Distributing the `xcelerate` CLI on Linux

Winget does not cover Linux. Ship the same CLI (`crates/xcelerate-cli`, binary
`xcelerate`) through the channels below. At runtime it needs Chrome or Edge on
the machine, or an explicit `--executable-path`, exactly like the Windows build.

## Build

```bash
cargo build --release -p xcelerate-cli
./target/release/xcelerate --version
```

## Tarball (works on every distro)

[`build-tarball.sh`](build-tarball.sh) builds and packages the current version:

```bash
./packaging/linux/build-tarball.sh                                   # host target
./packaging/linux/build-tarball.sh x86_64-unknown-linux-gnu          # explicit
./packaging/linux/build-tarball.sh x86_64-unknown-linux-musl         # static glibc-free
```

It writes `dist/xcelerate-<version>-<target>.tar.gz` plus a `.sha256`.
(`rustup target add <target>` first for the musl/arm64 variants.)

## Install from a tarball

```bash
name=xcelerate-1.0.10-x86_64-unknown-linux-gnu
tar -xzf "$name.tar.gz"
install -Dm755 "$name/xcelerate" ~/.local/bin/xcelerate    # or /usr/local/bin
# ~/.local/bin must be on PATH
xcelerate --version
```

## Channels

### GitHub Releases

Attach the tarballs to the release. The
[`Release CLI`](../../.github/workflows/release-cli.yml) workflow does this for
`v*` tags; users then `curl`/install from the stable URL:

```
https://github.com/ChaoswareHQ/xcelerate/releases/download/v<version>/xcelerate-<version>-x86_64-unknown-linux-gnu.tar.gz
```

### Homebrew / Linuxbrew tap

Create a tap repo `ChaoswareHQ/homebrew-tap` with `Formula/xcelerate.rb`:

```ruby
class Xcelerate < Formula
  desc "Chrome DevTools Protocol client and CLI"
  homepage "https://github.com/ChaoswareHQ/xcelerate"
  version "1.0.10"
  license any_of: ["MIT", "Apache-2.0"]

  on_macos do
    on_arm do
      url "https://github.com/ChaoswareHQ/xcelerate/releases/download/v1.0.10/xcelerate-1.0.10-aarch64-apple-darwin.tar.gz"
      sha256 "REPLACE"
    end
    on_intel do
      url "https://github.com/ChaoswareHQ/xcelerate/releases/download/v1.0.10/xcelerate-1.0.10-x86_64-apple-darwin.tar.gz"
      sha256 "REPLACE"
    end
  end

  on_linux do
    on_arm do
      url "https://github.com/ChaoswareHQ/xcelerate/releases/download/v1.0.10/xcelerate-1.0.10-aarch64-unknown-linux-gnu.tar.gz"
      sha256 "REPLACE"
    end
    on_intel do
      url "https://github.com/ChaoswareHQ/xcelerate/releases/download/v1.0.10/xcelerate-1.0.10-x86_64-unknown-linux-gnu.tar.gz"
      sha256 "REPLACE"
    end
  end

  def install
    bin.install "xcelerate"
  end
end
```

Then: `brew install ChaoswareHQ/tap/xcelerate` (works on macOS and Linux).

### AUR (Arch Linux)

A `PKGBUILD` that unpacks the release tarball:

```bash
pkgname=xcelerate
pkgver=1.0.10
pkgrel=1
pkgdesc="Chrome DevTools Protocol client and CLI"
arch=('x86_64' 'aarch64')
url="https://github.com/ChaoswareHQ/xcelerate"
license=('MIT' 'Apache')
source=("$url/releases/download/v$pkgver/xcelerate-$pkgver-x86_64-unknown-linux-gnu.tar.gz")
sha256sums=('REPLACE')

package() {
  install -Dm755 xcelerate-$pkgver-x86_64-unknown-linux-gnu/xcelerate "$pkgdir/usr/bin/xcelerate"
}
```

Push it to `ssh://aur@aur.archlinux.org/xcelerate.git`.

### `.deb` / `.rpm`

```bash
cargo install cargo-deb cargo-generate-rpm
cargo deb -p xcelerate-cli
cargo generate-rpm -p crates/xcelerate-cli
```

Add a `[package.metadata.deb]` section (license, maintainer, `assets`) to
`crates/xcelerate-cli/Cargo.toml` if you want richer package metadata.

### cargo-binstall / cargo install

`cargo install --path crates/xcelerate-cli` builds from source. With
conventionally named release assets, `cargo binstall xcelerate-cli` can fetch a
prebuilt binary instead.

## Static (distro-agnostic) builds

The core is portable (rustls, wasmtime, mimalloc), so `x86_64-unknown-linux-musl`
produces a static binary that runs anywhere:

```bash
rustup target add x86_64-unknown-linux-musl
./packaging/linux/build-tarball.sh x86_64-unknown-linux-musl
```
