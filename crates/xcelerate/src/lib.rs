//! # xcelerate
//!
//! `xcelerate` is a high-performance, lightweight Chrome DevTools Protocol (CDP) client.
//! It provides a fluent, chained API for browser automation, designed for speed and reliability.
//!
//! The workspace is split into focused crates:
//!
//! * [`xcelerate_core`] — the WebSocket transport and typed command layer.
//! * [`xcelerate_plugin_api`] — the plugin trait, manifest, audit log, and the
//!   `PageHost` interface plugins use to reach a page.
//! * [`xcelerate_plugins`] — the plugins that ship with the engine (`stealth`,
//!   `human`).
//!
//! This crate is the thin facade: it composes those pieces into the high-level
//! [`Browser`], [`Page`], and [`Element`] API and exposes it to other languages
//! through `uniffi`.

pub mod browser;
pub mod devices;
pub mod element;
pub mod error;
pub mod page;
pub mod plugin;
pub mod policy;
pub mod process;
pub mod profile;

pub mod options;

mod proxy;

pub mod adapters;

pub use browser::firefox;
pub use browser::{Browser, BrowserConfig, configure_user_data_dir};
pub use devices::Device;
pub use element::Element;
pub use error::{XcelerateError, XcelerateResult};
pub use page::Page;
pub use page::recording::VideoOptions;
pub use plugin::{Capability, Manifest, Plugin, PluginHandle};

// Proxy configuration (Rust-only; other languages set `XCELERATE_PROXY[_POOL]`).
pub use proxy::configure as configure_proxy;

// Domain allow/deny policy (Rust-only; enforced before navigation).
pub use policy::configure_domain_policy;

// Launch options (Rust-only; applied by `Browser::launch`).
pub use options::{LaunchOptions, configure_launch_options, reset_launch_options};

// Re-export the transport layer and the generated protocol crates so downstream
// code can reach everything through `xcelerate`.
pub use xcelerate_core::{CdpClient, CdpCommand, CdpHandler, connect};
pub use xcelerate_core::{browser_protocol, js_protocol};

/// WebDriver BiDi bindings, re-exported for the Firefox backend and consumers.
pub use xcelerate_core::webdriver_bidi;

uniffi::setup_scaffolding!("xcelerate");
