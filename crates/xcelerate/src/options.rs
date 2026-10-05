//! Process-wide launch options (capability F).
//!
//! Rust-only, mirroring [`crate::configure_proxy`] and
//! [`crate::configure_domain_policy`], so the UniFFI `BrowserConfig` record -
//! and therefore every language binding - stays unchanged.

use std::sync::RwLock;

/// Extra launch behaviour applied by `Browser::launch`.
#[derive(Clone, Debug, Default)]
pub struct LaunchOptions {
    /// Additional command-line flags passed to the browser process, verbatim.
    pub extra_args: Vec<String>,
    /// Whether Chrome may download files (`Browser.setDownloadBehavior`).
    /// `None` leaves the browser default untouched.
    pub accept_downloads: Option<bool>,
    /// Enable Chrome's deterministic-rendering flags (stable screenshots).
    pub deterministic_rendering: bool,
    /// Disable web security / site isolation (testing only).
    pub disable_security: bool,
    /// Keep the browser process alive after the handle is dropped.
    pub keep_alive: bool,
}

static OPTIONS: RwLock<Option<LaunchOptions>> = RwLock::new(None);

/// Installs the process-wide launch options. Later launches use them.
pub fn configure_launch_options(options: LaunchOptions) {
    let mut guard = OPTIONS.write().unwrap_or_else(|poison| poison.into_inner());
    *guard = Some(options);
}

/// Clears any configured launch options (back to browser defaults).
pub fn reset_launch_options() {
    let mut guard = OPTIONS.write().unwrap_or_else(|poison| poison.into_inner());
    *guard = None;
}

/// The configured options, or `None` when the defaults should be used.
pub(crate) fn configured() -> Option<LaunchOptions> {
    OPTIONS
        .read()
        .unwrap_or_else(|poison| poison.into_inner())
        .clone()
}
