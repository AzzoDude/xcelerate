//! Small helpers every mod shares: MessagePack pack/unpack, the `describe`
//! handshake, and the `mod_ops!` macro that wires your ops to the host.

use serde::de::DeserializeOwned;
use serde::Serialize;

/// MessagePack-encode a value for the host.
pub fn pack<T: Serialize>(value: &T) -> Result<Vec<u8>, String> {
    rmp_serde::to_vec_named(value).map_err(|error| error.to_string())
}

/// MessagePack-decode a value from the host.
pub fn unpack<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, String> {
    rmp_serde::from_slice(bytes).map_err(|error| error.to_string())
}

/// The `describe` handshake payload: `{ name, version, ops }`.
#[derive(Serialize)]
struct Describe<'a> {
    name: &'a str,
    version: &'a str,
    ops: &'a [&'a str],
}

/// Encode the `describe` handshake (MessagePack).
pub fn describe(name: &str, ops: &[&str]) -> Vec<u8> {
    pack(&Describe {
        name,
        version: env!("CARGO_PKG_VERSION"),
        ops,
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
