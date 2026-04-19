# Xcelerate Python SDK

A high-performance, lightweight Chrome DevTools Protocol (CDP) client for Python, designed for speed and stealth.

## Features

- **Blazing Fast**: Direct CDP communication over WebSockets.
- **Stealth by Design**: Built-in anti-bot protection and binary patching.
- **Universal**: Native bindings for high performance.
- **Async/Await**: Full support for Python's asyncio.

## Installation

```bash
pip install xcelerate
```

## Quick Start

```python
import asyncio
from xcelerate import Browser, BrowserConfig

async def main():
    # Launch browser with stealth enabled
    config = BrowserConfig(headless=True, stealth=True)
    browser = await Browser.launch(config)
    
    # Create a new page
    page = await browser.new_page("https://pixelscan.net/bot-check")
    
    # Interact with the page
    print(f"Title: {await page.title()}")
    
    # Take a screenshot
    screenshot = await page.screenshot_full()
    with open("result.png", "wb") as f:
        f.write(screenshot)
    
    await browser.close()

if __name__ == "__main__":
    asyncio.run(main())
```

## License

MIT
