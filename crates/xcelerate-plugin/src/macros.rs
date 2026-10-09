//! Authoring macros for WebAssembly plugins.
//!
//! [`plugin!`] generates a plugin's `describe` handshake and `invoke`
//! dispatcher from a declarative op table, so a plugin author writes no
//! boilerplate and never touches the ABI by hand. Each op forwards its raw
//! arguments to a host bridge verb (`host.browser`, `host.desktop`, `host.core`);
//! `=> "verb"` renames the verb when it differs from the op name.
//!
//! A declarative macro (not a `#[proc_macro]`) is used deliberately: a
//! `proc-macro` must live in its own crate, so folding it here keeps
//! `xcelerate-plugin` the single crate a plugin author depends on.
//!
//! An op may instead be **handled locally** when it needs composition rather
//! than a 1:1 forward - a `wait` that polls `snapshot` + `sleep`, a `tree` that
//! formats a snapshot, and so on. List those ops under `local` and give the
//! plugin a same-named function `fn(args: &[u8]) -> Result<Vec<u8>, String>`;
//! it calls the host primitives through the guest bridge itself. This keeps the
//! *logic* in the plugin while the host keeps the *protocol*.
//!
//! ```ignore
//! wit_bindgen::generate!({ path: "wit/plugin.wit", world: "plugin-world" });
//!
//! use xcelerate_plugin::plugin;
//!
//! struct Browser;
//!
//! plugin! {
//!     guest = Browser,
//!     name = "browser",
//!     // Composition lives in `fn wait` / `fn find` in this crate.
//!     local = { wait, find },
//!     ops = {
//!         open => "goto",
//!         click,
//!         fill,
//!     },
//! }
//!
//! export!(Browser);
//! ```

/// Defines a WebAssembly plugin's `Guest` impl from an op table.
///
/// * `guest` - the plugin's type; it receives the generated `describe` and
///   `invoke`, so `export!(<guest>)` works.
/// * `name` - the plugin id advertised in `describe`, checked against
///   `plugin.json`.
/// * `bridge` - the host interface to forward ops to: `browser`, `app`, or
///   `core`.
/// * `local` - (optional) ops implemented by a same-named function in the
///   plugin crate. Each must be `fn(&[u8]) -> Result<Vec<u8>, String>`.
/// * `ops` - the forwarded op table. `click` forwards to the host verb `click`;
///   `open => "goto"` forwards to `goto`. The method name is the op name.
#[macro_export]
macro_rules! plugin {
    (
        guest = $guest:ty,
        name = $name:literal,
        bridge = $bridge:ident,
        $( local = { $( $local:ident ),* $(,)? }, )?
        ops = { $( $op:ident $(=> $verb:literal)? ),* $(,)? } $(,)?
    ) => {
        impl exports::xcelerate::plugin::plugin::Guest for $guest {
            fn describe() -> ::std::vec::Vec<u8> {
                #[derive(::serde::Serialize)]
                struct __XclDescribe<'a> {
                    name: &'a str,
                    version: &'a str,
                    ops: ::std::vec::Vec<&'a str>,
                }
                let mut ops = ::std::vec![ $( stringify!($op) ),* ];
                $( $( ops.push(stringify!($local)); )* )?
                let describe = __XclDescribe {
                    name: $name,
                    version: env!("CARGO_PKG_VERSION"),
                    ops,
                };
                ::rmp_serde::to_vec_named(&describe).unwrap_or_default()
            }

            fn invoke(
                op: ::std::string::String,
                args: ::std::vec::Vec<u8>,
            ) -> ::core::result::Result<::std::vec::Vec<u8>, ::std::string::String> {
                $(
                    if op == stringify!($op) {
                        return xcelerate::plugin::host::$bridge(
                            $crate::__plugin_verb!($op $(=> $verb)?),
                            &args,
                        );
                    }
                )*
                $(
                    $(
                        if op == stringify!($local) {
                            return $local(&args);
                        }
                    )*
                )?
                ::core::result::Result::Err(
                    format!("{}: unknown op '{}'", $name, op),
                )
            }
        }
    };
}

/// Resolves an op's host verb: the op name, unless a `=> "verb"` override is
/// given. An implementation detail of [`plugin!`].
#[doc(hidden)]
#[macro_export]
macro_rules! __plugin_verb {
    ($op:ident) => {
        stringify!($op)
    };
    ($op:ident => $verb:literal) => {
        $verb
    };
}
