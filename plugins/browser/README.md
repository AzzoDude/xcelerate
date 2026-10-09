# xcelerate.browser

The **browser control surface as a plugin**. It implements the `plugin` interface
(`wit/plugin.wit`) and forwards every op to the host's semantic browser bridge,
`host.browser` - never raw CDP/BiDi.

| | |
| --- | --- |
| Name | `xcelerate.browser` |
| Capability | `browser` (dangerous: off by default, grant-audited) |
| Bridge | `host.browser(op, args)` |
| Artifact | `browser.wasm` |

## Ops

| op | args | host verb |
| --- | --- | --- |
| `open` | `{ "url": "https://…" }` | `goto` |
| `title` | `{}` | `title` |
| `url` | `{}` | `url` |
| `text` | `{}` | `text` |
| `html` | `{}` | `html` |
| `markdown` | `{}` | `markdown` |
| `snapshot` | `{}` | `snapshot` |
| `click` | `{ "target": "#submit" \| "Sign in" }` | `click` |
| `hover` | `{ "target": "#menu" }` | `hover` |
| `fill` | `{ "target": "#email", "text": "a@b.c" }` | `fill` |
| `press` | `{ "key": "Enter" }` | `press` |
| `scroll` | `{ "to": "down" \| "up" \| "top" \| "bottom" \| "800" }` | `scroll` |
| `wait` | `{ "selector": "#done" }` | `wait` |
| `find` | `{ "text": "hello" }` | `find` |
| `evaluate` | `{ "js": "document.title" }` | `evaluate` |
| `screenshot` | `{ "full": true }` | `screenshot` |

## Build

The CLI owns the build: it writes `wit/plugin.wit` (the canonical host ABI) and
stages `browser.wasm` beside `plugin.json`.

```bash
xcelerate build --wasm-only
```

`build.sh` / `build.ps1` are wrappers around that.

## Install and use

Copy this directory into the shared plugin home so any project or script can
import it:

```bash
mkdir -p ~/.xcl/plugins
cp -r . ~/.xcl/plugins/browser        # -> ~/.xcl/plugins/browser/{plugin.json,browser.wasm}
```

The `browser` capability is **dangerous**, so it must be granted explicitly with
`XCELERATE_PLUGIN_ALLOW`; the XCL `run` verb is *also* gated by `--allow-plugin`:

```bash
XCELERATE_PLUGIN_ALLOW=browser xcelerate --plugins browser run --allow-plugin browser job.xcl
```

```xcl
# job.xcl
run browser open {"url":"https://example.com"}
run browser find {"text":"Example"}
```

From Rust:

```rust
// `load_plugin` takes a directory (or a `plugin.json`); the bare-name search is
// a CLI convenience, so a Rust caller passes the staged path.
let dir = format!("{}/.xcl/plugins/browser", std::env::var("HOME").unwrap());
browser.load_plugin(dir)?;
let page = browser.new_page().await?;
let out = browser
    .plugin("xcelerate.browser".to_string())?
    .invoke_on("open".into(), r#"{"url":"https://example.com"}"#.into(), page)
    .await?;
```

## Layout

```
browser/
  Cargo.toml      # cdylib + wit-bindgen
  plugin.json     # name, ops, capabilities (browser), limits
  build.sh/.ps1   # wrappers around `xcelerate build --wasm-only`
  src/lib.rs      # the op -> host-verb table
  wit/plugin.wit  # the interface contract (written by `xcelerate build`)
```
