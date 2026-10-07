//! The in-page cursor: a small translucent dot that marks where the agent acts.
//!
//! This is the one piece of the old in-page HUD we kept. It is injected only
//! when the run asks for the overlay (`--hud` / `--ai`) and otherwise inert; the
//! Rust side just registers the script and flips a driving flag around mouse
//! steps so the page only reacts to the agent's own input.

use std::sync::Arc;

use xcelerate::{Page, XcelerateResult};

/// The cursor overlay source, compiled into the binary.
pub const CURSOR_JS: &str = include_str!("assets/cursor.js");

/// Injects the cursor into the current page and every new document.
pub async fn install(page: &Arc<Page>) -> XcelerateResult<()> {
    page.add_script_to_evaluate_on_new_document(CURSOR_JS.to_string())
        .await?;
    // Also run it against the page that is already loaded.
    let _ = page.evaluate_json(CURSOR_JS.to_string()).await;
    Ok(())
}

/// Marks whether pointer events should drive the cursor, and mirrors the input
/// gate: while a step drives the page the gate is lowered so the AI's own CDP
/// input reaches it, and it is raised again as the step ends.
///
/// Best effort: a page without the cursor installed simply ignores the flags.
pub async fn set_driving(page: &Arc<Page>, on: bool) {
    let js = format!(
        "(()=>{{window.__xcelerateDriving={on};if(window.__xcelerateGate)window.__xcelerateGate(!{on});return true}})()"
    );
    let _ = page.evaluate_json(js).await;
}

/// Raises or lowers the input gate on its own. While raised, the page swallows
/// pointer and keyboard events so a human cannot interfere mid-run.
pub async fn set_gate(page: &Arc<Page>, on: bool) {
    let js =
        format!("(()=>{{if(window.__xcelerateGate)window.__xcelerateGate({on});return true}})()");
    let _ = page.evaluate_json(js).await;
}

/// Removes the cursor host and clears the installed flag.
#[allow(dead_code)] // Part of the cursor API; the session keeps the dot installed.
pub async fn remove(page: &Arc<Page>) {
    let js = "(()=>{const h=document.getElementById('__xc_cursor_host');if(h&&h.parentNode)h.parentNode.removeChild(h);window.__xcelerateCursor=false;return true})()";
    let _ = page.evaluate_json(js.to_string()).await;
}
