# Xcelerate for Node.js

A high-performance Chrome DevTools Protocol (CDP) client for Node.js, built on a fast Rust core.

## Installation

```bash
npm install xcelerate
```

## Quick Start

```javascript
const { Browser, BrowserConfig } = require('xcelerate');

async function main() {
    // Launch browser with intelligent defaults
    const config = new BrowserConfig(); 
    const browser = await Browser.launch(config);
    
    const page = await browser.newPage("https://www.google.com");
    console.log("Title:", await page.title());
    
    await browser.close();
}

main().catch(console.error);
```

## Requirements

- Node.js 16+
- Windows (Current support)
