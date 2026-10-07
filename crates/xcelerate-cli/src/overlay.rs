//! The in-page HUD: a visible cursor, a bottom-center control bar, and an
//! interceptor panel that streams both the agent's steps and the human's own
//! interactions (clicks, typing, scrolling, navigation).
//!
//! The script is embedded in the binary so `--hud` needs no external file, and
//! it is injected on every document so it survives navigation.

use std::sync::Arc;

use xcelerate::{Page, XcelerateResult};

/// The HUD source (cursor + control bar), injected on every document.
pub const HUD_JS: &str = include_str!("assets/hud.js");

/// Installs the HUD on the current page and every future document.
pub async fn install(page: &Arc<Page>) -> XcelerateResult<()> {
    page.add_script_to_evaluate_on_new_document(HUD_JS.to_string())
        .await?;
    // Also run it against the page that is already loaded.
    let _ = page.evaluate_json(HUD_JS.to_string()).await;
    Ok(())
}

/// Removes the HUD from the current page.
pub async fn remove(page: &Arc<Page>) {
    let _ = page
        .evaluate_json(
            "(()=>{const h=document.getElementById('__xc_host');if(h)h.remove();window.__xcelerateHud=false;return true})()".to_string(),
        )
        .await;
}
