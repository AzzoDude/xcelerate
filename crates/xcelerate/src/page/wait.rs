//! Network-idle and DOM-stability waits (capability #2).
//!
//! Implemented as inherent `impl Page` methods in this module so they stay out
//! of the `#[uniffi::export]` block and binding checksums remain stable.

use std::time::Instant;

use crate::error::{XcelerateError, XcelerateResult};
use crate::page::Page;

/// How often the wait helpers re-probe the page.
const POLL_INTERVAL_MS: u64 = 100;

/// JS probe reporting the load state and the number of completed resources.
const NETWORK_PROBE_JS: &str = "(function(){return {resources: performance.getEntriesByType('resource').length, readyState: document.readyState};})()";

/// JS probe that installs the mutation observer exactly once per page.
const OBSERVER_INSTALL_JS: &str = "(function(){if(!window.__xcelerateObserverInstalled){window.__xcelerateObserverInstalled=true;window.__xcelerateMutations=0;new MutationObserver(function(){window.__xcelerateMutations++;}).observe(document,{subtree:true,childList:true,attributes:true,characterData:true});}})()";

/// Resolves the caller-supplied timeout, falling back to the page default when
/// it is `0`. A resolved value of `u64::MAX` means "no timeout".
fn effective_timeout_ms(page: &Page, timeout_ms: u64) -> u64 {
    if timeout_ms == 0 {
        page.default_timeout()
    } else {
        timeout_ms
    }
}

/// A start instant plus an optional millisecond budget.
struct Deadline {
    start: Instant,
    budget: Option<u64>,
}

impl Deadline {
    fn new(page: &Page, timeout_ms: u64) -> Self {
        let millis = effective_timeout_ms(page, timeout_ms);
        Self {
            start: Instant::now(),
            budget: if millis == u64::MAX {
                None
            } else {
                Some(millis)
            },
        }
    }

    fn expired(&self) -> bool {
        match self.budget {
            Some(millis) => self.start.elapsed().as_millis() as u64 >= millis,
            None => false,
        }
    }
}

impl Page {
    /// Waits until the page is network-idle.
    ///
    /// The page is considered idle when `document.readyState === "complete"`
    /// and the number of completed resource entries has not grown for `idle_ms`
    /// milliseconds. The page is probed roughly every 100ms; the wait fails with
    /// [`XcelerateError::NotFound`] once `timeout_ms` has elapsed. A `timeout_ms`
    /// of `0` falls back to the page default timeout.
    pub async fn wait_for_network_idle(
        &self,
        idle_ms: u64,
        timeout_ms: u64,
    ) -> XcelerateResult<()> {
        let deadline = Deadline::new(self, timeout_ms);
        let mut last_count: Option<u64> = None;
        let mut stable_since = Instant::now();

        loop {
            // A failed probe (for example, a navigation mid-flight) is treated
            // as activity; the enclosing timeout bounds how long we retry.
            if let Ok(text) = self.evaluate_json(NETWORK_PROBE_JS.to_string()).await {
                let value: serde_json::Value =
                    serde_json::from_str(&text).unwrap_or(serde_json::Value::Null);
                let ready = value.get("readyState").and_then(|v| v.as_str()) == Some("complete");
                let resources = value
                    .get("resources")
                    .and_then(serde_json::Value::as_u64)
                    .unwrap_or(0);

                if ready && last_count == Some(resources) {
                    if stable_since.elapsed().as_millis() as u64 >= idle_ms {
                        return Ok(());
                    }
                } else {
                    last_count = Some(resources);
                    stable_since = Instant::now();
                }
            }

            if deadline.expired() {
                return Err(XcelerateError::NotFound("network idle timeout".into()));
            }
            tokio::time::sleep(std::time::Duration::from_millis(POLL_INTERVAL_MS)).await;
        }
    }

    /// Waits until the DOM has been quiet for `quiet_ms` milliseconds.
    ///
    /// A `MutationObserver` is installed once per page (guarded by a global
    /// flag) that increments `window.__xcelerateMutations` for every mutation.
    /// The wait succeeds once the counter has been unchanged for `quiet_ms`. The
    /// page is probed roughly every 100ms and the wait fails with
    /// [`XcelerateError::NotFound`] once `timeout_ms` has elapsed. A `timeout_ms`
    /// of `0` falls back to the page default timeout.
    pub async fn wait_for_dom_stable(&self, quiet_ms: u64, timeout_ms: u64) -> XcelerateResult<()> {
        // Best effort: if the observer cannot be installed the counter reads
        // as zero and the wait simply runs out its timeout.
        let _ = self.evaluate_json(OBSERVER_INSTALL_JS.to_string()).await;

        let deadline = Deadline::new(self, timeout_ms);
        let mut last_count: Option<u64> = None;
        let mut stable_since = Instant::now();

        loop {
            if let Ok(text) = self
                .evaluate_json("window.__xcelerateMutations".to_string())
                .await
            {
                let count = text.trim().parse::<u64>().unwrap_or(0);
                if last_count == Some(count) {
                    if stable_since.elapsed().as_millis() as u64 >= quiet_ms {
                        return Ok(());
                    }
                } else {
                    last_count = Some(count);
                    stable_since = Instant::now();
                }
            }

            if deadline.expired() {
                return Err(XcelerateError::NotFound("dom stable timeout".into()));
            }
            tokio::time::sleep(std::time::Duration::from_millis(POLL_INTERVAL_MS)).await;
        }
    }
}
