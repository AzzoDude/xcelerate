//! Download handling (capability #4).
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
async fn with_deadline<F, T>(budget: u64, label: &str, fut: F) -> XcelerateResult<T>
where
    F: std::future::Future<Output = XcelerateResult<T>>,
{
    if budget == u64::MAX {
        return fut.await;
    }
    match tokio::time::timeout(std::time::Duration::from_millis(budget), fut).await {
        Ok(result) => result,
        Err(_) => Err(XcelerateError::NotFound(format!("{label} timeout"))),
    }
}

/// Whether an event belongs to this page's session. Browser-level events carry
/// no `sessionId` and are always accepted.
fn session_matches(value: &serde_json::Value, session_id: &str) -> bool {
    match value.get("sessionId").and_then(|s| s.as_str()) {
        Some(id) => id == session_id,
        None => true,
    }
}

impl Page {
    /// Enables downloads and directs them to `path`.
    ///
    /// Sends `Browser.setDownloadBehavior` on the root session with events
    /// enabled, and remembers the directory so [`Page::wait_for_download`] can
    /// resolve absolute paths.
    pub async fn set_download_path(&self, path: String) -> XcelerateResult<()> {
        self.client
            .execute_raw(
                "Browser.setDownloadBehavior",
                serde_json::json!({
                    "behavior": "allow",
                    "downloadPath": path,
                    "eventsEnabled": true,
                }),
            )
            .await?;
        *self.downloads_path.lock().await = Some(path);
        Ok(())
    }

    /// Waits for the next download to finish and returns its local path.
    ///
    /// Subscribes to CDP events, waits for `Page.downloadWillBegin` to learn the
    /// download's `guid` and `suggestedFilename`, then waits for
    /// `Browser.downloadProgress` to report `state == "completed"`. The returned
    /// path joins the directory set by [`Page::set_download_path`] with the
    /// suggested filename, or is just the filename when no directory is known.
    /// The whole wait is bounded by `timeout_ms` (`0` falls back to the page
    /// default) and fails with [`XcelerateError::NotFound`] on timeout.
    pub async fn wait_for_download(&self, timeout_ms: u64) -> XcelerateResult<String> {
        let budget = effective_timeout_ms(self, timeout_ms);
        let mut receiver = self.client.subscribe();
        let session_id = self.session_id.clone();

        with_deadline(budget, "download", async {
            // Phase 1: capture the guid and filename from `downloadWillBegin`.
            // Chrome emits this either on the page session or (via
            // `Browser.setDownloadBehavior`) on the root session.
            let (guid, filename) = loop {
                let value = match receiver.recv().await {
                    Ok(value) => value,
                    Err(RecvError::Lagged(_)) => continue,
                    Err(RecvError::Closed) => return Err(XcelerateError::InternalError),
                };
                if !session_matches(&value, &session_id) {
                    continue;
                }
                let method = value.get("method").and_then(|m| m.as_str()).unwrap_or("");
                if method != "Page.downloadWillBegin" && method != "Browser.downloadWillBegin" {
                    continue;
                }
                let params = value
                    .get("params")
                    .cloned()
                    .unwrap_or(serde_json::Value::Null);
                let guid = params
                    .get("guid")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                if guid.is_empty() {
                    continue;
                }
                let filename = params
                    .get("suggestedFilename")
                    .and_then(|v| v.as_str())
                    .unwrap_or_default()
                    .to_string();
                break (guid, filename);
            };

            // Phase 2: wait for the matching completion event.
            loop {
                let value = match receiver.recv().await {
                    Ok(value) => value,
                    Err(RecvError::Lagged(_)) => continue,
                    Err(RecvError::Closed) => return Err(XcelerateError::InternalError),
                };
                if value.get("method").and_then(|m| m.as_str()) != Some("Browser.downloadProgress")
                {
                    continue;
                }
                let params = value
                    .get("params")
                    .cloned()
                    .unwrap_or(serde_json::Value::Null);
                if params.get("guid").and_then(|v| v.as_str()) != Some(guid.as_str()) {
                    continue;
                }
                if params.get("state").and_then(|v| v.as_str()) == Some("completed") {
                    break;
                }
            }

            let dir = self.downloads_path.lock().await.clone();
            Ok(join_download_path(dir.as_deref(), &filename))
        })
        .await
    }

    /// Returns the configured download directory, if any.
    pub async fn downloads_dir(&self) -> Option<String> {
        self.downloads_path.lock().await.clone()
    }
}

/// Joins a download directory and filename.
///
/// Handles a missing directory (returns the bare filename) and trailing `/` or
/// `\` separators, reusing whichever separator the directory already uses.
pub(crate) fn join_download_path(dir: Option<&str>, filename: &str) -> String {
    let dir = match dir {
        Some(dir) if !dir.is_empty() => dir,
        _ => return filename.to_string(),
    };
    let separator = dir
        .chars()
        .rev()
        .find(|c| *c == '/' || *c == '\\')
        .unwrap_or('/');
    let base = dir.trim_end_matches(['/', '\\']);
    format!("{base}{separator}{filename}")
}

#[cfg(test)]
mod tests {
    use super::join_download_path;

    #[test]
    fn none_returns_bare_filename() {
        assert_eq!(join_download_path(None, "report.pdf"), "report.pdf");
    }

    #[test]
    fn empty_directory_returns_bare_filename() {
        assert_eq!(join_download_path(Some(""), "report.pdf"), "report.pdf");
    }

    #[test]
    fn joins_unix_directory_with_trailing_separator() {
        assert_eq!(
            join_download_path(Some("/tmp/downloads/"), "report.pdf"),
            "/tmp/downloads/report.pdf"
        );
    }

    #[test]
    fn joins_windows_directory_with_trailing_separator() {
        assert_eq!(
            join_download_path(Some("C:\\Users\\me\\Downloads\\"), "report.pdf"),
            "C:\\Users\\me\\Downloads\\report.pdf"
        );
    }

    #[test]
    fn joins_directory_without_trailing_separator() {
        assert_eq!(
            join_download_path(Some("/tmp/downloads"), "report.pdf"),
            "/tmp/downloads/report.pdf"
        );
        assert_eq!(
            join_download_path(Some("C:\\Downloads"), "report.pdf"),
            "C:\\Downloads\\report.pdf"
        );
    }

    #[test]
    fn preserves_a_root_directory_separator() {
        assert_eq!(join_download_path(Some("/"), "report.pdf"), "/report.pdf");
        assert_eq!(
            join_download_path(Some("C:\\"), "report.pdf"),
            "C:\\report.pdf"
        );
    }
}
