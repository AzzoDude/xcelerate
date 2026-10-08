//! Shared element-picking rules for `click` and `tap`.
//!
//! Both the interactive session and the XCL interpreter resolve a click target
//! the same way: a bare integer is a snapshot index, a string that reads as a CSS
//! selector is treated as one, and anything else is matched against visible text
//! (or `aria-label`). Keeping the rules here means the two front ends can never
//! drift apart.

use std::sync::Arc;

use xcelerate::{Element, Page};

/// Whether `value` reads as a CSS selector rather than visible text.
pub fn looks_like_a_selector(value: &str) -> bool {
    let value = value.trim();
    value.starts_with(['#', '.', '['])
        || value.starts_with("//")
        || (value.contains('[') && value.contains(']'))
}

/// The controls `click` / `tap` match by visible text. Includes listbox/combobox
/// controls and their options, so a dropdown can be opened and chosen by text
/// (`click "Select day"`, then `click "14"`).
const TEXT_CONTROLS_JS: &str = "const els=[...document.querySelectorAll('a,button,summary,input[type=\"submit\"],[role=\"button\"],[role=\"link\"],[role=\"combobox\"],[role=\"listbox\"],[role=\"option\"],[role=\"menuitem\"],[role=\"menuitemcheckbox\"],[role=\"menuitemradio\"],[role=\"tab\"],[role=\"treeitem\"],[role=\"radio\"],[role=\"checkbox\"],[role=\"switch\"]')];";

/// Finds the first control whose visible text or `aria-label` contains `text`.
pub async fn control_by_text(
    page: &Arc<Page>,
    text: &str,
) -> Result<Option<Arc<Element>>, Box<dyn std::error::Error>> {
    let needle = serde_json::to_string(text)?;
    let expr = format!(
        "els.find(e=>e.offsetParent!==null&&(((e.innerText||'')+' '+(e.getAttribute('aria-label')||'')).toLowerCase().includes({needle}.toLowerCase())))"
    );
    if page
        .evaluate_bool(format!(
            "(() => {{ {TEXT_CONTROLS_JS} return !!({expr}); }})()"
        ))
        .await?
    {
        Ok(Some(
            Arc::clone(page)
                .evaluate_handle(format!("(() => {{ {TEXT_CONTROLS_JS} return {expr}; }})()"))
                .await?,
        ))
    } else {
        Ok(None)
    }
}
