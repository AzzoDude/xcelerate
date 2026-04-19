# Xcelerate

[![NuGet](https://img.shields.io/nuget/v/Xcelerate.svg)](https://www.nuget.org/packages/Xcelerate)
[![Crates.io](https://img.shields.io/crates/v/xcelerate.svg)](https://crates.io/crates/xcelerate)
[![Documentation](https://img.shields.io/badge/docs.rs-xcelerate-blue)](https://docs.rs/xcelerate)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

Xcelerate is a high-performance, lightweight Chrome DevTools Protocol (CDP) client designed for Rust, .NET, and Python. It provides a modular architecture that combines a fast Rust core with idiomatic wrappers for every major language.

## Key Features

- **Automated Process Management**: Seamlessly discovers and initializes Chrome or Edge binaries on Windows.
- **Advanced Stealth Integration**: Built-in binary patching and runtime JavaScript payloads to neutralize automation detection.
- **Async Implementation**: Fully optimized for `tokio` in Rust and `Task`-based async/await in C#.
- **Fluent API**: Designed for readability with chained method patterns for common interactions like clicking, typing, and hovering.
- **Headless=New Support**: Utilizes the modern Chrome headless engine for superior compatibility with state-of-the-art web applications.

## Installation

### Rust
Add the following to your `Cargo.toml`:

```toml
[dependencies]
xcelerate = "0.1.3"
tokio = { version = "1.0", features = ["full"] }
```

### .NET / C#
```powershell
dotnet add package Xcelerate
```

### Python
```bash
pip install xcelerate
```

## Usage Examples

### Rust Implementation
```rust
use xcelerate::{Browser, BrowserConfig, XcelerateResult};

#[tokio::main]
async fn main() -> XcelerateResult<()> {
    let (browser, handler) = Browser::launch(
        BrowserConfig::builder().headless(true).stealth(true).build()?
    ).await?;
    tokio::spawn(handler.run());

    let page = browser.new_page("https://www.example.com").await?;
    let title = page.title().await?;
    println!("Title: {}", title);

    Ok(())
}
```

### Python Implementation
Xcelerate for Python offers full `asyncio` support with a very lightweight API.

```python
import asyncio
from xcelerate import Browser, BrowserConfig

async def main():
    # Intelligent defaults: headless=True, stealth=True, detached=True
    browser = await Browser.launch(BrowserConfig())
    
    page = await browser.new_page("https://www.example.com")
    print(f"Title: {await page.title()}")
    
    await browser.close()

if __name__ == "__main__":
    asyncio.run(main())
```

### C# Implementation
Xcelerate offers idiomatic .NET support with standard `IDisposable` patterns for resource management.

```csharp
using Xcelerate;

// Launch browser with modern headless mode and stealth patches
// (Optional: headless=true, stealth=true, detached=true)
using var browser = await Browser.Launch(new BrowserConfig());

// Create a new page and perform navigation
using var page = await browser.NewPageAsync("https://pixelscan.net/");

// Wait for a selector and extract results
using var element = await page.WaitForSelectorAsync("body");
string title = await page.GetTitleAsync();
Console.WriteLine($"Page Title: {title}");

// Capture full-page documentation of results
byte[] screenshot = await page.ScreenshotFullAsync();
File.WriteAllBytes("result.png", screenshot);
```

## Advanced Capabilities

### Stealth and Anti-Detection
Xcelerate implements a defense-in-depth strategy to bypass bot detection services:
- **Binary Patching**: Actively replaces `cdc_` signatures in the browser binary.
- **Runtime Masking**: Injects a hardened JavaScript payload to hide `navigator.webdriver`, mock `window.chrome`, and protect the Permissions API.
- **Detached Logic**: Supports spawning browser instances that persist independently of the parent application.

## Development and Contributions

Xcelerate is actively maintained. To contribute or modify the cross-language bindings, refer to the automation scripts located in the `scripts/` directory.

## License
Distributed under the MIT License.
