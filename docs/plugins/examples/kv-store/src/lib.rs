//! `acme.kv` - a shared, sandboxed key/value store plugin.
//!
//! Implements two exported interfaces:
//!
//! * `xcelerate:plugin/plugin` - the standard plugin surface (`describe`,
//!   `invoke`) the host drives it through; and
//! * `acme:kv/store` - the typed API other plugins import and depend on.
//!
//! The store lives in the component's own linear memory (a sandbox has no
//! filesystem), so it is process-lifetime state shared between whichever
//! plugins the host wires to it.
//!
//! Build: `cargo build --release --target wasm32-wasip2` - see the README.

wit_bindgen::generate!({
    path: "wit",
    world: "kv-plugin",
    generate_all,
});

use std::collections::BTreeMap;
use std::sync::Mutex;

use serde::{Deserialize, Serialize};

/// The store itself. A `Mutex` keeps the two exported interfaces consistent even
/// though a component is single-threaded today.
static STORE: Mutex<BTreeMap<String, String>> = Mutex::new(BTreeMap::new());

/// The handshake payload `describe` returns (MessagePack).
#[derive(Serialize, Deserialize)]
struct Describe {
    name: String,
    version: String,
    ops: Vec<String>,
}

fn with_store<T>(f: impl FnOnce(&mut BTreeMap<String, String>) -> T) -> T {
    let mut guard = STORE
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    f(&mut guard)
}

struct KvStore;

impl exports::xcelerate::plugin::plugin::Guest for KvStore {
    fn describe() -> Vec<u8> {
        let describe = Describe {
            name: "acme.kv".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            ops: ["get", "set", "del", "keys"]
                .iter()
                .map(|op| (*op).to_string())
                .collect(),
        };
        rmp_serde::to_vec_named(&describe).unwrap_or_default()
    }

    fn invoke(op: String, args: Vec<u8>) -> Result<Vec<u8>, String> {
        match op.as_str() {
            // args: [key, value]
            "set" => {
                let (key, value): (String, String) =
                    rmp_serde::from_slice(&args).map_err(|error| error.to_string())?;
                store_set(key, value)?;
                rmp_serde::to_vec_named(&()).map_err(|error| error.to_string())
            }
            // args: key
            "get" => {
                let key: String =
                    rmp_serde::from_slice(&args).map_err(|error| error.to_string())?;
                let value = store_get(key)?;
                rmp_serde::to_vec_named(&value).map_err(|error| error.to_string())
            }
            // args: key -> whether it existed
            "del" => {
                let key: String =
                    rmp_serde::from_slice(&args).map_err(|error| error.to_string())?;
                let removed = store_delete(key)?;
                rmp_serde::to_vec_named(&removed).map_err(|error| error.to_string())
            }
            // args: () -> all keys
            "keys" => {
                let keys = store_keys();
                rmp_serde::to_vec_named(&keys).map_err(|error| error.to_string())
            }
            other => Err(format!("acme.kv: unknown op '{other}'")),
        }
    }
}

impl exports::acme::kv::store::Guest for KvStore {
    fn set(key: String, value: String) -> Result<(), String> {
        store_set(key, value)
    }

    fn get(key: String) -> Result<Option<String>, String> {
        store_get(key)
    }

    fn delete(key: String) -> Result<bool, String> {
        store_delete(key)
    }

    fn keys() -> Vec<String> {
        store_keys()
    }
}

fn store_set(key: String, value: String) -> Result<(), String> {
    if key.trim().is_empty() {
        return Err("acme.kv: empty key".to_string());
    }
    with_store(|store| {
        store.insert(key, value);
    });
    Ok(())
}

fn store_get(key: String) -> Result<Option<String>, String> {
    Ok(with_store(|store| store.get(&key).cloned()))
}

fn store_delete(key: String) -> Result<bool, String> {
    Ok(with_store(|store| store.remove(&key).is_some()))
}

fn store_keys() -> Vec<String> {
    with_store(|store| store.keys().cloned().collect())
}

export!(KvStore);
