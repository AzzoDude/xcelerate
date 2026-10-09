//! Shared element-picking rules for `click` and `tap`.
//!
//! Both the interactive session and the XCL interpreter resolve a click target
//! the same way: a bare integer is a snapshot index, a string that reads as a CSS
//! selector is treated as one, and anything else is matched against visible text
//! (or `aria-label`). Keeping the rules here means the two front ends can never
//! drift apart.

use std::sync::Arc;

use xcelerate::{Element, Page};

/// Whether `value` reads as a selector rather than visible text.
pub fn looks_like_a_selector(value: &str) -> bool {
    let value = value.trim();
    value.starts_with(['#', '.', '['])
        || value.starts_with("//")
        || value.starts_with("xpath=")
        || value.starts_with("role=")
        || value.starts_with("label=")
        || value.starts_with("text=")
        || (value.contains('[') && value.contains(']'))
}

/// Finds the first visible, clickable control whose text (or `aria-label`)
/// matches `text`.
///
/// Delegates to the page's shared matcher so `click`/`tap`/`hover`/`find` all
/// agree on the same element: NFC-normalized, `aria-label`-aware,
/// visibility-checked, and piercing open shadow roots and same-origin frames.
pub async fn control_by_text(
    page: &Arc<Page>,
    text: &str,
) -> Result<Option<Arc<Element>>, Box<dyn std::error::Error>> {
    Ok(Arc::clone(page)
        .find_clickable_by_text(text.to_string())
        .await?)
}
