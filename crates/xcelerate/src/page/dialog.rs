//! JavaScript-dialog safety (capability #1).
//!
//! `alert`/`confirm`/`prompt`/`beforeunload` block the renderer: while a dialog
//! is open, every `Runtime.evaluate` on the page blocks until it is answered, and
//! page script cannot dismiss it. Left alone, a single `alert()` would wedge the
//! whole run with no way back.
//!
//! To make that impossible, every page runs a background watcher subscribed to
//! `Page.javascriptDialogOpening`. It answers the dialog with the page's
//! [`DialogPolicy`] (default [`DialogPolicy::Dismiss`]) via
//! `Page.handleJavaScriptDialog`, which resolves the dialog from the browser
//! process even while the renderer is blocked.
//!
//! Implemented as inherent `impl Page` methods in this module so they stay out of
//! the `#[uniffi::export]` block and binding checksums remain stable.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use crate::CdpClient;
use crate::error::XcelerateResult;
use crate::page::Page;

/// What to do when the page opens a JavaScript dialog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DialogPolicy {
    /// Dismiss every dialog automatically. The default, so a page can never
    /// block the run on a stray `alert`/`confirm`/`prompt`.
    #[default]
    Dismiss,
    /// Accept every dialog automatically. Useful for `beforeunload` prompts.
    Accept,
}

impl DialogPolicy {
    fn to_u8(self) -> u8 {
        match self {
            DialogPolicy::Dismiss => 0,
            DialogPolicy::Accept => 1,
        }
    }

    fn from_u8(value: u8) -> Self {
        if value == DialogPolicy::Accept.to_u8() {
            DialogPolicy::Accept
        } else {
            DialogPolicy::Dismiss
        }
    }
}

/// Per-session dialog state. Lives in a process-global registry (keyed by CDP
/// session id) because adding a field to [`Page`] would require touching the
/// `browser` module that constructs it.
struct DialogController {
    policy: AtomicU8,
    /// Set once the watcher has enabled the `Page` domain and subscribed.
    started: tokio::sync::OnceCell<()>,
}

/// The process-global dialog controllers, keyed by CDP session id.
fn controllers() -> &'static Mutex<HashMap<String, Arc<DialogController>>> {
    static CONTROLLERS: OnceLock<Mutex<HashMap<String, Arc<DialogController>>>> = OnceLock::new();
    CONTROLLERS.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Returns the controller for `session_id`, creating it (with the default
/// policy) on first use.
fn controller_for(session_id: &str) -> Arc<DialogController> {
    let mut map = controllers().lock().unwrap();
    Arc::clone(map.entry(session_id.to_string()).or_insert_with(|| {
        Arc::new(DialogController {
            policy: AtomicU8::new(DialogPolicy::default().to_u8()),
            started: tokio::sync::OnceCell::new(),
        })
    }))
}

/// Background watcher: answers `Page.javascriptDialogOpening` with the current
/// policy until the session's event stream closes.
async fn run_dialog_watcher(
    client: Arc<CdpClient>,
    session_id: String,
    controller: Arc<DialogController>,
    ready: tokio::sync::oneshot::Sender<()>,
) {
    // The `Page` domain must be enabled for the event to be delivered. Best
    // effort: if it fails the watcher simply never fires.
    let _ = client
        .execute_raw_with_session(Some(&session_id), "Page.enable", serde_json::json!({}))
        .await;
    let mut receiver = client.subscribe_session(&session_id);
    // The subscription is live; the caller may proceed.
    let _ = ready.send(());
    loop {
        let value = match receiver.recv().await {
            Ok(value) => value,
            // A slow consumer only misses events; keep pumping.
            Err(tokio::sync::broadcast::error::RecvError::Lagged(_)) => continue,
            Err(tokio::sync::broadcast::error::RecvError::Closed) => break,
        };
        if value.get("sessionId").and_then(|s| s.as_str()) != Some(session_id.as_str()) {
            continue;
        }
        if value.get("method").and_then(|m| m.as_str()) != Some("Page.javascriptDialogOpening") {
            continue;
        }
        let accept = controller.policy.load(Ordering::SeqCst) == DialogPolicy::Accept.to_u8();
        // Answering is fire-and-forget; if it fails the next dialog still tries.
        let _ = client
            .execute_raw_with_session(
                Some(&session_id),
                "Page.handleJavaScriptDialog",
                serde_json::json!({ "accept": accept }),
            )
            .await;
    }
    // The session is gone; forget the controller so the registry does not grow.
    controllers().lock().unwrap().remove(&session_id);
}

impl Page {
    /// Ensures this page's dialog watcher is running.
    ///
    /// Idempotent and cheap after the first call. Called from every action that
    /// can run page script, so a dialog opened by any of them is answered.
    pub(crate) async fn ensure_dialog_watcher(&self) {
        let controller = controller_for(&self.session_id);
        let client = Arc::clone(&self.client);
        let session_id = self.session_id.clone();
        controller
            .started
            .get_or_init(|| async {
                let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
                tokio::spawn(run_dialog_watcher(
                    client,
                    session_id,
                    Arc::clone(&controller),
                    ready_tx,
                ));
                // Do not return until the watcher has subscribed, so a dialog
                // opened immediately afterwards cannot be missed.
                let _ = ready_rx.await;
            })
            .await;
    }

    /// Sets the policy used to answer JavaScript dialogs on this page.
    ///
    /// Starts the watcher; the default is [`DialogPolicy::Dismiss`].
    pub async fn set_dialog_policy(&self, policy: DialogPolicy) -> XcelerateResult<()> {
        controller_for(&self.session_id)
            .policy
            .store(policy.to_u8(), Ordering::SeqCst);
        self.ensure_dialog_watcher().await;
        Ok(())
    }

    /// The policy currently used to answer JavaScript dialogs.
    pub async fn dialog_policy(&self) -> DialogPolicy {
        DialogPolicy::from_u8(
            controller_for(&self.session_id)
                .policy
                .load(Ordering::SeqCst),
        )
    }
}
