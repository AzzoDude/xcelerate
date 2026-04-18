# xcelerate

[![Crates.io](https://img.shields.io/crates/v/xcelerate.svg)](https://crates.io/crates/xcelerate)
[![Documentation](https://docs.rs/xcelerate/badge.svg)](https://docs.rs/xcelerate)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A high-performance, lightweight **Chrome DevTools Protocol (CDP)** client for Rust. Built for speed and developer experience, `xcelerate` provides a clean, chained API for browser automation that feels like `chromiumoxide` but with a minimalist, "Zero-Config" core.

## 🚀 Features

- **Zero-Config**: Automatic discovery and launching of Chrome/Edge on Windows.
- **Fluent API**: Chained methods for intuitive automation scripts (Type, Click, Hover).
- **Handshake Recovery**: Reliable debugger connection via HTTP handshake.
- **Event Broadcasting**: Built-in system to subscribe to browser-wide events.
- **Async Ready**: Fully optimized for `tokio` and `futures`.

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
xcelerate = "0.1.0"
tokio = { version = "1.0", features = ["full"] }
```

## 🛠 Usage Example

```rust
use xcelerate::{Browser, BrowserConfig, XcelerateResult};

#[tokio::main]
async fn main() -> XcelerateResult<()> {
    // 1. Launch browser automatically
    let (browser, handler) = Browser::launch(
        BrowserConfig::builder().headless(false).build()?
    ).await?;
    
    // 2. Run the event handler in the background
    tokio::spawn(handler.run());

    // 3. Clean, chained automation
    let page = browser.new_page("https://www.google.com").await?;
    
    page.find_element("input[name='q']")
        .await?
        .type_text("Xcelerate Rust Automation")
        .await?
        .click()
        .await?;

    Ok(())
}
```

## 🏗 Why xcelerate?
Unlike other CDP wrappers that can be heavy or complex to set up, `xcelerate` focuses on the "First 5 Minutes" experience. It handles the messy process launching, port polling, and PID management so you can focus on your automation logic.

## ⚖ License
Distributed under the MIT License. See `LICENSE` for more information.

---
*Developed by Nguyễn Quý Ngọc*
