//! Popup / new-tab ergonomics (capability #8).
//!
//! Implemented as inherent `impl Page` methods in this module so they stay out
//! of the `#[uniffi::export]` block and binding checksums remain stable.

use tokio::sync::broadcast::error::RecvError;

use crate::error::{XcelerateError, XcelerateResult};
use crate::page::Page;

/// Resolves the caller-supplied timeout, falling back to the page default when
/// it is `0`. A resolved value of `u64::MAX` means "no timeout".
fn effective_timeout_ms(page: &Page, timeout_ms: u64) -> u64 {
    if timeout_ms == 0 {
        page.default_timeout()
    } else {
        timeout_ms
    }
}

/// Runs `fut`, failing with a `NotFound` error after `budget` ms. A budget of
/// `u64::MAX` means "no timeout" and waits indefinitely.
async fn with_deadline<F, T>(budget: u64, fut: F) -> XcelerateResult<T>
where
    F: std::future::Future<Output = XcelerateResult<T>>,
{
    if budget == u64::MAX {
        return fut.await;
    }
    match tokio::time::timeout(std::time::Duration::from_millis(budget), fut).await {
        Ok(result) => result,
        Err(_) => Err(XcelerateError::NotFound("popup timeout".into())),
    }
}

/// Which field of the newly created target to surface.
#[derive(Clone, Copy)]
enum PopupField {
    TargetId,
    Url,
}

/// Whether `target_info` describes a `page` target other than `current`.
pub(crate) fn is_new_page_target(target_info: &serde_json::Value, current: &str) -> bool {
    let kind = target_info.get("type").and_then(|v| v.as_str());
    let target_id = target_info
        .get("targetId")
        .and_then(|v| v.as_str())
        .unwrap_or_default();
    kind == Some("page") && !target_id.is_empty() && target_id != current
}

impl Page {
    /// Waits for a new page target (popup or tab) and returns its `targetId`.
    ///
    /// Best-effort enables `Target.setDiscoverTargets`, then watches for a
    /// `Target.targetCreated` event describing a `page` target other than this
    /// page. The wait is bounded by `timeout_ms` (`0` falls back to the page
    /// default) and fails with [`XcelerateError::NotFound`] on timeout.
    pub async fn wait_for_popup(&self, timeout_ms: u64) -> XcelerateResult<String> {
        self.wait_for_popup_target(timeout_ms, PopupField::TargetId)
            .await
    }

    /// Waits for a new page target (popup or tab) and returns its URL.
    ///
    /// Identical to [`Page::wait_for_popup`] but returns the new target's URL
    /// instead of its `targetId`.
    pub async fn wait_for_popup_url(&self, timeout_ms: u64) -> XcelerateResult<String> {
        self.wait_for_popup_target(timeout_ms, PopupField::Url)
            .await
    }

    /// Shared implementation for the popup waits.
    async fn wait_for_popup_target(
        &self,
        timeout_ms: u64,
        field: PopupField,
    ) -> XcelerateResult<String> {
        let budget = effective_timeout_ms(self, timeout_ms);
        // Discovery is best effort: if it fails we still watch for events that
        // arrive over the existing connection.
        let _ = self
            .client
            .execute_raw(
                "Target.setDiscoverTargets",
                serde_json::json!({ "discover": true }),
            )
            .await;
        let mut receiver = self.client.subscribe_session(&self.session_id);
        let current = self.target_id.clone();

        with_deadline(budget, async {
            loop {
                let value = match receiver.recv().await {
                    Ok(value) => value,
                    Err(RecvError::Lagged(_)) => continue,
                    Err(RecvError::Closed) => return Err(XcelerateError::InternalError),
                };
                if value.get("method").and_then(|m| m.as_str()) != Some("Target.targetCreated") {
                    continue;
                }
                let info = value
                    .get("params")
                    .and_then(|p| p.get("targetInfo"))
                    .cloned()
                    .unwrap_or(serde_json::Value::Null);
                if !is_new_page_target(&info, &current) {
                    continue;
                }
                let result = match field {
                    PopupField::TargetId => info.get("targetId"),
                    PopupField::Url => info.get("url"),
                };
                return Ok(result
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string());
            }
        })
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::is_new_page_target;
    use serde_json::json;

    #[test]
    fn accepts_a_different_page_target() {
        let info = json!({ "type": "page", "targetId": "abc" });
        assert!(is_new_page_target(&info, "self"));
    }

    #[test]
    fn rejects_the_current_target() {
        let info = json!({ "type": "page", "targetId": "self" });
        assert!(!is_new_page_target(&info, "self"));
    }

    #[test]
    fn rejects_non_page_targets() {
        let info = json!({ "type": "iframe", "targetId": "abc" });
        assert!(!is_new_page_target(&info, "self"));
    }

    #[test]
    fn rejects_missing_type_or_id() {
        assert!(!is_new_page_target(&json!({ "targetId": "abc" }), "self"));
        assert!(!is_new_page_target(&json!({ "type": "page" }), "self"));
        assert!(!is_new_page_target(
            &json!({ "type": "page", "targetId": "" }),
            "self"
        ));
    }
}
