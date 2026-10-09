# Plugins

Two ready-made xcelerate plugins that package the **browser** and **native-app**
control surfaces as portable WebAssembly components. They show the whole plugin
model end to end and are the decoupled home for browser/app control: the
interpreter and CLI stay small, and the surfaces evolve here.

| Plugin | Name | Ops | Capability |
| --- | --- | --- | --- |
| [`browser/`](browser) | `xcelerate.browser` | `open`, `title`, `url`, `text`, `html`, `markdown`, `snapshot`, `click`, `hover`, `fill`, `press`, `scroll`, `wait`, `find`, `evaluate`, `screenshot` | `browser` |
| [`app/`](app) | `xcelerate.app` | `windows`, `launch`, `tree`, `find`, `wait`, `click`, `set_value`, `key`, `wheel`, `scroll` | `app` |

Neither plugin speaks CDP, BiDi, or the OS directly. Each forwards its ops to a
**capability-gated host bridge** - `host.browser` for the browser and `host.app`
for native windows - so a sandboxed guest asks the host for a *semantic verb*
(`goto`, `click`, `fill`, `tree`, …) and the host performs it, audits it, and
enforces the grant. See `crates/xcelerate/wit/plugin.wit`.

## Build

The CLI owns the build: `xcelerate build --wasm-only` writes `wit/plugin.wit` (the
canonical host ABI) and stages the `.wasm` beside `plugin.json`. `build.sh` /
`build.ps1` are wrappers around it.

```bash
(cd browser && xcelerate build --wasm-only)   # -> browser.wasm
(cd app     && xcelerate build --wasm-only)   # -> app.wasm
```

## Install into the shared plugin home

Plugin *names* are resolved from the user-global home (`$XCELERATE_HOME/plugins`,
else `~/.xcl/plugins`), then `./plugins`, then `.`. Install once and import from
anywhere:

```bash
mkdir -p ~/.xcl/plugins
cp -r browser ~/.xcl/plugins/browser
cp -r app     ~/.xcl/plugins/app
```

## Use

The `browser` and `app` capabilities are **dangerous**, so they are denied unless
granted via `XCELERATE_PLUGIN_ALLOW`; the XCL `run` verb is *also* gated by
`--allow-plugin`.

```bash
# browser plugin
XCELERATE_PLUGIN_ALLOW=browser xcelerate --plugins browser run --allow-plugin browser job.xcl

# app plugin (Windows only)
XCELERATE_PLUGIN_ALLOW=app xcelerate --plugins app run --allow-plugin app job.xcl
```

```xcl
# job.xcl
run browser open {"url":"https://example.com"}
run browser find {"text":"Example"}
run app launch {"target":"notepad","title":"Notepad"}
```

From Rust, bind the browser plugin to a live page with `PluginHandle::invoke_on`:

```rust
// `load_plugin` takes a directory (or a `plugin.json`); the bare-name search is
// a CLI convenience, so a Rust caller passes the staged path.
let dir = format!("{}/.xcl/plugins/browser", std::env::var("HOME").unwrap());
browser.load_plugin(dir)?;
let page = browser.new_page().await?;
browser
    .plugin("xcelerate.browser".to_string())?
    .invoke_on("open".into(), r#"{"url":"https://example.com"}"#.into(), page)
    .await?;
```

See [`docs/plugins/`](../docs/plugins/README.md) for the manifest, capability,
and packaging reference, and `crates/xcelerate/wit/plugin.wit` for the ABI.
