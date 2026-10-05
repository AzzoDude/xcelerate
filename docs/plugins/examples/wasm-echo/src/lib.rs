//! `example.wasm-echo` - a minimal xcelerate plugin compiled to WebAssembly.
//!
//! Implements the `plugin` interface from `wit/plugin.wit`. Op arguments and
//! results are MessagePack-encoded (not JSON): compact and cheap to decode, and
//! the host/guest structs stay typed through `serde`.
//!
//! Build: `cargo build --release --target wasm32-wasip2` (the target emits a
//! Component Model component directly; see the README).

wit_bindgen::generate!({
    path: "../../../../crates/xcelerate/wit/plugin.wit",
    world: "plugin-world",
});

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Manifest {
    name: String,
    version: String,
    ops: Vec<String>,
}

struct WasmEcho;

impl exports::xcelerate::plugin::plugin::Guest for WasmEcho {
    fn describe() -> Vec<u8> {
        let manifest = Manifest {
            name: "example.wasm-echo".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            ops: vec!["echo".to_string(), "log".to_string()],
        };
        rmp_serde::to_vec_named(&manifest).unwrap_or_default()
    }

    fn invoke(op: String, args: Vec<u8>) -> Result<Vec<u8>, String> {
        match op.as_str() {
            // Echo the arguments straight back.
            "echo" => Ok(args),
            // Ask the host to log a message (always-allowed host callback).
            "log" => {
                let message: String = rmp_serde::from_slice(&args).unwrap_or_default();
                xcelerate::plugin::host::log(&message);
                rmp_serde::to_vec_named(&"logged").map_err(|error| error.to_string())
            }
            other => Err(format!("unknown op '{other}'")),
        }
    }
}

export!(WasmEcho);
