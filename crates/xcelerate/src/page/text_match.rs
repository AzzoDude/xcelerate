//! Shared text matching for the agent verbs (capabilities #2 and #6).
//!
//! `find`, `click`, `tap` and `hover` all resolve a target the same way: the
//! needle and every candidate are Unicode-normalized to NFC before comparison,
//! an element matches on its composed/nested text plus `aria-label`/`value`, and
//! only visible elements match. The walk pierces open shadow roots and
//! same-origin iframe documents, exactly like the snapshot/frame queries.
//!
//! Implemented as inherent `impl Page` methods in this module so they stay out of
//! the `#[uniffi::export]` block and binding checksums remain stable.

use std::sync::Arc;

use crate::element::Element;
use crate::error::{XcelerateError, XcelerateResult};
use crate::page::Page;

/// Shared JS helpers: composed-tree walk (shadow roots + same-origin frames),
/// visibility, NFC-normalized haystack, and the match set.
pub(crate) const JS_TEXT_HELPERS: &str = r#"
const __xclComposedParent = (n) => { if (!n) return null; if (n.parentNode) return n.parentNode; const r = n.getRootNode ? n.getRootNode() : null; return (r && r.host) ? r.host : null; };
const __xclVisible = (el) => { if (!el || !el.getBoundingClientRect) return false; const r = el.getBoundingClientRect(); if (r.width <= 0 || r.height <= 0) return false; const s = getComputedStyle(el); return s.display !== 'none' && s.visibility !== 'hidden' && s.opacity !== '0'; };
const __xclHaystack = (el) => { const parts = []; if (el.getAttribute) { const a = el.getAttribute('aria-label'); if (a) parts.push(a); } if (typeof el.value === 'string' && el.value) parts.push(el.value); parts.push(el.textContent || ''); return parts.join(' ').normalize('NFC').toLowerCase(); };
const __xclHits = (root, needle) => { const target = (needle || '').normalize('NFC').toLowerCase(); if (!target) return []; const seen = new Set(); const hits = []; const push = (el) => { if (!el || seen.has(el)) return; seen.add(el); if (__xclVisible(el)) hits.push(el); }; const visit = (scope) => { if (!scope || !scope.querySelectorAll) return; const walker = document.createTreeWalker(scope, NodeFilter.SHOW_TEXT, { acceptNode: (n) => { const p = n.parentElement; if (!p) return NodeFilter.FILTER_REJECT; const t = p.tagName ? p.tagName.toLowerCase() : ''; if (t === 'script' || t === 'style' || t === 'noscript' || t === 'template') return NodeFilter.FILTER_REJECT; if (!n.nodeValue) return NodeFilter.FILTER_SKIP; return NodeFilter.FILTER_ACCEPT; } }); let node; while ((node = walker.nextNode())) { if ((node.nodeValue || '').normalize('NFC').toLowerCase().indexOf(target) !== -1) push(node.parentElement); } for (const el of scope.querySelectorAll('[aria-label],input,textarea,select,option,img[alt],area[alt]')) { if (__xclHaystack(el).indexOf(target) !== -1) push(el); } for (const el of scope.querySelectorAll('*')) { if (el.shadowRoot) visit(el.shadowRoot); let d = null; try { d = el.contentDocument; } catch (e) { d = null; } if (d) visit(d); } }; visit(root); return hits; };
const __xclLeaves = (hits) => hits.filter(el => !hits.some(o => o !== el && el.contains(o)));
"#;

/// Shared JS helpers identifying elements that can be clicked.
pub(crate) const JS_ACTABLE_HELPERS: &str = r#"
const __xclActable = (el) => { const tag = (el.tagName || '').toLowerCase(); if (['a','button','summary','input','select','textarea','label','option'].indexOf(tag) !== -1) return true; const role = ((el.getAttribute && el.getAttribute('role')) || '').toLowerCase(); if (['button','link','checkbox','radio','switch','tab','menuitem','menuitemcheckbox','menuitemradio','option','combobox','listbox','treeitem','textbox','searchbox'].indexOf(role) !== -1) return true; if (el.onclick || (el.getAttribute && el.getAttribute('onclick'))) return true; try { if (getComputedStyle(el).cursor === 'pointer') return true; } catch (e) {} return false; };
const __xclActableAncestor = (el) => { let n = el, g = 0; while (n && g++ < 30) { if (__xclActable(n)) return n; n = __xclComposedParent(n); } return null; };
"#;

/// Builds the JS expression that returns the first clickable element whose text
/// matches `needle`, or `null`.
fn clickable_expression(needle: &str) -> XcelerateResult<String> {
    let needle_lit =
        serde_json::to_string(needle).map_err(|e| XcelerateError::SerdeError(e.to_string()))?;
    Ok(String::new()
        + "(function(){"
        + JS_TEXT_HELPERS
        + JS_ACTABLE_HELPERS
        + "const hits=__xclHits(document.documentElement,"
        + &needle_lit
        + ");const ordered=__xclLeaves(hits).concat(hits);for(const h of ordered){const a=__xclActableAncestor(h);if(a)return a;}return null;})()")
}

/// Builds the JS expression that returns the first element (deepest match first)
/// whose text matches `needle`, or `null`.
fn text_expression(needle: &str) -> XcelerateResult<String> {
    let needle_lit =
        serde_json::to_string(needle).map_err(|e| XcelerateError::SerdeError(e.to_string()))?;
    Ok(String::new()
        + "(function(){"
        + JS_TEXT_HELPERS
        + "const hits=__xclHits(document.documentElement,"
        + &needle_lit
        + ");const leaves=__xclLeaves(hits);return leaves[0] || hits[0] || null;})()")
}

/// Turns an `evaluate_handle` result into an `Option`, treating a `null` result
/// (`NotFound`) as "no match" while propagating real errors.
async fn handle_or_none(
    page: &Arc<Page>,
    expression: String,
) -> XcelerateResult<Option<Arc<Element>>> {
    match Arc::clone(page).evaluate_handle(expression).await {
        Ok(element) => Ok(Some(element)),
        Err(XcelerateError::NotFound(_)) => Ok(None),
        Err(error) => Err(error),
    }
}

impl Page {
    /// Finds the first visible element (deepest match first) whose text matches
    /// `text`, using the shared pierce-shadows/frames matcher.
    pub async fn find_text_element(
        self: Arc<Self>,
        text: String,
    ) -> XcelerateResult<Option<Arc<Element>>> {
        handle_or_none(&self, text_expression(&text)?).await
    }

    /// Finds the first visible, clickable element whose text (or `aria-label`)
    /// matches `text`. Returns `None` when nothing matches, so callers can report
    /// a miss instead of acting on the wrong element.
    pub async fn find_clickable_by_text(
        self: Arc<Self>,
        text: String,
    ) -> XcelerateResult<Option<Arc<Element>>> {
        handle_or_none(&self, clickable_expression(&text)?).await
    }

    /// Clicks the first clickable element whose text matches `text`.
    ///
    /// Fails with [`XcelerateError::NotFound`] when nothing matches, so a miss is
    /// never reported as a successful click.
    pub async fn click_text(self: Arc<Self>, text: String) -> XcelerateResult<Arc<Self>> {
        let element = self
            .clone()
            .find_clickable_by_text(text.clone())
            .await?
            .ok_or_else(|| {
                XcelerateError::NotFound(format!("no clickable element matches text: {text:?}"))
            })?;
        element.click_mouse().await?;
        Ok(self)
    }

    /// Performs a DOM click (no mouse movement) on the first clickable element
    /// whose text matches `text`.
    pub async fn tap_text(self: Arc<Self>, text: String) -> XcelerateResult<Arc<Self>> {
        let element = self
            .clone()
            .find_clickable_by_text(text.clone())
            .await?
            .ok_or_else(|| {
                XcelerateError::NotFound(format!("no clickable element matches text: {text:?}"))
            })?;
        element.click().await?;
        Ok(self)
    }

    /// Moves the mouse over the first clickable element whose text matches
    /// `text`.
    pub async fn hover_text(self: Arc<Self>, text: String) -> XcelerateResult<Arc<Self>> {
        let element = self
            .clone()
            .find_clickable_by_text(text.clone())
            .await?
            .ok_or_else(|| {
                XcelerateError::NotFound(format!("no clickable element matches text: {text:?}"))
            })?;
        element.hover_mouse().await?;
        Ok(self)
    }
}

/// Counts, scrolls to, and highlights the first element whose visible text
/// contains `text`, using the shared matcher. Returns `1` on a hit, else `0`.
pub(crate) async fn find_text_scroll_highlight(page: &Page, text: String) -> XcelerateResult<u32> {
    if text.is_empty() {
        return Ok(0);
    }
    let needle_lit =
        serde_json::to_string(&text).map_err(|e| XcelerateError::SerdeError(e.to_string()))?;
    let script = String::new()
        + "(function(){"
        + JS_TEXT_HELPERS
        + "const hits=__xclHits(document.documentElement,"
        + &needle_lit
        + ");if(!hits.length)return 0;const leaves=__xclLeaves(hits);const node=leaves[0]||hits[0];node.scrollIntoView({block:'center',inline:'center'});("
        + &crate::page::highlight::apply_highlight_fn()
        + ").call(node);return 1;})()";
    let raw = page.evaluate_json(script).await?;
    Ok(raw.trim().parse::<u32>().unwrap_or(0))
}

/// Counts the distinct text locations matching `text` (innermost elements only).
pub(crate) async fn count_text_matches(page: &Page, text: String) -> XcelerateResult<u32> {
    if text.is_empty() {
        return Ok(0);
    }
    let needle_lit =
        serde_json::to_string(&text).map_err(|e| XcelerateError::SerdeError(e.to_string()))?;
    let script = String::new()
        + "(function(){"
        + JS_TEXT_HELPERS
        + "const hits=__xclHits(document.documentElement,"
        + &needle_lit
        + ");return __xclLeaves(hits).length;})()";
    let raw = page.evaluate_json(script).await?;
    Ok(raw.trim().parse::<u32>().unwrap_or(0))
}
