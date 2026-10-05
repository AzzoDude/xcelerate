//! Per-call CDP timeouts (capability #11).
//!
//! Implemented as inherent `impl Page` methods in this module so they stay out
//! of the `#[uniffi::export]` block and binding checksums remain stable.

use std::time::Duration;

use crate::error::XcelerateResult;
use crate::page::Page;

impl Page {
    /// Escape hatch like [`Page::execute_cdp_cmd`], but with an explicit
    /// per-call timeout.
    ///
    /// `params_json` is parsed as JSON (falling back to `null` when it is not
    /// valid JSON). The command is abandoned and reported as an error if no
    /// response arrives within `timeout_ms` milliseconds; a value of `0` is
    /// clamped to `1` so the call still gets a chance to complete.
    pub async fn execute_cdp_cmd_timeout(
        &self,
        method: String,
        params_json: String,
        timeout_ms: u64,
    ) -> XcelerateResult<String> {
        let params: serde_json::Value =
            serde_json::from_str(&params_json).unwrap_or(serde_json::Value::Null);
        let res = self
            .client
            .execute_raw_with_session_timeout(
                Some(&self.session_id),
                &method,
                params,
                Duration::from_millis(timeout_ms.max(1)),
            )
            .await?;
        Ok(res.to_string())
    }
}
