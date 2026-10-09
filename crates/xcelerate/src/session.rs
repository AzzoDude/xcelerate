//! Session bootstrap for script runners.
//!
//! Running an XCL script needs a browser, but a *runner* should not carry the
//! browser's flags - the browser is a host/plugin concern. This module launches
//! (or attaches to) a browser from **environment** configuration and opens the
//! first page, so a runner (the CLI, or any embedder) stays browser-agnostic.
//!
//! | Variable | Meaning |
//! | --- | --- |
//! | `XCELERATE_BROWSER` | a known browser id or an executable path |
//! | `XCELERATE_EXECUTABLE` | an explicit executable path |
//! | `XCELERATE_HEADLESS` | `1`/`true` to run without a window |
//! | `XCELERATE_DETACHED` | `1`/`true` to detach the browser process |
//! | `XCELERATE_CONNECT` | attach to a running browser's CDP websocket |
//! | `XCELERATE_ATTACH` | attach to an existing target id |
//! | `XCELERATE_PLUGINS` | comma-separated plugin paths to load at launch |

use std::sync::Arc;

use crate::{Browser, BrowserConfig, Page, XcelerateResult};

/// A boolean environment flag: `1`, `true`, or `yes` (case-insensitive).
fn env_flag(key: &str) -> bool {
    matches!(
        std::env::var(key)
            .ok()
            .map(|value| value.trim().to_ascii_lowercase())
            .as_deref(),
        Some("1") | Some("true") | Some("yes")
    )
}

/// A non-empty environment value.
fn env_value(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .filter(|value| !value.trim().is_empty())
}

/// Launches (or attaches to) a browser from the environment and opens `url`,
/// returning the browser and its first page.
pub async fn launch(url: &str) -> XcelerateResult<(Arc<Browser>, Arc<Page>)> {
    let browser = match env_value("XCELERATE_CONNECT") {
        Some(ws_url) => Browser::connect(ws_url).await?,
        None => {
            let executable_path =
                env_value("XCELERATE_EXECUTABLE").or_else(|| env_value("XCELERATE_BROWSER"));
            let plugins = env_value("XCELERATE_PLUGINS").map(|list| {
                list.split(',')
                    .map(|entry| entry.trim().to_string())
                    .filter(|entry| !entry.is_empty())
                    .collect::<Vec<_>>()
            });
            let config = BrowserConfig {
                headless: env_flag("XCELERATE_HEADLESS"),
                detached: env_flag("XCELERATE_DETACHED"),
                executable_path,
                plugins,
            };
            Browser::launch(config).await?
        }
    };
    let page = match env_value("XCELERATE_ATTACH") {
        Some(target_id) => Arc::clone(&browser).attach_page(target_id).await?,
        None => Arc::clone(&browser).new_page(url.to_string()).await?,
    };
    Ok((browser, page))
}
