//! Element highlight overlays (capability #6).
//!
//! Implemented as inherent `impl Page` methods in this module so they stay out
//! of the `#[uniffi::export]` block and binding checksums remain stable.

use crate::element::JS_QUERY_ALL;
use crate::error::{XcelerateError, XcelerateResult};
use crate::page::Page;

/// `id` of the stylesheet injected by [`Page::highlight_all`].
pub(crate) const HIGHLIGHT_STYLE_ID: &str = "xcelerate-highlight";

/// Selectors treated as "interactive" and outlined by [`Page::highlight_all`].
pub(crate) const HIGHLIGHT_SELECTOR: &str =
    "a, button, input, select, textarea, [role=button], [onclick]";

/// Outline applied to highlighted elements.
const HIGHLIGHT_OUTLINE: &str = "2px solid #ff3b30";

/// Background tint applied to individually highlighted elements.
const HIGHLIGHT_BACKGROUND: &str = "rgba(255, 59, 48, 0.15)";

/// Marker attribute set on elements that carry an individual highlight.
pub(crate) const HIGHLIGHT_ATTR: &str = "data-xcelerate-highlighted";

/// Remembers an element's inline `outline` before it was highlighted.
const PREV_OUTLINE_ATTR: &str = "data-xcelerate-prev-outline";

/// Remembers an element's inline `background-color` before it was highlighted.
const PREV_BACKGROUND_ATTR: &str = "data-xcelerate-prev-bg";

/// Object group used by `DOM.resolveNode` when resolving snapshot indices.
const HIGHLIGHT_OBJECT_GROUP: &str = "xcelerate.highlight";

/// Builds the stylesheet rule injected by [`Page::highlight_all`].
pub(crate) fn highlight_css(selector: &str) -> String {
    format!("{selector} {{ outline: {HIGHLIGHT_OUTLINE} !important; }}")
}

/// A JS function expression that applies the highlight overlay to `this`.
///
/// The element's previous inline `outline`/`background-color` are remembered on
/// data attributes so [`Page::clear_highlights`] can restore them exactly.
pub(crate) fn apply_highlight_fn() -> String {
    format!(
        "function(){{const el=this;if(!el.hasAttribute('{HIGHLIGHT_ATTR}')){{el.setAttribute('{PREV_OUTLINE_ATTR}',el.style.outline||'');el.setAttribute('{PREV_BACKGROUND_ATTR}',el.style.backgroundColor||'');el.setAttribute('{HIGHLIGHT_ATTR}','');}}el.style.setProperty('outline','{HIGHLIGHT_OUTLINE}','important');el.style.setProperty('background-color','{HIGHLIGHT_BACKGROUND}','important');return true;}}"
    )
}

/// A JS function expression that removes the overlay from `this`; the inverse
/// of [`apply_highlight_fn`].
fn clear_element_fn() -> String {
    format!(
        "function(){{const el=this;if(el.hasAttribute('{HIGHLIGHT_ATTR}')){{el.style.removeProperty('outline');el.style.removeProperty('background-color');const po=el.getAttribute('{PREV_OUTLINE_ATTR}');const pb=el.getAttribute('{PREV_BACKGROUND_ATTR}');if(po)el.style.outline=po;if(pb)el.style.backgroundColor=pb;el.removeAttribute('{PREV_OUTLINE_ATTR}');el.removeAttribute('{PREV_BACKGROUND_ATTR}');el.removeAttribute('{HIGHLIGHT_ATTR}');}}return true;}}"
    )
}

/// Parses a CDP JSON number into a count, defaulting to `0`.
fn parse_u32(raw: &str) -> u32 {
    raw.trim().parse::<u32>().unwrap_or(0)
}

impl Page {
    /// Outlines every interactive element and returns how many matched.
    ///
    /// Injects a single `<style id="xcelerate-highlight">` element (reused on
    /// repeat calls) whose rule targets `a, button, input, select, textarea,
    /// [role=button], [onclick]` with a `2px solid #ff3b30` outline. Because a
    /// document stylesheet does not apply across shadow boundaries, the same
    /// rule is also injected into every open shadow root. Matching (and thus the
    /// returned count) pierces open shadow roots too.
    pub async fn highlight_all(&self) -> XcelerateResult<u32> {
        let id_lit = serde_json::to_string(HIGHLIGHT_STYLE_ID)
            .map_err(|e| XcelerateError::SerdeError(e.to_string()))?;
        let sel_lit = serde_json::to_string(HIGHLIGHT_SELECTOR)
            .map_err(|e| XcelerateError::SerdeError(e.to_string()))?;
        let css_lit = serde_json::to_string(&highlight_css(HIGHLIGHT_SELECTOR))
            .map_err(|e| XcelerateError::SerdeError(e.to_string()))?;
        let script = format!(
            "(function(){{const id={id_lit};const sel={sel_lit};const css={css_lit};let s=document.getElementById(id);if(!s){{s=document.createElement('style');s.id=id;(document.head||document.documentElement).appendChild(s);}}s.textContent=css;const hosts=({JS_QUERY_ALL}).call(document,'*');for(let i=0;i<hosts.length;i++){{const root=hosts[i].shadowRoot;if(!root)continue;let ss=root.getElementById(id);if(!ss){{ss=document.createElement('style');ss.id=id;root.appendChild(ss);}}ss.textContent=css;}}return ({JS_QUERY_ALL}).call(document,sel).length;}})()"
        );
        let raw = self.evaluate_json(script).await?;
        Ok(parse_u32(&raw))
    }

    /// Highlights the element that carried `index` in the most recent snapshot.
    ///
    /// Resolves the snapshot `[index]` to a live node via `DOM.resolveNode` and
    /// then applies a visible outline plus background tint with
    /// `Runtime.callFunctionOn`.
    pub async fn highlight_index(&self, index: u32) -> XcelerateResult<()> {
        let backend_node_id = {
            let cache = self.snapshot_index.lock().await;
            cache.get(&index).copied()
        }
        .ok_or_else(|| {
            XcelerateError::NotFound(format!(
                "snapshot index {index} is unknown; call agent_snapshot first"
            ))
        })?;

        let resolved = self
            .client
            .execute_raw_with_session(
                Some(&self.session_id),
                "DOM.resolveNode",
                serde_json::json!({
                    "backendNodeId": backend_node_id,
                    "objectGroup": HIGHLIGHT_OBJECT_GROUP,
                }),
            )
            .await?;
        let object_id = resolved
            .pointer("/object/objectId")
            .and_then(serde_json::Value::as_str)
            .ok_or_else(|| {
                XcelerateError::NotFound(format!(
                    "could not resolve a live node for snapshot index {index}"
                ))
            })?;

        self.client
            .execute_raw_with_session(
                Some(&self.session_id),
                "Runtime.callFunctionOn",
                serde_json::json!({
                    "objectId": object_id,
                    "functionDeclaration": apply_highlight_fn(),
                    "returnByValue": true,
                }),
            )
            .await?;
        Ok(())
    }

    /// Removes the injected stylesheet and clears every individual highlight.
    ///
    /// Both the document stylesheet and the per-shadow-root copies added by
    /// [`Page::highlight_all`] are removed, and highlights inside open shadow
    /// roots are cleared as well.
    pub async fn clear_highlights(&self) -> XcelerateResult<()> {
        let id_lit = serde_json::to_string(HIGHLIGHT_STYLE_ID)
            .map_err(|e| XcelerateError::SerdeError(e.to_string()))?;
        let clear_fn = clear_element_fn();
        let script = format!(
            "(function(){{const id={id_lit};const s=document.getElementById(id);if(s)s.remove();const hosts=({JS_QUERY_ALL}).call(document,'*');for(let i=0;i<hosts.length;i++){{const root=hosts[i].shadowRoot;if(!root)continue;const ss=root.getElementById(id);if(ss)ss.remove();}}const els=({JS_QUERY_ALL}).call(document,'[{HIGHLIGHT_ATTR}]');for(let i=0;i<els.length;i++){{({clear_fn}).call(els[i]);}}return true;}})()"
        );
        self.evaluate_json(script).await?;

        // Drop the node handles resolved for `highlight_index`.
        let _ = self
            .client
            .execute_raw_with_session(
                Some(&self.session_id),
                "Runtime.releaseObjectGroup",
                serde_json::json!({ "objectGroup": HIGHLIGHT_OBJECT_GROUP }),
            )
            .await;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn highlight_css_targets_interactive_selectors() {
        let css = highlight_css(HIGHLIGHT_SELECTOR);
        assert!(css.contains("outline: 2px solid #ff3b30"));
        assert!(css.contains("[role=button]"));
        assert!(css.contains("[onclick]"));
    }

    #[test]
    fn apply_and_clear_functions_share_tracking_attribute() {
        assert!(apply_highlight_fn().contains(HIGHLIGHT_ATTR));
        assert!(clear_element_fn().contains(HIGHLIGHT_ATTR));
    }

    #[test]
    fn parse_u32_handles_numbers_and_garbage() {
        assert_eq!(parse_u32("12"), 12);
        assert_eq!(parse_u32(" 3 "), 3);
        assert_eq!(parse_u32("null"), 0);
        assert_eq!(parse_u32(""), 0);
    }
}
