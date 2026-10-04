# Example plugin: `example.echo`

A minimal plugin that demonstrates the whole authoring contract. It
exposes one op, `echo`, which navigates to a URL and returns the page title plus
the text of the first matching selector - using only the two capabilities it
declares (`query`, `get_text`).

> **Status.** The manifest validates and `Browser::load_plugin` runs the
> `entrypoint` out-of-process behind the capability gate. See the
> [plugin guide](../README.md) for status.

## Files

```
examples/echo/
  plugin.json     # validated manifest (see below)
  README.md       # this file
```

In a real package you would also ship `example-echo.wasm`, the program named by
`entrypoint`.

## The manifest

```json
{
  "name": "example.echo",
  "version": "0.1.0",
  "host_api": ">=1.0 <2.0",
  "entrypoint": "example-echo.wasm",
  "abi": "wasm32-wasip2/1",
  "ops": ["echo"],
  "capabilities": ["query", "get_text"],
  "limits": { "max_invoke_millis": 5000, "max_response_bytes": 65536 }
}
```

Because the name is not reserved, an entrypoint is present, at least one op is
declared, no host-only capability is requested, and the limits are within
the host maxima, `Manifest::from_json` returns `Ok`.

## What the plugin author writes

The author compiles a program to the sandbox ABI and implements the exported ops
plus the host callbacks it needs. Nothing is called unless the corresponding
capability was granted to the manifest.

Pseudo-code (the real host API is a JSON RPC; the shape is the same in every
language):

```rust
// Compiles to wasm32-wasip2, runs in the sandbox.
#[no_mangle]
pub extern "C" fn invoke(op_ptr: *const u8, op_len: usize) -> *mut u8 {
    let op = read_request(op_ptr, op_len);
    match op.name.as_str() {
        "echo" => {
            let url = op.args["url"].as_str().unwrap_or("about:blank");
            host::navigate(url);                       // needs: navigate (not declared -> request it)
            let title = host::title();
            let selector = op.args["selector"].as_str().unwrap_or("h1");
            let text = host::get_text(selector);       // needs: get_text
            reply(json!({ "title": title, "text": text }))
        }
        other => reply_error(format!("unknown op: {other}")),
    }
}
```

Notice the author only reaches the page through `host::*` callbacks. There is no
filesystem handle, no socket, and no raw CDP in scope - those either do not exist
in the sandbox or require a dangerous capability that must be granted.

## How the host would invoke it

```rust
use xcelerate::{Browser, BrowserConfig};

let browser = Browser::launch(BrowserConfig::default()).await?;
browser.load_plugin("path/to/example.echo".into())?;
let handle = browser.plugin("example.echo")?;
let result = handle.invoke("echo".into(), r#"{"url":"https://example.com","selector":"h1"}"#.into()).await?;
println!("{result}");
# Ok::<(), xcelerate::XcelerateError>(())
```

`invoke` returns a JSON string, and the call is written to the audit log with the
plugin name and op. If the op runs longer than `max_invoke_millis` or returns more
than `max_response_bytes`, the host kills it and audits the overrun.

## Validating this manifest

```rust
use xcelerate::plugin::Manifest;

let manifest = Manifest::load(concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/plugins/examples/echo/plugin.json"))?;
assert_eq!(manifest.name, "example.echo");
# Ok::<(), xcelerate::XcelerateError>(())
```

Or validate the JSON against [`plugin.schema.json`](../plugin.schema.json) in any
editor/CI.
