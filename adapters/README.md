# API-style adapters

Xcelerate ships a **native** API (`navigate`, `find_element`, `type_text`, ...).
Many users arrive from **Selenium**, **Playwright**, or **Puppeteer** and don't
want to rewrite their scripts. The adapters expose those libraries' method
names on top of the same xcelerate engine, so a one-line import is enough to
switch style:

```python
from xcelerate import use

pw = use("playwright")
browser = await pw.launch()
page = await browser.new_page()
await page.goto("https://example.com")
await page.click("#submit")
print(await page.inner_text("h1"))
await browser.close()
```

```python
from xcelerate import use

sel = use("selenium")
driver = await sel.launch()
await driver.get("https://example.com")
element = await driver.find_element("css selector", "#submit")
await element.click()
await driver.quit()
```

## Stealth and plugins

Stealth is no longer on by default: it is a first-party **plugin** you opt into
through the launch config. Every adapter's `launch(config)` forwards that config
to the core `Browser`, so enabling stealth works the same in all three styles:

```python
from xcelerate import BrowserConfig, use

config = BrowserConfig(plugins=["stealth", "human"])  # opt into first-party plugins
pw = use("playwright")
browser = await pw.launch(config)
```

Nothing runs unless it is listed (default-deny). Third-party plugins load
out-of-process with `Browser::load_plugin` and speak a line-delimited JSON-RPC
protocol, so they can be written in any language. See the
[plugins section](../README.md#plugins) of the top-level README for the trust
tiers, capabilities, and the append-only audit log.

## How it works

```
adapters/
|-- profiles/            # DATA  - one JSON per target library
|   |-- playwright.json
|   |-- selenium.json
|   |-- puppeteer.json
|-- runtime.py           # CODE  - the ops, implemented once
|-- discovered/          # OUTPUT - harvester snapshots (optional)

scripts/
|-- generate_adapters.py       # profiles -> Python wrappers + Rust modules + support
|-- backfill_impls.py          # populate impls, inject classes, --prune out-of-scope members
|-- harvest_adapters.py        # upstream libs -> coverage report / snapshots / --check
```

One generator consumes the profiles and emits both target languages:

* **`generate_adapters.py`** emits thin **Python** wrappers into
  `bindings/python/xcelerate/adapters/` and **Rust** adapters into
  `crates/xcelerate/src/adapters/`. Use `--target python` or `--target rust` to
  emit just one. The Rust side generates every file, including `mod.rs` and
  `support.rs`, so the whole `adapters/` directory is reproducible. Supported
  methods carry typed signatures and call the core; unsupported methods return
  `XcelerateError::Unsupported`.

* **`profiles/*.json`** declare, per target library, the classes and method
  names and which runtime op each maps to. This is the only thing you edit to
  add coverage - no codegen code changes.
* **`runtime.py`** is the single implementation of every op. It talks to the
  xcelerate core and absorbs library quirks (Selenium `(By, value)` selectors,
  Playwright `full_page`, Puppeteer camelCase). It is copied into the generated
  package as `xcelerate/adapters/_runtime.py`.
* **`harvest_adapters.py`** introspects the installed libraries, snapshots their
  surface, and stubs the gaps. `--check` validates every profile op against
  `runtime.py` and the Rust op table and prints a coverage report (no libraries
  required), so it can gate CI.

## Adding or refreshing an adapter

1. Harvest what the library currently exposes:

   ```bash
   python scripts/harvest_adapters.py --write
   ```

   This writes `adapters/discovered/<name>.json` and prints which upstream
   methods are **not yet mapped** by the profile.

   For JavaScript libraries (Puppeteer) there is no Python package to import,
   so introspect it at runtime with Node instead:

   ```bash
   # one-off scratch install of the type definitions (no browser download)
   npm install --prefix .probe-puppeteer puppeteer-core
   node scripts/harvest_puppeteer_js.js ./.probe-puppeteer/node_modules/puppeteer-core \
     > adapters/discovered/puppeteer.json
   python scripts/harvest_adapters.py --from-json adapters/discovered/puppeteer.json --update
   ```

2. Map the useful ones in `adapters/profiles/<name>.json`. A method entry is:
   ```json
   { "name": "goto", "op": "page_goto", "params": ["url"], "returns": null }
   ```
   * `name` - the target library's method name (use `"as"` if it is not a valid
     Python identifier, e.g. Puppeteer's `$` -> `query_selector`).
   * `op` - a function in `runtime.py`.
   * `params` - `"arg"` or `{ "name": "arg", "default": "None" }`.
   * `returns` - a profile class name to wrap the result in, else `null`.

3. Regenerate:

   ```bash
   python scripts/generate_adapters.py          # Python + Rust
   python scripts/generate_adapters.py --target rust   # Rust only
   python scripts/harvest_adapters.py --check   # validate + coverage
   ```

   `scripts/generate_all.py` runs this automatically as part of the binding
   pipeline.

## Adding a new adapter (e.g. a new library)

1. Add `adapters/profiles/<name>.json`.
2. Reuse ops from `runtime.py`; add a new op only if the library needs
   behaviour xcelerate doesn't already express.
3. Run the generator. `use("<name>")` becomes available.

## Scope and honesty

These adapters match **method names and call shapes**, not the full semantics of
each library. Known gaps:

* **Selectors** - CSS only. Selenium's XPath raises `NotImplementedError`.
* **Sync APIs** - xcelerate is async end-to-end, so all adapters are `async`.
  Selenium users should `await` (there is no blocking shim yet).
* **Waits/timeouts** - xcelerate applies its own 30s timeout; a `timeout=`
  argument is accepted for shape but not yet honoured.
* **Missing primitives** - a few upstream APIs have no faithful equivalent
  (callback-style `expose_function`/`on(handler)`, Chrome-specific PWA/extension
  APIs, Playwright-only `video`/`pick_locator`/`clock`). These members are
  **removed** from the profiles rather than faked, so the adapter never silently
  claims behaviour it does not have.

Profiles are **supported-only**: every declared member has a real backing (a
named op, an inline JS `impl`, or a `cdp` template). Members that xcelerate
cannot faithfully express are **removed** by
`python scripts/backfill_impls.py --prune` rather than left as stubs, so
`harvest_adapters.py --check` always reports `0 stubs`.

`harvest_adapters.py --update` still records newly discovered upstream members
as candidate stubs; run `backfill_impls.py` to map them (or `--prune` to drop
them). `scripts/generate_all.py` runs populate+prune automatically before
generating the adapters.
