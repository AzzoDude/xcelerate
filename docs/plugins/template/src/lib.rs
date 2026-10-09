//! `{{name}}` - a xcelerate mod compiled to a WebAssembly component.
//!
//! Args and results travel as MessagePack through the typed `plugin` interface in
//! `wit/plugin.wit`; the host converts to/from each language's own types. Build
//! with `xcelerate build --wasm-only`, then import the `.wasm` and `plugin.json`.

use serde::{Deserialize, Serialize};

use support::{pack, unpack};

mod support;

wit_bindgen::generate!({
    path: "wit/plugin.wit",
    world: "plugin-world",
});

/// Arguments to the `echo` op.
#[derive(Serialize, Deserialize)]
pub struct EchoArgs {
    pub message: String,
}

/// Result of the `echo` op.
#[derive(Serialize, Deserialize)]
pub struct EchoResult {
    pub message: String,
    pub from: String,
}

/// Echo the message back, and record it in the host's audit log.
fn echo(args: Vec<u8>) -> Result<Vec<u8>, String> {
    let EchoArgs { message } = unpack(&args)?;
    xcelerate::plugin::host::log(&message);
    pack(&EchoResult {
        message,
        from: "{{name}}".to_string(),
    })
}

mod_ops! {
    name: "{{name}}",
    "echo" => echo,
}
