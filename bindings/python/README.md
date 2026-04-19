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
    # Launch browser with intelligent defaults
    # (Optional: headless=True, stealth=True, detached=True, executable_path=None)
    config = BrowserConfig()
    browser = await Browser.launch(config)
    
    # Create a new page
    print("Opening Pixelscan...")
    page = await browser.new_page("https://pixelscan.net/bot-check")
    
    # Wait for result to load
    print("Waiting 10 seconds for bot check...")
    await asyncio.sleep(10)
    
    # Interact with the page
    print(f"Title: {await page.title()}")
    
    # Take a screenshot
    print("Capturing screenshot...")
    screenshot = await page.screenshot_full()
    with open("result.png", "wb") as f:
        f.write(screenshot)
    
    await browser.close()

if __name__ == "__main__":
    asyncio.run(main())
```

## Advanced Configuration

The `BrowserConfig` object allows you to fine-tune the browser behavior:

- **stealth (default: True)**: Applies binary patches and JS masking to bypass bot detection.
- **detached (default: True)**: Spawns the browser as an independent process that stays open even if your script finishes.
- **headless (default: True)**: Runs the browser without a visible window.
- **executable_path (default: None)**: Manually specify the location of Chrome or Edge.

## License

MIT
