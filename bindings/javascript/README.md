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
    // Stealth is opt-in: nothing runs unless a plugin is enabled.
    const browser = await Browser.launch({ plugins: ['stealth'] });

    const page = await browser.newPage("https://www.google.com");
    console.log("Title:", await page.title());

    await browser.close();
}

main().catch(console.error);
```

`Browser.launch()` accepts a plain config object. The defaults are
`{ headless: true, detached: true, executable_path: null, plugins: null }`; pass
`plugins: ['stealth', 'human']` to enable the built-in plugins.
`availablePlugins()` returns `['stealth', 'human']`; use `pluginNames()` and
`plugin(name)` to inspect and drive them at runtime.

## Requirements

- Node.js 16+
- Windows (Current support)
