//! `acme.notes` - a plugin that depends on another plugin.
//!
//! `save`/`load`/`list` are all backed by the **`acme:kv/store`** interface,
//! which this component *imports*. There is no store implementation here and no
//! filesystem access: the host links the import to the `acme.kv` component. This
//! is the whole point of the dependency example - a typed, cross-plugin call
//! with each plugin still sandboxed on its own.
//!
//! Build: `cargo build --release --target wasm32-wasip2` - see the README.

wit_bindgen::generate!({
    path: "wit",
    world: "notes-plugin",
    generate_all,
});

use serde::{Deserialize, Serialize};

/// Prefix we namespace our keys with so `acme.kv` stays generic.
const PREFIX: &str = "note:";

#[derive(Serialize, Deserialize)]
struct Describe {
    name: String,
    version: String,
    ops: Vec<String>,
}

struct Notes;

impl exports::xcelerate::plugin::plugin::Guest for Notes {
    fn describe() -> Vec<u8> {
        let describe = Describe {
            name: "acme.notes".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            ops: ["save", "load", "titles"]
                .iter()
                .map(|op| (*op).to_string())
                .collect(),
        };
        rmp_serde::to_vec_named(&describe).unwrap_or_default()
    }

    fn invoke(op: String, args: Vec<u8>) -> Result<Vec<u8>, String> {
        match op.as_str() {
            // args: [title, body] -> key
            "save" => {
                let (title, body): (String, String) =
                    rmp_serde::from_slice(&args).map_err(|error| error.to_string())?;
                let key = notes_save(title, body)?;
                rmp_serde::to_vec_named(&key).map_err(|error| error.to_string())
            }
            // args: title -> option<string>
            "load" => {
                let title: String =
                    rmp_serde::from_slice(&args).map_err(|error| error.to_string())?;
                let body = notes_load(title)?;
                rmp_serde::to_vec_named(&body).map_err(|error| error.to_string())
            }
            // args: () -> list<string>
            "titles" => {
                let titles = notes_list()?;
                rmp_serde::to_vec_named(&titles).map_err(|error| error.to_string())
            }
            other => Err(format!("acme.notes: unknown op '{other}'")),
        }
    }
}

impl exports::acme::notes::notes::Guest for Notes {
    fn save(title: String, body: String) -> Result<String, String> {
        notes_save(title, body)
    }

    fn load(title: String) -> Result<Option<String>, String> {
        notes_load(title)
    }

    fn titles() -> Result<Vec<String>, String> {
        notes_list()
    }
}

/// Every call below crosses into the `acme.kv` component through the host linker.
fn notes_save(title: String, body: String) -> Result<String, String> {
    let key = format!("{PREFIX}{title}");
    acme::kv::store::set(&key, &body).map_err(|error| format!("acme.kv: {error}"))?;
    Ok(key)
}

fn notes_load(title: String) -> Result<Option<String>, String> {
    acme::kv::store::get(&format!("{PREFIX}{title}")).map_err(|error| format!("acme.kv: {error}"))
}

fn notes_list() -> Result<Vec<String>, String> {
    let mut keys = acme::kv::store::keys();
    keys.retain(|key| key.starts_with(PREFIX));
    Ok(keys
        .into_iter()
        .map(|key| key.trim_start_matches(PREFIX).to_string())
        .collect())
}

export!(Notes);
