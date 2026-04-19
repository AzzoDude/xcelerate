# xcelerate

[![NuGet](https://img.shields.io/nuget/v/Xcelerate.svg)](https://www.nuget.org/packages/Xcelerate)
[![Crates.io](https://img.shields.io/crates/v/xcelerate.svg)](https://crates.io/crates/xcelerate)
[![Documentation](https://img.shields.io/badge/docs.rs-xcelerate-blue)](https://docs.rs/xcelerate)
[![GitHub](https://img.shields.io/github/stars/AzzoDude/xcelerate.svg?style=social)](https://github.com/AzzoDude/xcelerate)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A high-performance, lightweight **Chrome DevTools Protocol (CDP)** client for Rust and .NET. Built for speed and developer experience, `xcelerate` provides a clean, chained API for browser automation that feels like `chromiumoxide` but with a minimalist, "Zero-Config" core.

## 🚀 Features

- **Zero-Config**: Automatic discovery and launching of Chrome/Edge on Windows.
- **Fluent API**: Chained methods for intuitive automation scripts (Type, Click, Hover).
- **Handshake Recovery**: Reliable debugger connection via HTTP handshake.
- **Async Ready**: Fully optimized for `tokio` and `futures`.
- **Safe .NET Wrapper**: Modern, managed C# API with no `unsafe` blocks required.

## 📦 Installation

### 🦀 Rust
Add this to your `Cargo.toml`:

```toml
[dependencies]
xcelerate = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

### 🔷 C# / .NET
The C# wrapper is **NativeAOT compatible** and 100% managed (safe).

```powershell
dotnet add package Xcelerate
```

## 🛠 Usage Examples

### Rust (Native)
```rust
use xcelerate::{Browser, BrowserConfig, XcelerateResult};

#[tokio::main]
async fn main() -> XcelerateResult<()> {
    let (browser, handler) = Browser::launch(
        BrowserConfig::builder().headless(false).build()?
    ).await?;
    tokio::spawn(handler.run());

    let page = browser.new_page("https://www.google.com").await?;
    page.find_element("input[name='q']")
        .await?
        .type_text("Xcelerate Rust")
        .await?
        .click()
        .await?;

    Ok(())
}
```

### C# (.NET)
`xcelerate` provides a clean, modern .NET API with support for both **Synchronous** and **Asynchronous** programming models.

#### ⚡ Synchronous (Simple Scripts)
```csharp
using Xcelerate;

// 1. Launch & Automatic Cleanup (Disposes on exit)
using var browser = Browser.Launch(headless: false);

// 2. High-level Managed API
using var page = browser.NewPage("https://www.google.com");
Console.WriteLine($"Title: {page.GetTitle()}");

// 3. Navigation & Interaction
page.Navigate("https://github.com");
using var search = page.WaitForSelector("input[name='q']");
search.TypeText("Xcelerate");
```

#### 🌐 Asynchronous (Modern Applications)
```csharp
using Xcelerate;

// 1. Launch Async
using var browser = await Browser.LaunchAsync(headless: true);

// 2. Multi-step Async Logic
using var page = await browser.NewPageAsync("https://www.google.com");
await page.NavigateAsync("https://github.com");

var title = await page.GetTitleAsync();
Console.WriteLine($"Title: {title}");

// 3. Selector Handling
using var element = await page.WaitForSelectorAsync("input[name='q']");
await element.TypeTextAsync("Xcelerate SDK");
await element.ClickAsync();
```

## 🛠️ Build & Development
To regenerate the C# bindings after modifying the Rust source, use the provided automation script:

```powershell
python scripts/generate_cs_wrapper.py
```

## 🏗 Why xcelerate?
Unlike other CDP wrappers that can be heavy or complex to set up, `xcelerate` focuses on the "First 5 Minutes" experience. It handles the messy process launching, port polling, and PID management so you can focus on your automation logic.

## ⚖ License
Distributed under the MIT License. See `LICENSE` for more information.

---
*Developed by AzzoDude*
