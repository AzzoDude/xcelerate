//! Browser health / crash monitoring (capability D).
//!
//! Implemented as inherent `impl Page` methods in this module so they stay out
//! of the `#[uniffi::export]` block and binding checksums remain stable.

use std::time::Duration;

use serde_json::json;

use crate::error::{XcelerateError, XcelerateResult};
use crate::page::Page;

/// Timeout used by [`Page::health`] when the page has no default timeout set.
const HEALTH_PROBE_TIMEOUT_MS: u64 = 5_000;

/// Whether a CDP event envelope reports a renderer/target crash.
///
/// Both the `Inspector` and `Target` domains announce crashes, and either may
/// arrive on the page session or the root session, so the method name alone is
/// what identifies them.
pub(crate) fn is_crash_event(event: &serde_json::Value) -> bool {
    matches!(
        event.get("method").and_then(|m| m.as_str()),
        Some("Inspector.targetCrashed") | Some("Target.targetCrashed")
    )
}

impl Page {
    /// Whether the page's JS context still answers within `timeout_ms`.
    ///
    /// Runs a trivial `Runtime.evaluate` and reports success only if the reply
    /// arrives in time. A `0` timeout is clamped to `1ms` so the probe always
    /// gets a chance to complete.
    pub async fn is_responsive(&self, timeout_ms: u64) -> bool {
        self.client
            .execute_raw_with_session_timeout(
                Some(&self.session_id),
                "Runtime.evaluate",
                json!({ "expression": "1", "returnByValue": true }),
                Duration::from_millis(timeout_ms.max(1)),
            )
            .await
            .is_ok()
    }

    /// Waits for a renderer/target crash, returning the crash event JSON.
    ///
    /// Enables `Inspector` and `Target` discovery best-effort first (errors are
    /// ignored), subscribes to CDP events, and resolves as soon as an
    /// `Inspector.targetCrashed` (from any session) or `Target.targetCrashed`
    /// event is seen. The whole event envelope is returned serialized. Returns
    /// [`XcelerateError::NotFound`] if nothing arrives within `timeout_ms`
    /// (a `0` timeout is clamped to `1ms`).
    pub async fn wait_for_crash(&self, timeout_ms: u64) -> XcelerateResult<String> {
        // Best effort: the domains that carry crash notifications may or may not
        // be supported, and we only need one of them to fire.
        let _ = self
            .client
            .execute_raw_with_session(Some(&self.session_id), "Inspector.enable", json!({}))
            .await;
        let _ = self
            .client
            .execute_raw_with_session(
                None,
                "Target.setDiscoverTargets",
                json!({ "discover": true }),
            )
            .await;

        let mut receiver = self.client.subscribe();
        let timeout = Duration::from_millis(timeout_ms.max(1));
        let start = std::time::Instant::now();
        loop {
            let remaining = timeout.saturating_sub(start.elapsed());
            if remaining.is_zero() {
                return Err(XcelerateError::NotFound(
                    "Timeout waiting for crash".to_string(),
                ));
            }
            match tokio::time::timeout(remaining, receiver.recv()).await {
                Ok(Ok(value)) => {
                    if is_crash_event(&value) {
                        return Ok(value.to_string());
                    }
                }
                // A slow consumer only missed events; keep waiting.
                Ok(Err(tokio::sync::broadcast::error::RecvError::Lagged(_))) => continue,
                Ok(Err(tokio::sync::broadcast::error::RecvError::Closed)) => break,
                Err(_) => {
                    return Err(XcelerateError::NotFound(
                        "Timeout waiting for crash".to_string(),
                    ));
                }
            }
        }

        Err(XcelerateError::NotFound(
            "Event stream closed while waiting for crash".to_string(),
        ))
    }

    /// A JSON summary: `{ responsive, url, targetId }`.
    ///
    /// Probes responsiveness via [`Page::is_responsive`] using the page's
    /// default timeout (falling back to 5 seconds when none is set), then
    /// reports the current URL and target id.
    pub async fn health(&self) -> XcelerateResult<String> {
        let probe_ms = match self.default_timeout() {
            u64::MAX => HEALTH_PROBE_TIMEOUT_MS,
            millis => millis,
        };
        let responsive = self.is_responsive(probe_ms).await;
        let url = self.url().await.unwrap_or_default();
        Ok(json!({
            "responsive": responsive,
            "url": url,
            "targetId": self.target_id,
        })
        .to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::is_crash_event;
    use serde_json::json;

    #[test]
    fn recognizes_inspector_crash() {
        assert!(is_crash_event(&json!({
            "method": "Inspector.targetCrashed",
            "params": {},
            "sessionId": "abc",
        })));
    }

    #[test]
    fn recognizes_target_crash() {
        assert!(is_crash_event(&json!({
            "method": "Target.targetCrashed",
            "params": { "targetId": "t", "status": "crashed" },
        })));
    }

    #[test]
    fn ignores_unrelated_events() {
        assert!(!is_crash_event(&json!({
            "method": "Inspector.detached",
            "params": {},
        })));
        assert!(!is_crash_event(&json!({ "id": 1, "result": {} })));
        assert!(!is_crash_event(&json!({
            "method": "Target.targetCrashedExtra",
        })));
    }
}
