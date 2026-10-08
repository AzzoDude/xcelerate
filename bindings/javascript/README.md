# Xcelerate for Node.js

A high-performance Chrome DevTools Protocol (CDP) client for Node.js, built on a fast Rust core.

## Installation

```bash
npm install xcelerate
```

## Quick Start

```javascript
const { Browser } = require('xcelerate');

async function main() {
    // plugins: none by default — load one from disk to opt in
    const browser = await Browser.launch();

    const page = await browser.newPage("https://www.google.com");
    console.log("Title:", await page.title());

    await browser.close();
}

main().catch(console.error);
```

`Browser.launch()` accepts a plain config object. The defaults are
`{ headless: true, detached: true, executable_path: null, plugins: null }`. The
`plugins` config takes paths to plugin directories (or `plugin.json` manifests).
Plugins are default-deny and the built-in catalog is empty; nothing runs unless
you load one from disk. Opt in with `loadPlugin(path)` then `usePlugin(name)`,
and use `pluginNames()` (loaded on this browser) and `availablePlugins()` (the
ones it can load) to inspect.

## Requirements

- Node.js 16+
- Windows (Current support)
