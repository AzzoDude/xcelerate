# Xcelerate Python SDK

A high-performance, lightweight Chrome DevTools Protocol (CDP) client for Python, designed for speed.

## Features

- **Blazing Fast**: Direct CDP communication over WebSockets.
- **Security-first plugins**: Default-deny and loaded from disk; the built-in catalog is empty. Pass paths via `plugins` or use `load_plugin(path)` to opt in.
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
    # Launch with intelligent defaults (headless=True, detached=True,
    # executable_path=None). plugins: none by default — load one from disk to
    # opt in.
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

- **plugins (default: None)**: Paths to plugin directories (or `plugin.json`
  manifests) to load at launch. Plugins are default-deny and the built-in
  catalog is empty; nothing runs unless you load one from disk. Opt in at
  runtime with `load_plugin(path)`, then `use_plugin(name)`.
- **detached (default: True)**: Spawns the browser as an independent process that stays open even if your script finishes.
- **headless (default: True)**: Runs the browser without a visible window.
- **executable_path (default: None)**: Manually specify the location of Chrome or Edge.

### Plugins

Inspect and drive plugins at runtime with the cross-language bridge:

```python
print(browser.plugin_names())       # [] — empty until you load one
print(browser.available_plugins())  # [] — the built-in catalog is empty

browser.load_plugin("plugins/my-plugin")
print(browser.plugin_names())       # ['my-plugin']

plug = browser.plugin("my-plugin")
print(await plug.invoke("info", "{}"))
```

Plugins load from disk sandboxed in a WebAssembly store behind the capability
gate: the `entrypoint` component in the manifest is instantiated and its ops are
reachable through `plugin(name).invoke(op, args_json)`. See the
[top-level README](../../README.md#plugins) for capabilities and the append-only
audit log.

## License

MIT
