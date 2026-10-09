//! Compile-test: `plugin!` must expand against the same crate-root shapes that
//! `wit_bindgen::generate!` produces in a real plugin - the exported `Guest`
//! trait under `exports::xcelerate::plugin::plugin`, and the imported host bridge
//! under `xcelerate::plugin::host`.
//!
//! This is a stand-in for the wasm build: it proves the generated `describe` /
//! `invoke` type-check and dispatch correctly, without a wasm32-wasip2 toolchain.

/// The exported guest interface (as `wit-bindgen` generates it).
mod exports {
    pub mod xcelerate {
        pub mod plugin {
            pub mod plugin {
                pub trait Guest {
                    fn describe() -> Vec<u8>;
                    fn invoke(op: String, args: Vec<u8>) -> Result<Vec<u8>, String>;
                }
            }
        }
    }
}

/// The imported host bridge (as `wit-bindgen` generates it).
mod xcelerate {
    pub mod plugin {
        pub mod host {
            pub fn browser(op: &str, args: &[u8]) -> Result<Vec<u8>, String> {
                let _ = args;
                // Echo the op name back so the test can assert dispatch.
                Ok(op.as_bytes().to_vec())
            }

            pub fn app(op: &str, args: &[u8]) -> Result<Vec<u8>, String> {
                let _ = args;
                Ok(op.as_bytes().to_vec())
            }
        }
    }
}

use xcelerate_plugin::plugin;

struct Browser;

plugin! {
    guest = Browser,
    name = "browser",
    bridge = browser,
    ops = {
        open => "goto",
        click,
    },
}

/// A plugin mixing forwarded ops with locally-implemented ones.
struct App;

/// A `local` op: named by its op, it composes host primitives itself.
fn tree(args: &[u8]) -> Result<Vec<u8>, String> {
    let _ = args;
    xcelerate::plugin::host::app("snapshot", &[])
}

fn click(args: &[u8]) -> Result<Vec<u8>, String> {
    let _ = args;
    Ok(b"local-click".to_vec())
}

plugin! {
    guest = App,
    name = "app",
    bridge = app,
    local = { tree, click },
    ops = {
        launch,
    },
}

#[test]
fn describe_lists_the_ops_in_declaration_order() {
    use exports::xcelerate::plugin::plugin::Guest;

    let payload = Browser::describe();
    let described: Describe = rmp_serde::from_slice(&payload).unwrap();

    assert_eq!(described.name, "browser");
    assert_eq!(described.ops, vec!["open", "click"]);
}

#[test]
fn invoke_dispatches_to_the_host_bridge() {
    use exports::xcelerate::plugin::plugin::Guest;

    // `open` forwards to the `goto` host verb (the bridge echo shows it).
    let out = Browser::invoke("open".to_string(), Vec::new()).unwrap();
    assert_eq!(out, b"goto");

    let out = Browser::invoke("click".to_string(), Vec::new()).unwrap();
    assert_eq!(out, b"click");

    assert!(Browser::invoke("missing".to_string(), Vec::new()).is_err());
}

#[test]
fn local_ops_are_described_and_dispatched_here() {
    use exports::xcelerate::plugin::plugin::Guest;

    let described: Describe = rmp_serde::from_slice(&App::describe()).unwrap();
    assert_eq!(described.name, "app");
    // Forwarded and local ops both appear.
    assert_eq!(described.ops, vec!["launch", "tree", "click"]);

    // A local op runs the plugin's own function (not the host bridge)...
    let out = App::invoke("click".to_string(), Vec::new()).unwrap();
    assert_eq!(out, b"local-click");
    // ...which may itself call host primitives (`tree` echoes `snapshot`).
    let out = App::invoke("tree".to_string(), Vec::new()).unwrap();
    assert_eq!(out, b"snapshot");
    // A forwarded op still goes to the bridge.
    let out = App::invoke("launch".to_string(), Vec::new()).unwrap();
    assert_eq!(out, b"launch");
}

#[derive(serde::Deserialize)]
struct Describe {
    name: String,
    ops: Vec<String>,
}
