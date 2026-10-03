# Xcelerate

[![Crates.io](https://img.shields.io/crates/v/xcelerate.svg)](https://crates.io/crates/xcelerate)
[![PyPI](https://img.shields.io/pypi/v/xcelerate.svg)](https://pypi.org/project/xcelerate/)
[![npm](https://img.shields.io/npm/v/xcelerate.svg)](https://www.npmjs.com/package/xcelerate)
[![NuGet](https://img.shields.io/nuget/v/Xcelerate.svg)](https://www.nuget.org/packages/Xcelerate)
[![Maven Central](https://img.shields.io/maven-central/v/io.github.azzodude/xcelerate.svg)](https://central.sonatype.com/artifact/io.github.azzodude/xcelerate)
[![Documentation](https://img.shields.io/badge/docs.rs-xcelerate-blue)](https://docs.rs/xcelerate)
[![License: MIT OR Apache-2.0](https://img.shields.io/badge/License-MIT%20OR%20Apache--2.0-blue.svg)](#license)

Xcelerate is a high-performance, lightweight Chrome DevTools Protocol (CDP) client
with idiomatic bindings for Rust, .NET, Python, JavaScript (Node.js), Kotlin,
Java, Swift, Ruby, Dart/Flutter, and Go. It pairs a fast Rust core with an
async-first API and a data-driven adapter layer that lets existing Selenium,
Playwright, and Puppeteer scripts run against the same engine.

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
| Dart / Flutter | `xcelerate` | built from source ([readme](bindings/dart/README.md)) |
| Go | `xcelerate` | built from source ([readme](bindings/go/README.md)) |

## Features

- **Automated process management** - discovers and launches Chrome or Edge, and
  manages the lifecycle of the browser process.
- **Security-first plugins** - a default-deny plugin system with explicit trust
  tiers and an append-only audit log. First-party plugins you opt into:
  `stealth` and `human`.
- **Stealth and human plugins** - `stealth` applies binary patching and a runtime
  JavaScript payload that reduce automation fingerprints; `human` makes input
  behave like a person (Bezier mouse travel, paced typing, uneven scrolling).
  Both are enabled per browser through `BrowserConfig.plugins`.
- **Async-first** - built on `tokio` in Rust and `async`/`await` in every binding.
- **API-style adapters** - expose Selenium, Playwright, and Puppeteer method names on
  top of the native engine, generated from declarative profiles.
- **Multi-language bindings** - one core, generated bindings for Rust, Python, Node.js,
  .NET, Kotlin, and Java via `uniffi`.

## Installation

### Rust

```toml
[dependencies]
xcelerate = "1.0.9"
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
    implementation("io.github.azzodude:xcelerate:1.0.8")        // Kotlin
    implementation("io.github.azzodude:xcelerate-java:1.0.8")   // Java
}
```

The Java bindings require **JDK 22+** and run with `--enable-native-access=ALL-UNNAMED`.
See [`bindings/kotlin/README.md`](bindings/kotlin/README.md) and
[`bindings/java/README.md`](bindings/java/README.md) for usage.

Building them from source:

```bash
python scripts/install_toolchains.py        # JDK 22+ via winget, Gradle, uniffi-bindgen-java
python scripts/generate_kotlin_bindings.py  # Kotlin sources + Gradle build
python scripts/generate_java_bindings.py    # Java sources + Gradle build
```

### Swift / Ruby / Dart / Go

These targets are built from source with the universal generator. Swift and Ruby
are built-in UniFFI targets; Dart needs `uniffi-bindgen-dart`; Go needs the
NordSecurity `uniffi-bindgen-go` generator and a Go toolchain.

```bash
python scripts/generate_swift_bindings.py    # Swift sources + SwiftPM package
python scripts/generate_ruby_bindings.py     # Ruby sources + gemspec
cargo install uniffi-bindgen-dart            # once
python scripts/generate_dart_bindings.py     # Dart sources + pubspec
go install github.com/NordSecurity/uniffi-bindgen-go/v2/uniffi-bindgen-go@latest
python scripts/generate_go_bindings.py       # Go sources + go.mod
```

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

> The `stealth: bool` field still exists as **deprecated sugar**: `stealth: true`
> is equivalent to adding `"stealth"` to `plugins`, and it will be removed in a
> future major release.

## Plugins

Xcelerate ships a **security-first plugin system**. A plugin is a named bundle of
launch-time configuration, page hooks, and invokable operations, and it does
nothing unless you enable it (**default-deny**). `stealth` is the first-party
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
| `available_plugins()` | Names of the compiled-in first-party catalog |
| `plugin_names()` | Plugins enabled on this browser |
| `use_plugin(name)` | Enable a first-party plugin at runtime |
| `load_plugin(path)` | Load a third-party plugin - **refused** in this phase |
| `plugin(name)` | A handle to an enabled plugin |
| `plugin(name).ops()` | The operations the plugin exposes |
| `plugin(name).invoke(op, args_json)` | Invoke an operation with JSON args, returning JSON |

```rust
let enabled = browser.plugin_names();            // e.g. ["stealth", "human"]
let catalog = browser.available_plugins();       // ["stealth", "human"]
let stealth = browser.plugin("stealth".into())?; // error if not enabled
let info = stealth.invoke("info".into(), "{}".into()).await?;
```

### Trust tiers and capabilities

| Tier | Where it runs | Privileges |
| --- | --- | --- |
| First-party | In-process, compiled in | Launch flags, binary patching, detached spawn, init scripts |
| Third-party | Out-of-process, OS-sandboxed, capability-gated | Default-deny subset (not implemented yet) |

Capabilities are classified before they can ever be granted. `LaunchControl`,
`BinaryPatch`, and `DetachedSpawn` are **first-party only**; `Evaluate`,
`CdpProxy`, cookie access, init scripts, screenshots, and network capture are
**dangerous** and require explicit consent. Third-party plugins are designed to
run out-of-process behind a default-deny capability proxy with per-invocation
time and response-size budgets - but until that runner exists, `load_plugin`
**refuses** to execute unknown code rather than silently trusting it.

### Audit log

Every privileged action (launch configuration, page hooks, and every `invoke`) is
recorded in an append-only, hash-chained audit log. Secrets such as cookies and
credentials are never written to it.

```rust
assert!(browser.audit_verify()); // the hash chain is intact
println!("{}", browser.audit_log());
```

Third-party plugins remain the user's responsibility to trust: the engine's job
is to make what they *can* do explicit, auditable, and impossible by default.

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

## Workspace layout

```
xcelerate/
  crates/
    xcelerate-core/        # WebSocket transport and typed CDP command layer
    xcelerate-plugin-api/  # plugin trait, manifest, capabilities, audit, host interface
    xcelerate-plugins/     # first-party plugins (stealth, human) + OS helpers
    xcelerate/             # high-level facade: Browser, Page, Element, adapters
    xcelerate-bindgen/     # uniffi bindgen helper binary
  adapters/             # adapter profiles, runtime, and generator inputs
  bindings/             # generated Python, JavaScript, C#, Kotlin, Java, Swift, Ruby, Dart, and Go packages
  docs/plugins/         # third-party plugin authoring guide, JSON schema, examples
  scripts/              # code generation, harvesting, and release tooling
```

The plugin API - the `Plugin` trait, `Manifest`, capabilities, audit log, and the
`PageHost` interface - lives in `crates/xcelerate-plugin-api/`. The first-party
implementations (`stealth`, `human`) and the low-level OS helpers (binary
patching, process management, payload) live in `crates/xcelerate-plugins/`. The
facade owns the `PluginManager` and bridges `Page` to the plugin host interface,
so plugins never touch a raw page or the transport.

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
