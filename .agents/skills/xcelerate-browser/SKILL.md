---
name: xcelerate-browser
description: Drive a real Chrome/Edge browser with xcelerate - the xcelerate-mcp Model Context Protocol tools (navigate, screenshot, click, type, query, evaluate, plugin invoke) or the xcelerate-cli commands. Use for browser automation, scraping rendered pages, capturing screenshots/PDFs, or testing a web UI.
---

# xcelerate browser automation

Xcelerate is a Chrome DevTools Protocol engine. There are two entry points: prefer
the **MCP tools** inside an agent loop, and the **CLI** for one-shot terminal work.

## MCP server (`xcelerate-mcp`, also `xcelerate-cli mcp`)

A stdio Model Context Protocol server. The browser launches lazily on the first
tool call and stays alive for the session; call `browser_close` when finished.

| Tool | Purpose |
| --- | --- |
| `browser_navigate {url}` | Open a URL (starts the browser session if needed). |
| `browser_title`, `browser_url` | Current title / document URL. |
| `browser_get_content`, `browser_get_text` | Full HTML / visible text. |
| `browser_screenshot {full?}` | PNG image result; `full: true` for the whole page. |
| `browser_pdf` | PDF result. |
| `browser_click {selector}` | Click the first match. |
| `browser_type {selector, text, submit?}` | Focus + realistic keystrokes (`submit` presses Enter). |
| `browser_fill {selector, value}` | Set an input's value directly and fire input/change. |
| `browser_hover {selector}`, `browser_press {selector, key}` | Hover / press a key. |
| `browser_query {selector, attribute?, html?}` | Text (default), an attribute, or inner HTML. |
| `browser_query_all {selector}` | Text of every match. |
| `browser_evaluate {expression}` | Evaluate JS, returns JSON. |
| `browser_wait_for_selector {selector}` | Wait for an element to appear. |
| `browser_go_back`, `browser_reload` | History navigation. |
| `browser_plugins`, `browser_plugin_invoke {name, op, args}` | Inspect / drive first-party plugins. |
| `browser_close` | Close the browser and end the session. |

### Conventions

- Selectors are CSS, and every selector-based tool waits for the element first.
- Navigate before querying; the session starts on `about:blank`.
- Prefer `browser_fill` for plain inputs; use `browser_type` when the page reacts
  to key events or you need `submit`.
- After a click that replaces content, `browser_wait_for_selector` for something
  on the new state before reading.
- Take a `browser_screenshot` when layout matters or the text is ambiguous.
- Never type credentials or secrets into a page unless the user asked for it.

### Configuration (environment)

`XCELERATE_CHROME` (browser path), `XCELERATE_HEADLESS` (`1`/`true`, default),
`XCELERATE_DETACHED` (`1`/`true`), `XCELERATE_PLUGINS` (e.g. `stealth,human`).

## CLI (`xcelerate-cli`)

One browser action per invocation; useful in a terminal or shell script.

```bash
xcelerate-cli title https://example.com
xcelerate-cli text https://example.com
xcelerate-cli screenshot https://example.com -o shot.png --full
xcelerate-cli pdf https://example.com -o page.pdf
xcelerate-cli query https://example.com h1 --attr href
xcelerate-cli query-all https://example.com 'a'      # text of every match
xcelerate-cli evaluate https://example.com 'document.title'
xcelerate-cli plugins
```

Global flags: `--no-headless`, `--detached`, `--executable-path <path>`,
`--plugins stealth,human`, `--timeout <ms>`.

## Running from source

```bash
cargo run -p xcelerate-cli -- title https://example.com
cargo run -p xcelerate-mcp            # speak JSON-RPC 2.0 over stdio
```

`xcelerate-cli` prints the result to stdout and exits non-zero on failure; the
MCP server writes only JSON-RPC to stdout (diagnostics go to stderr).
