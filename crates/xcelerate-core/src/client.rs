use std::sync::atomic::{AtomicU32, Ordering};

use serde_json::{Value, json};
use tokio::sync::{broadcast, mpsc, oneshot};

use crate::command::CdpCommand;
use crate::error::{Error, Result};
use crate::handler::CdpHandler;

/// A client for issuing typed CDP commands over a single connection.
pub struct CdpClient {
    pub(crate) next_id: AtomicU32,
    pub(crate) cmd_tx: mpsc::UnboundedSender<(u32, Value, oneshot::Sender<Result<Value>>)>,
    pub(crate) event_tx: broadcast::Sender<Value>,
}

impl CdpClient {
    pub fn new(
        cmd_tx: mpsc::UnboundedSender<(u32, Value, oneshot::Sender<Result<Value>>)>,
        event_tx: broadcast::Sender<Value>,
    ) -> Self {
        Self {
            next_id: AtomicU32::new(1),
            cmd_tx,
            event_tx,
        }
    }

    /// Subscribe to CDP events broadcast by the connection.
    pub fn subscribe(&self) -> broadcast::Receiver<Value> {
        self.event_tx.subscribe()
    }

    /// Whether the connection is still live.
    pub fn is_connected(&self) -> bool {
        !self.cmd_tx.is_closed()
    }

    /// Sends a command on the root session.
    pub async fn execute<T: CdpCommand>(&self, params: T) -> Result<T::Response> {
        self.execute_with_session(None, params).await
    }

    /// Sends a raw CDP command (by method name) and returns the raw response.
    pub async fn execute_raw(&self, method: &str, params: Value) -> Result<Value> {
        self.execute_raw_with_session(None, method, params).await
    }

    /// Sends a raw CDP command scoped to an optional flattened session id.
    pub async fn execute_raw_with_session(
        &self,
        session_id: Option<&str>,
        method: &str,
        params: Value,
    ) -> Result<Value> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let mut envelope = json!({ "id": id, "method": method, "params": params });
        if let Some(sid) = session_id {
            envelope["sessionId"] = json!(sid);
        }

        let (tx, rx) = oneshot::channel();
        self.cmd_tx
            .send((id, envelope, tx))
            .map_err(|_| Error::Internal)?;
        rx.await.map_err(|_| Error::Internal)?
    }

    /// Sends a command, optionally scoped to a flattened session id.
    pub async fn execute_with_session<T: CdpCommand>(
        &self,
        session_id: Option<&str>,
        params: T,
    ) -> Result<T::Response> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let params_val = serde_json::to_value(params)?;

        let mut envelope = json!({
            "id": id,
            "method": T::METHOD,
            "params": params_val,
        });

        if let Some(sid) = session_id {
            envelope["sessionId"] = json!(sid);
        }

        let (tx, rx) = oneshot::channel();
        self.cmd_tx
            .send((id, envelope, tx))
            .map_err(|_| Error::Internal)?;

        let res = rx.await.map_err(|_| Error::Internal)??;
        let response: T::Response = serde_json::from_value(res)?;
        Ok(response)
    }

    /// Sends a raw CDP command scoped to an optional session id, aborting with
    /// [`Error::Ws`] if no response arrives within `timeout`.
    ///
    /// A send failure or a dropped response channel is reported as
    /// [`Error::Internal`]. Existing (untimed) callers are unaffected.
    pub async fn execute_raw_with_session_timeout(
        &self,
        session_id: Option<&str>,
        method: &str,
        params: Value,
        timeout: std::time::Duration,
    ) -> Result<Value> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let mut envelope = json!({ "id": id, "method": method, "params": params });
        if let Some(sid) = session_id {
            envelope["sessionId"] = json!(sid);
        }

        let (tx, rx) = oneshot::channel();
        self.cmd_tx
            .send((id, envelope, tx))
            .map_err(|_| Error::Internal)?;

        match tokio::time::timeout(timeout, rx).await {
            Ok(Ok(result)) => result,
            Ok(Err(_)) => Err(Error::Internal),
            Err(_) => Err(Error::Ws(format!(
                "CDP call `{method}` timed out after {timeout:?}"
            ))),
        }
    }
}

/// Connects to a CDP WebSocket endpoint and starts its handler task.
pub async fn connect(ws_url: &str) -> Result<CdpClient> {
    let (ws, _) = tokio_tungstenite::connect_async(ws_url).await?;
    let (tx, rx) = mpsc::unbounded_channel();
    let (handler, _events) = CdpHandler::new(ws, rx);
    let client = CdpClient::new(tx, handler.event_tx.clone());
    tokio::spawn(handler.run());
    Ok(client)
}
