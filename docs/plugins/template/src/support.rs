//! Small helpers every mod shares: MessagePack pack/unpack, the `describe`
//! handshake, and the `mod_ops!` macro that wires your ops to the host.

use serde::de::DeserializeOwned;
use serde::Serialize;
use std::collections::BTreeMap;

/// MessagePack-encode a value for the host.
pub fn pack<T: Serialize>(value: &T) -> Result<Vec<u8>, String> {
    rmp_serde::to_vec_named(value).map_err(|error| error.to_string())
}

/// MessagePack-decode a value from the host.
pub fn unpack<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, String> {
    rmp_serde::from_slice(bytes).map_err(|error| error.to_string())
}

/// The `describe` handshake payload: `{ name, version, ops, schema_version?,
/// schemas? }`. `schema_version` (omitted = none, `1` = config supported)
/// advertises whether `schemas` carries per-op `{ schema, defaults }` JSON
/// strings. Legacy hosts ignore the extra keys.
#[derive(Serialize)]
struct Describe<'a> {
    name: &'a str,
    version: &'a str,
    ops: &'a [&'a str],
    #[serde(skip_serializing_if = "Option::is_none")]
    schema_version: Option<u32>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    schemas: BTreeMap<&'static str, OpSchema>,
}

/// A per-op config carried in `describe`: the JSON Schema input object and the
/// flat defaults, both as JSON strings.
#[derive(Serialize, Clone)]
pub struct OpSchema {
    pub schema: &'static str,
    #[serde(skip_serializing_if = "str::is_empty")]
    pub defaults: &'static str,
}

/// Encode the `describe` handshake (MessagePack), with no config.
pub fn describe(name: &str, ops: &[&str]) -> Vec<u8> {
    describe_with_schema(name, ops, &BTreeMap::new())
}

/// Encode the `describe` handshake (MessagePack), with optional per-op config.
pub fn describe_with_schema(
    name: &str,
    ops: &[&str],
    schemas: &BTreeMap<&'static str, OpSchema>,
) -> Vec<u8> {
    let schema_version = if schemas.is_empty() { None } else { Some(1) };
    pack(&Describe {
        name,
        version: env!("CARGO_PKG_VERSION"),
        ops,
        schema_version,
        schemas: schemas.clone(),
    })
    .unwrap_or_default()
}

/// Implement the `plugin` world and export it.
///
/// Each op maps to a `fn(Vec<u8>) -> Result<Vec<u8>, String>` that takes the
/// MessagePack args and returns the MessagePack result. Call it once, at the
/// bottom of `lib.rs`:
///
/// ```ignore
/// mod_ops! {
///     name: "{{name}}",
///     "echo" => echo,
///     "add"  => add,
/// }
/// ```
#[macro_export]
macro_rules! mod_ops {
    (
        name: $name:literal,
        $($op:literal => $handler:path),* $(,)?
    ) => {
        struct Mod;

        impl exports::xcelerate::plugin::plugin::Guest for Mod {
            fn describe() -> Vec<u8> {
                $crate::support::describe($name, &[$($op),*])
            }

            fn invoke(op: String, args: Vec<u8>) -> Result<Vec<u8>, String> {
                match op.as_str() {
                    $($op => $handler(args),)*
                    other => Err(format!("{}: unknown op '{}'", $name, other)),
                }
            }
        }

        export!(Mod);
    };
}

/// Like [`mod_ops!`], but also advertises per-op config (JSON Schema input
/// object + defaults) in the `describe` handshake, so hosts and bindings can
/// discover typed inputs and defaults without reading the DOM.
///
/// ```ignore
/// mod_ops_schema! {
///     name: "acme.user",
///     schemas: [create_user_schemas()],
///     "create_user" => create_user,
/// }
/// ```
///
/// Each `schemas` entry is a `BTreeMap<&'static str, OpSchema>` keyed by op.
#[macro_export]
macro_rules! mod_ops_schema {
    (
        name: $name:literal,
        schemas: [$($schemas:expr),* $(,)?],
        $($op:literal => $handler:path),* $(,)?
    ) => {
        struct Mod;

        impl exports::xcelerate::plugin::plugin::Guest for Mod {
            fn describe() -> Vec<u8> {
                use std::collections::BTreeMap;
                let mut schemas = BTreeMap::new();
                $( schemas.extend($schemas.clone()); )*
                $crate::support::describe_with_schema($name, &[$($op),*], &schemas)
            }

            fn invoke(op: String, args: Vec<u8>) -> Result<Vec<u8>, String> {
                match op.as_str() {
                    $($op => $handler(args),)*
                    other => Err(format!("{}: unknown op '{}'", $name, other)),
                }
            }
        }

        export!(Mod);
    };
}
