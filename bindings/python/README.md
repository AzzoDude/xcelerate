# Xcelerate Python SDK

A high-performance, lightweight Chrome DevTools Protocol (CDP) client for Python, designed for speed and stealth.

## Features

- **Blazing Fast**: Direct CDP communication over WebSockets.
- **Security-first plugins**: Default-deny; `stealth` is a built-in plugin you opt into.
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
    # Launch with intelligent defaults (headless=True, stealth=False,
    # detached=True, executable_path=None). Plugins are opt-in:
    # (Optional: plugins=["stealth", "human"])
    config = BrowserConfig(plugins=["stealth"])
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

- **plugins (default: None)**: Built-in plugins to enable, e.g.
  `["stealth", "human"]`. Nothing runs unless it is listed here (default-deny).
  `stealth` masks automation fingerprints; `human` makes input behave like a
  person (`info`, `move`, `click`, `type`, `scroll`, `delay`).
- **detached (default: True)**: Spawns the browser as an independent process that stays open even if your script finishes.
- **headless (default: True)**: Runs the browser without a visible window.
- **executable_path (default: None)**: Manually specify the location of Chrome or Edge.

### Plugins

Inspect and drive plugins at runtime with the cross-language bridge:

```python
print(browser.plugin_names())       # ['stealth']
print(browser.available_plugins())  # ['stealth', 'human']
stealth = browser.plugin("stealth")
print(await stealth.invoke("info", "{}"))
```

Plugins are loaded from disk out-of-process behind the capability gate: the
`entrypoint` in the manifest is spawned and its ops are reachable through
`plugin(name).invoke(op, args_json)`. See the
[top-level README](../../README.md#plugins) for capabilities and the append-only
audit log.

## License

MIT
