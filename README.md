# xcelerate 🚀

A high-performance, lightweight Chrome DevTools Protocol (CDP) client for Rust.

Built for speed and developer experience, `xcelerate` provides a clean, chained API for browser automation that feels like `chromiumoxide` but with a minimalist core.

## Features

- **Zero-Config**: Automatic browser detection and launching.
- **Fluent API**: Chained methods for intuitive automation scripts.
- **Async First**: Built on `tokio` and `futures`.
- **Type-Safe**: Full support for `browser-protocol` and `js-protocol`.
- **Lightweight**: Minimal dependencies, fast compilation.

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
xcelerate = "0.1.0"
```

### Basic Example

```rust
use xcelerate::{Browser, BrowserConfig, XcelerateResult};

#[tokio::main]
async fn main() -> XcelerateResult<()> {
    // Launch browser automatically
    let (browser, handler) = Browser::launch(
        BrowserConfig::builder().headless(false).build()?
    ).await?;
    
    // Run the event handler in the background
    tokio::spawn(handler.run());

    // Clean, chained automation
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

## Running Examples

Check out the `examples/` directory:

```bash
cargo run --example google_search
```

## License

MIT License - Copyright (c) 2026 Nguyễn Quý Ngọc
