# Xcelerate

[![NuGet](https://img.shields.io/nuget/v/Xcelerate.svg)](https://www.nuget.org/packages/Xcelerate)
[![Crates.io](https://img.shields.io/crates/v/xcelerate.svg)](https://crates.io/crates/xcelerate)
[![Documentation](https://img.shields.io/badge/docs.rs-xcelerate-blue)](https://docs.rs/xcelerate)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Xcelerate is a high-performance, lightweight Chrome DevTools Protocol (CDP) client
with idiomatic bindings for Rust, .NET, Python, JavaScript (Node.js), Kotlin, and
Java. It pairs a fast Rust core with an async-first API and a data-driven adapter
layer that lets existing Selenium, Playwright, and Puppeteer scripts run against
the same engine.

## Features

- **Automated process management** - discovers and launches Chrome or Edge, and
  manages the lifecycle of the browser process.
- **Stealth by default** - optional binary patching and a runtime JavaScript payload
  that reduces common automation fingerprints.
- **Async-first** - built on `tokio` in Rust and `async`/`await` in every binding.
- **API-style adapters** - expose Selenium, Playwright, and Puppeteer method names on
  top of the native engine, generated from declarative profiles.
- **Multi-language bindings** - one core, generated bindings for Rust, Python, Node.js,
  .NET, Kotlin, and Java via `uniffi`.

## Installation

### Rust

```toml
[dependencies]
xcelerate = "1.0.7"
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

The JVM bindings are built from source (they are not on Maven Central yet):

```bash
python scripts/install_toolchains.py        # JDK 22+ via winget, Gradle, uniffi-bindgen-java
python scripts/generate_kotlin_bindings.py  # Kotlin sources + Gradle build
python scripts/generate_java_bindings.py    # Java sources + Gradle build
```

See [`bindings/kotlin/README.md`](bindings/kotlin/README.md) and
[`bindings/java/README.md`](bindings/java/README.md) for usage.

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

`BrowserConfig` defaults to headless mode with stealth enabled. Disable either when
you do not need it:

```rust
use xcelerate::BrowserConfig;

let config = BrowserConfig {
    headless: false,
    stealth: false,
    detached: false,
    executable_path: None, // auto-discover Chrome/Edge
};
```

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
    browser = await Browser.launch(BrowserConfig())
    page = await browser.new_page("https://example.com")
    print(await page.title())
    await browser.close()

asyncio.run(main())
```

JavaScript (Node.js):

```javascript
const { Browser, BrowserConfig } = require("xcelerate");

async function main() {
    const browser = await Browser.launch(new BrowserConfig());
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
    xcelerate-core/     # WebSocket transport and typed CDP command layer
    xcelerate-stealth/  # binary patching, detached processes, stealth payload
    xcelerate/          # high-level facade: Browser, Page, Element, adapters
    xcelerate-bindgen/  # uniffi bindgen helper binary
  adapters/             # adapter profiles, runtime, and generator inputs
  bindings/             # generated Python, JavaScript, C#, Kotlin, and Java packages
  scripts/              # code generation, harvesting, and release tooling
```

## Development

```bash
# Generate the API-style adapters (Python and Rust) from the profiles.
python scripts/generate_adapters.py

# Validate the profiles and print coverage.
python scripts/harvest_adapters.py --check

# Build, lint, and test.
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test -p xcelerate --test adapters_e2e -- --nocapture

# JVM bindings (JDK 22+, Gradle, uniffi-bindgen-java).
python scripts/install_toolchains.py
```

The end-to-end tests launch a real browser and require Chrome or Edge. Point them at a
different site or browser with `XCELERATE_TEST_URL` and `XCELERATE_CHROME`.

## License

Distributed under the MIT License.
