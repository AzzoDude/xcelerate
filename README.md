**[English](README.md)** | [简体中文](README.zh-CN.md) | [日本語](README.ja-JP.md) | [Tiếng Việt](README.vi-VN.md)

# Xcelerate

[![CI](https://github.com/ChaoswareHQ/xcelerate/actions/workflows/ci.yml/badge.svg)](https://github.com/ChaoswareHQ/xcelerate/actions/workflows/ci.yml)
[![CodeQL](https://github.com/ChaoswareHQ/xcelerate/actions/workflows/codeql.yml/badge.svg)](https://github.com/ChaoswareHQ/xcelerate/actions/workflows/codeql.yml)
[![Semgrep](https://github.com/ChaoswareHQ/xcelerate/actions/workflows/semgrep.yml/badge.svg)](https://github.com/ChaoswareHQ/xcelerate/actions/workflows/semgrep.yml)
[![OpenSSF Scorecard](https://api.securityscorecards.dev/projects/github.com/ChaoswareHQ/xcelerate/badge)](https://securityscorecards.dev/viewer/?uri=github.com/ChaoswareHQ/xcelerate)

[![Crates.io](https://img.shields.io/crates/v/xcelerate.svg)](https://crates.io/crates/xcelerate)
[![Crates.io downloads](https://img.shields.io/crates/d/xcelerate.svg)](https://crates.io/crates/xcelerate)
[![PyPI](https://img.shields.io/pypi/v/xcelerate.svg)](https://pypi.org/project/xcelerate/)
[![npm](https://img.shields.io/npm/v/xcelerate.svg)](https://www.npmjs.com/package/xcelerate)
[![pub.dev](https://img.shields.io/pub/v/xcelerate.svg)](https://pub.dev/packages/xcelerate)
[![NuGet](https://img.shields.io/nuget/v/Xcelerate.svg)](https://www.nuget.org/packages/Xcelerate)
[![Maven Central](https://img.shields.io/maven-central/v/io.github.azzodude/xcelerate.svg)](https://central.sonatype.com/artifact/io.github.azzodude/xcelerate)
[![PowerShell Gallery](https://img.shields.io/powershellgallery/v/Xcelerate.svg)](https://www.powershellgallery.com/packages/Xcelerate)

[![docs.rs](https://img.shields.io/docsrs/xcelerate.svg)](https://docs.rs/xcelerate)
[![Rust](https://img.shields.io/badge/rust-1.99%2B-dea584.svg)](https://github.com/ChaoswareHQ/xcelerate/blob/master/Cargo.toml)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](#license)

[![Chromium](https://img.shields.io/badge/Chromium-supported-4285F4?logo=googlechrome&logoColor=white)](#supported-engines)
[![Firefox](https://img.shields.io/badge/Firefox-supported-FF7139?logo=firefoxbrowser&logoColor=white)](#supported-engines)

Xcelerate is a high-performance, lightweight Chrome DevTools Protocol (CDP) client
with idiomatic bindings for Rust, .NET, Python, JavaScript (Node.js), Kotlin,
Java, Swift, Ruby, Dart/Flutter, and Go. It pairs a fast Rust core with an
async-first API and a data-driven adapter layer that lets existing Selenium,
Playwright, and Puppeteer scripts run against the same engine.

## Supported engines

One core, two engines: Chromium over CDP, Firefox over WebDriver BiDi.

| Engine | Protocol | Backend |
| --- | --- | --- |
| ![Chromium](https://img.shields.io/badge/Chromium-4285F4?logo=googlechrome&logoColor=white) Chromium, Chrome, Edge | Chrome DevTools Protocol (CDP) | `xcelerate::Browser` |
| ![Firefox](https://img.shields.io/badge/Firefox-FF7139?logo=firefoxbrowser&logoColor=white) Firefox | [WebDriver BiDi](https://w3c.github.io/webdriver-bidi/) | `xcelerate::firefox` |

```rust
// Chromium (CDP)
let browser = xcelerate::Browser::launch(Default::default()).await?;

// Firefox (WebDriver BiDi)
let browser = xcelerate::firefox::FirefoxBrowser::launch(Default::default()).await?;
let page = browser.clone().new_page("https://example.com".to_string()).await?;
println!("{}", page.title().await?);
```

## Bindings

| Language | Package | Registry |
| --- | --- | --- |
| Rust | `xcelerate` | [crates.io](https://crates.io/crates/xcelerate) |
| Python | `xcelerate` | [PyPI](https://pypi.org/project/xcelerate/) |
| JavaScript | `xcelerate` | [npm](https://www.npmjs.com/package/xcelerate) |
| .NET | `Xcelerate` | [NuGet](https://www.nuget.org/packages/Xcelerate) |
| Kotlin | `io.github.azzodude:xcelerate` | [Maven Central](https://central.sonatype.com/artifact/io.github.azzodude/xcelerate) |
| Java | `io.github.azzodude:xcelerate-java` | [Maven Central](https://central.sonatype.com/artifact/io.github.azzodude/xcelerate-java) |
| Swift | `Xcelerate` | built from source ([readme](bindings/swift/README.md)) |
| Ruby | `xcelerate` | built from source ([readme](bindings/ruby/README.md)) |
| Dart / Flutter | `xcelerate` | [pub.dev](https://pub.dev/packages/xcelerate) |
| Go | `xcelerate` | built from source ([readme](bindings/go/README.md)) |
| PowerShell | `Xcelerate` | [PowerShell Gallery](https://www.powershellgallery.com/packages/Xcelerate) |

## Features

- **Automated process management** - discovers and launches Chrome or Edge, and
  manages the lifecycle of the browser process.
- **Security-first plugins** - a default-deny plugin system with an append-only
  audit log. Built-in plugins you opt into: `stealth` and `human`.
- **Stealth and human plugins** - `stealth` applies binary patching and a runtime
  JavaScript payload that reduce automation fingerprints; `human` makes input
  behave like a person (Bezier mouse travel, paced typing, uneven scrolling).
  Both are enabled per browser through `BrowserConfig.plugins`.
- **Async-first** - built on `tokio` in Rust and `async`/`await` in every binding.
- **API-style adapters** - expose Selenium, Playwright, and Puppeteer method names on
  top of the native engine, generated from declarative profiles.
- **Multi-language bindings** - one core, generated bindings for Rust, Python,
  JavaScript (Node.js), .NET, Kotlin, Java, Swift, Ruby, Dart/Flutter, and Go via
  `uniffi`, plus a PowerShell module over the .NET SDK.
- **Session video recording** - capture the page to a video through the CDP
  screencast; writes Motion-JPEG AVI with no external tools, or H.264/VP9
  MP4/WebM when `ffmpeg` is on `PATH`.
- **Proxy pool** - route the browser through one or more upstream HTTP proxies
  (with credentials) via a built-in local gateway.
- **Persistent profiles** - keep logins, cookies, and site storage between runs
  with a durable `user-data-dir`.
- **Accessibility snapshots** - a semantic `role`/`name` view of the page for
  robust selectors and agent-driven automation.
- **Agent snapshots** - an indexed, LLM-friendly rendering of the page in which
  every interactive element can be acted on by its `[index]`.
- **CLI and MCP server** - the `xcelerate` command for one-shot actions, and
  `xcelerate mcp` (or the `xcelerate-mcp` binary) to drive the browser from an
  MCP client.

## Installation

### Rust

```toml
[dependencies]
xcelerate = "1.0.12"
tokio = { version = "1", features = ["full"] }
```

### Python

```bash
pip install xcelerate
```

### JavaScript (Node.js)

```bash
npm install xcelerate
```

### .NET / C#

```powershell
dotnet add package Xcelerate
```

### Kotlin / Java

Published to Maven Central as `io.github.azzodude:xcelerate` (Kotlin) and
`io.github.azzodude:xcelerate-java` (Java):

```kotlin
// build.gradle.kts
dependencies {
    implementation("io.github.azzodude:xcelerate:1.0.12")        // Kotlin
    implementation("io.github.azzodude:xcelerate-java:1.0.12")   // Java
}
```

The Java bindings require **JDK 22+** and run with `--enable-native-access=ALL-UNNAMED`.
See [`bindings/kotlin/README.md`](bindings/kotlin/README.md) and
[`bindings/java/README.md`](bindings/java/README.md) for usage.

Building them from source:

```bash
python scripts/install_toolchains.py        # JDK 22+ via winget, Gradle, uniffi-bindgen-java
python scripts/generate_bindings/kotlin.py  # Kotlin sources + Gradle build
python scripts/generate_bindings/java.py    # Java sources + Gradle build
```

### Swift / Ruby / Dart / Go

These targets are built from source with the universal generator. Swift and Ruby
are built-in UniFFI targets; Dart needs `uniffi-bindgen-dart`; Go needs the
NordSecurity `uniffi-bindgen-go` generator and a Go toolchain.

```bash
python scripts/generate_bindings/swift.py    # Swift sources + SwiftPM package
python scripts/generate_bindings/ruby.py     # Ruby sources + gemspec
cargo install uniffi-bindgen-dart            # once
python scripts/generate_bindings/dart.py     # Dart sources + pubspec
go install github.com/NordSecurity/uniffi-bindgen-go/v2/uniffi-bindgen-go@latest
python scripts/generate_bindings/go.py       # Go sources + go.mod
```

### PowerShell

```powershell
Install-PSResource Xcelerate      # PSResourceGet (PowerShell 7.4+)
# or, with PowerShellGet:
Install-Module Xcelerate
```

PowerShell has no UniFFI generator, so the module wraps the .NET SDK. To build
and stage the payload from source instead:

```powershell
python scripts/generate_bindings/powershell.py
Import-Module ./bindings/powershell/Xcelerate.psd1
```

See [`bindings/powershell/README.md`](bindings/powershell/README.md) for usage, or
run the end-to-end example at [`examples/powershell/quickstart.ps1`](examples/powershell/quickstart.ps1).

See [`bindings/`](bindings/) for each package's README.

## Quick start (Rust)

```rust
use xcelerate::{Browser, BrowserConfig};

#[tokio::main]
async fn main() -> Result<(), xcelerate::XcelerateError> {
    let browser = Browser::launch(BrowserConfig::default()).await?;
    let page = browser.new_page("https://example.com".to_string()).await?;

    println!("Title: {}", page.title().await?);

    let heading = page.query_selector("h1".to_string()).await?;
    println!("Heading: {}", heading.text().await?);

    browser.close().await?;
    Ok(())
}
```

`BrowserConfig` defaults to headless and detached, **without** stealth. Enable
plugins explicitly, or opt out of the other defaults:

```rust
use xcelerate::BrowserConfig;

let config = BrowserConfig {
    headless: false,
    detached: false,
    executable_path: None,                      // auto-discover Chrome/Edge
    plugins: Some(vec!["stealth".to_string()]), // opt into the stealth plugin
    ..Default::default()
};
```

## Plugins

Xcelerate ships a **security-first plugin system**. A plugin is a named bundle of
launch-time configuration, page hooks, and invokable operations, and it does
nothing unless you enable it (**default-deny**). `stealth` is the built-in
plugin built on this system.

### Enabling plugins

Plugins are listed in `BrowserConfig.plugins` and enabled before the browser
launches, so they can contribute to the launch itself (for example, patching the
binary):

```rust
use xcelerate::{Browser, BrowserConfig};

let config = BrowserConfig {
    plugins: Some(vec!["stealth".to_string()]),
    ..Default::default()
};
let browser = Browser::launch(config).await?;
```

The same list travels through `BrowserConfig` in every language:

```python
config = BrowserConfig(plugins=["stealth"])
```

```javascript
const browser = await Browser.launch({ plugins: ["stealth"] });
```

```csharp
var browser = await Browser.Launch(new BrowserConfig(Plugins: new[] { "stealth" }));
```

```kotlin
val config = BrowserConfig(plugins = listOf("stealth"))
```

```java
var config = new BrowserConfig(false, false, true, null, List.of("stealth"));
```

### Built-in catalog

| Plugin | What it does | Ops |
| --- | --- | --- |
| `stealth` | Patches the browser binary at launch and injects the anti-fingerprint payload into every document. | `info` |
| `human` | Human-like input: Bezier mouse travel with jitter, clicks that pause and hold, per-key typing delays, uneven scroll steps. | `info`, `move`, `click`, `type`, `scroll`, `delay` |

```rust
use xcelerate::{Browser, BrowserConfig};

let config = BrowserConfig {
    plugins: Some(vec!["stealth".to_string(), "human".to_string()]),
    ..Default::default()
};
let browser = Browser::launch(config).await?;

// Drive the human plugin through the same cross-language bridge.
let human = browser.plugin("human".into())?;
human.invoke("move".into(), r#"{"x": 320, "y": 240}"#.into()).await?;
human.invoke("type".into(), r#"{"text": "hello"}"#.into()).await?;
```

Each op runs under the invocation budget and is written to the audit log; a
plugin only ever acts on pages it has been handed.

### Inspecting and invoking plugins

Every binding exposes the same tiny, fixed bridge, so a new plugin never requires
new binding code:

| Method | Purpose |
| --- | --- |
| `available_plugins()` | Names of the compiled-in built-in catalog |
| `plugin_names()` | Plugins enabled on this browser |
| `use_plugin(name)` | Enable a built-in plugin at runtime |
| `install_plugins([plugin])` | Install trusted plugins compiled in as a Cargo library (Rust only) |
| `load_plugin(path)` | Load a sandboxed plugin (a directory or `plugin.json`) |
| `plugin(name)` | A handle to an enabled plugin |
| `plugin(name).ops()` | The operations the plugin exposes |
| `plugin(name).invoke(op, args_json)` | Invoke an operation with JSON args, returning JSON |

```rust
let enabled = browser.plugin_names();            // e.g. ["stealth", "human"]
let catalog = browser.available_plugins();       // ["stealth", "human"]
let stealth = browser.plugin("stealth".into())?; // error if not enabled
let info = stealth.invoke("info".into(), "{}".into()).await?;
```

### Where plugins run and what they may do

| Model | Where it runs | Privileges |
| --- | --- | --- |
| Built-in | In-process, compiled in | Launch flags, binary patching, detached spawn, init scripts |
| Loaded from disk | WebAssembly, sandboxed, capability-gated | Default-deny subset, audited |

Capabilities are classified before they can ever be granted. `LaunchControl`,
`BinaryPatch`, and `DetachedSpawn` are **built-in only**; `Evaluate`,
`CdpProxy`, cookie access, init scripts, screenshots, and network capture are
**dangerous** and require explicit consent. A plugin loaded from disk is a
WebAssembly component that runs sandboxed with **no ambient authority**: the
host imports are the only way out, they are capability-gated, and every call is
audited - so a plugin cannot reach the filesystem or the network on its own.
Dangerous callbacks are **denied by default** and must be opted into per host via
`XCELERATE_PLUGIN_ALLOW` (per plugin, or broadly), under per-invocation time and
response-size budgets. See [`docs/plugins/`](docs/plugins/README.md).

### Audit log

Every privileged action (launch configuration, page hooks, and every `invoke`) is
recorded in an append-only, hash-chained audit log. Secrets such as cookies and
credentials are never written to it.

```rust
assert!(browser.audit_verify()); // the hash chain is intact
println!("{}", browser.audit_log());
```

Plugins loaded from disk remain the user's responsibility to trust: the engine's
job is to make what they *can* do explicit, auditable, and impossible by default.

### Making a mod

Scaffold a starter mod and build it in one step - see the
[guide](docs/plugins/MAKING_A_MOD.md):

```bash
xcelerate plugin new acme.hello
cd hello && ./build.sh          # Windows:  .\build.ps1
```

You write plain Rust op handlers; xcelerate handles the WebAssembly plumbing.

## API-style adapters

The adapter layer exposes familiar Selenium, Playwright, and Puppeteer method names
over the native engine, so existing scripts can be ported with minimal changes. Each
adapter is generated from a declarative profile and is fully typed in Rust and Python.

Rust:

```rust
use xcelerate::adapters::playwright;

let browser = playwright::launch(None).await?;
let page = browser.new_page("https://example.com".to_string()).await?;

let paragraphs = page.query_selector_all("p".to_string()).await?;
println!("{} paragraph(s)", paragraphs.len());

browser.close().await?;
```

Python:

```python
from xcelerate import use

pw = use("playwright")
browser = await pw.launch()
page = await browser.new_page()
await page.goto("https://example.com")
print(await page.title())
await browser.close()
```

The same engine is available through `use("selenium")` and `use("puppeteer")`. See
[`adapters/README.md`](adapters/README.md) for the profile format and how to extend
an adapter.

## Native bindings

The generated bindings expose the same asynchronous API in each language.

Python:

```python
import asyncio
from xcelerate import Browser, BrowserConfig

async def main():
    # Opt into the stealth plugin - nothing runs unless it is enabled.
    browser = await Browser.launch(BrowserConfig(plugins=["stealth"]))
    page = await browser.new_page("https://example.com")
    print(await page.title())
    print(await browser.plugin_names())   # ["stealth"]
    await browser.close()

asyncio.run(main())
```

JavaScript (Node.js):

```javascript
const { Browser } = require("xcelerate");

async function main() {
    const browser = await Browser.launch({ plugins: ["stealth"] });
    const page = await browser.newPage("https://example.com");
    console.log(await page.title());
    await browser.close();
}

main();
```

## Command-line interface

The `xcelerate` command performs one browser action per invocation:

```bash
xcelerate title https://example.com
xcelerate screenshot https://example.com -o shot.png --full
xcelerate query https://example.com h1 --attr href
xcelerate query-all https://example.com 'a'   # text of every match
xcelerate evaluate https://example.com 'document.title'
xcelerate list                                  # built-in devices + plugins
xcelerate --device "iPhone 13" screenshot https://example.com -o phone.png
xcelerate plugins
xcelerate snapshot https://example.com          # indexed, LLM-friendly snapshot
xcelerate click-index https://example.com 2     # click element [2] from the snapshot
```

Global flags apply to every command: `--no-headless`, `--detached`,
`--executable-path <path>`, `--plugins stealth,human`, `--device <name>`, and
`--timeout <ms>`. `xcelerate --device <name> <command>` renders as a built-in
mobile device, and `xcelerate list` prints every device and plugin.
Install it with `cargo install --path crates/xcelerate-cli`, or
`winget install Chaosware.Xcelerate` on Windows; from a checkout, prefix any
command with `cargo run -p xcelerate-cli --`.

## MCP server

`xcelerate-mcp` - also reachable as `xcelerate mcp` - is a
[Model Context Protocol](https://modelcontextprotocol.io) server over stdio, so
an MCP client can drive a real browser. It exposes 21 tools covering navigation,
titles, page content, screenshots, PDFs, clicking, typing, hovering, key presses,
querying, JavaScript evaluation, and plugin invocation.

```jsonc
{ "mcpServers": { "xcelerate": { "command": "xcelerate-mcp" } } }
```

Configure it with the environment: `XCELERATE_CHROME` (browser path),
`XCELERATE_HEADLESS` (`1`/`true`, default), `XCELERATE_DETACHED` (`1`/`true`), and
`XCELERATE_PLUGINS` (comma-separated external plugin paths to load).

## Video recording

A page can be recorded to a video file. Recording is driven by the CDP
screencast, so the capture is change-driven: an animating page produces real
motion, while a static page yields a short clip.

```rust
use xcelerate::{Browser, BrowserConfig};

let browser = Browser::launch(BrowserConfig::default()).await?;
let page = browser.new_page("https://example.com".to_string()).await?;

page.start_video("demo.mp4".to_string()).await?;
tokio::time::sleep(std::time::Duration::from_secs(5)).await;
let path = page.stop_video().await?;   // the path actually written
```

The output extension selects the back end:

- `.mp4` / `.mov` / `.mkv` / `.webm` are muxed with `ffmpeg` (H.264 or VP9) at
  the frames' real timestamps when `ffmpeg` is on `PATH`. If it is missing the
  frames fall back to a sibling `.avi`, and the returned path says which was used.
- Any other extension (for example `.avi`) always uses the built-in Motion-JPEG
  AVI writer, which needs no external tools.

`start_video_with_options` takes tuning knobs (`quality`, `max_width`,
`max_height`, `fps`, `ffmpeg`). This API is intentionally not exported through
UniFFI yet, so it is available in Rust, the CLI, and the MCP server without
changing the generated bindings' checksums.

From the CLI:

```bash
xcelerate record https://example.com -o demo.mp4 --duration 5
```

From the MCP server: `browser_start_recording {path}` … `browser_wait
{milliseconds}` … `browser_stop_recording`.

## Proxy

A configured proxy applies to every page in the browser. Chrome's
`--proxy-server` cannot carry credentials and cannot switch proxies, so
xcelerate runs a small local HTTP/CONNECT gateway on `127.0.0.1` and points
Chrome at it; the gateway forwards each connection to an upstream chosen from a
**pool** and injects `Proxy-Authorization`.

```text
Chrome --(HTTP/CONNECT)--> xcelerate gateway (127.0.0.1) --> upstream pool --> internet
```

```bash
# environment (works from every language binding)
XCELERATE_PROXY=http://user:pass@proxy.example:8080 xcelerate title https://example.com
XCELERATE_PROXY_POOL=http://a:8080,http://b:8080 ./your-app      # round-robin

# CLI flag (repeatable)
xcelerate --proxy http://user:pass@proxy.example:8080 --proxy http://backup:8080 \
  title https://example.com
```

```rust
xcelerate::configure_proxy(&["http://user:pass@proxy.example:8080".to_string()])?;
let browser = Browser::launch(BrowserConfig::default()).await?;
```

- Upstreams are `http://[user:pass@]host:port`; the pool is rotated round-robin.
- HTTPS uses the proxy's `CONNECT` tunnel; plain HTTP uses absolute-form requests -
  both flow through the upstream with credentials injected.
- SOCKS upstreams are unnecessary: Chrome speaks SOCKS natively via
  `--proxy-server`, so point Chrome at it directly.
- `https://` upstreams (TLS to the proxy) are not supported yet.

## Persistent profiles

By default each browser gets a throwaway profile that is deleted on close. Point
it at a directory to keep cookies, logins, and site storage between runs:

```bash
XCELERATE_USER_DATA_DIR=~/.xcelerate/profile xcelerate title https://example.com
xcelerate --user-data-dir ./profile title https://example.com
```

```rust
xcelerate::configure_user_data_dir(Some("./profile".to_string()))?;
```

The path is canonicalized (a relative `--user-data-dir` would otherwise resolve
against Chrome's own cwd), and `Browser::close` lets Chrome flush the profile to
disk before it exits.

## Accessibility snapshots

`page.accessibility_snapshot()` returns a compact semantic view of the page -
`[{ role, name, value? }]` in document order - which is far more resilient than
CSS selectors for asserting or driving a page. It is exposed as the CLI command
`xcelerate accessibility <url>` and the MCP tool `browser_accessibility`.

## Agent snapshots

`page.agent_snapshot()` renders the page as indented text where every
interactive element is tagged with a stable `[index]`:

```
[0]<link> "Home"
[1]<textbox> "Email" = "a@b.com"
[2]<button> "Sign in"
```

The snapshot is built entirely in Rust from a single
`Accessibility.getFullAXTree` and `DOMSnapshot.captureSnapshot` call on the
persistent CDP session, so it is far cheaper and more predictable than
serializing the DOM in a scripting language. Pass an index to
`page.click_index(n)` to click that element without re-resolving a CSS selector
(`page.snapshot_json()` returns the same elements with roles, names, bounds, and
backend node ids). It is exposed as the CLI commands `xcelerate snapshot <url>`
and `xcelerate click-index <url> <index>`, and the MCP tools `browser_snapshot`
and `browser_click_index`.

## Workspace layout

```
xcelerate/
  crates/
    xcelerate-core/        # WebSocket transport and typed CDP command layer
    xcelerate-plugin/      # plugin trait, manifest, capabilities, audit, host interface
    xcelerate/             # high-level facade: Browser, Page, Element, adapters
    xcelerate-bindgen/     # uniffi bindgen helper binary
    xcelerate-cli/         # CLI (binary `xcelerate`)
    xcelerate-mcp/         # `xcelerate-mcp` Model Context Protocol server
  plugins/
    stealth/               # stealth plugin: binary patching + anti-fingerprint payload
    human/                 # human plugin: human-like mouse, typing, and scrolling
  adapters/             # adapter profiles, runtime, and generator inputs
  bindings/             # generated Python, JavaScript, C#, Kotlin, Java, Swift, Ruby, Dart, and Go packages (plus the PowerShell module)
  docs/plugins/         # plugin authoring guide, JSON schema, examples
  scripts/              # code generation, harvesting, and release tooling
```

The plugin API - the `Plugin` trait, `Manifest`, capabilities, audit log, and the
`PageHost` interface - lives in `crates/xcelerate-plugin/`. **No plugins are built
into the core**: `plugins/` holds external plugin crates (`stealth`, `human`) that
an embedder installs in-process, or that are built to `.wasm` and loaded
sandboxed. The `stealth` crate owns its binary patcher and the anti-fingerprint
payload; the engine owns browser process control
(`crates/xcelerate/src/process.rs`). The facade owns the `PluginManager` and
bridges `Page` to the plugin host interface, so plugins never touch a raw page or
the transport.

## Development

```bash
# Generate the API-style adapters (Python and Rust) from the profiles.
python scripts/generate_adapters.py

# Validate the profiles and print coverage.
python scripts/harvest_adapters.py --check

# Build, lint, and test.
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test -p xcelerate --lib                    # plugin + unit tests (no browser)
cargo test -p xcelerate --test adapters_e2e -- --nocapture

# JVM bindings (JDK 22+, Gradle, uniffi-bindgen-java).
python scripts/install_toolchains.py
```

The end-to-end tests launch a real browser and require Chrome or Edge. Point them at a
different site or browser with `XCELERATE_TEST_URL` and `XCELERATE_CHROME`.

## Security

Found a vulnerability? Please follow [SECURITY.md](SECURITY.md) - do not open a
public issue. Supported versions, reporting channels, and scope are listed there.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for
inclusion in this project by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
