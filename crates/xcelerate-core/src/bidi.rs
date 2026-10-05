//! WebDriver BiDi transport — the Firefox backend's connection to the browser.
//!
//! BiDi uses the same `{ id, method, params }` command envelope as CDP, so the
//! multiplexing here mirrors [`crate::handler`]. The differences are the
//! `{ type: "success" | "error" | "event" }` discriminator and the fact that
//! events must be subscribed to.
//!
//! The wire types come from the published `webdriver-bidi` crate (re-exported
//! as [`crate::webdriver_bidi`]).

use futures::{sink::SinkExt, stream::StreamExt};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio_tungstenite::tungstenite::protocol::Message;

use crate::error::{Error, Result};

const METHOD_SESSION_NEW: &str = "session.new";

/// A client for issuing BiDi commands over a single connection.
pub struct BidiClient {
    next_id: AtomicU64,
    cmd_tx: mpsc::UnboundedSender<(u64, Value, oneshot::Sender<Result<Value>>)>,
    event_tx: broadcast::Sender<Value>,
}

impl BidiClient {
    /// Subscribe to BiDi events pushed by the browser.
    pub fn subscribe(&self) -> broadcast::Receiver<Value> {
        self.event_tx.subscribe()
    }

    /// Whether the connection is still live.
    pub fn is_connected(&self) -> bool {
        !self.cmd_tx.is_closed()
    }

    /// Sends a BiDi command and awaits its `result`.
    pub async fn command(&self, method: &str, params: Value) -> Result<Value> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let envelope = json!({ "id": id, "method": method, "params": params });
        let (tx, rx) = oneshot::channel();
        self.cmd_tx
            .send((id, envelope, tx))
            .map_err(|_| Error::Internal)?;
        rx.await.map_err(|_| Error::Internal)?
    }

    /// Runs the `session.new` handshake. It must be the first command sent.
    pub async fn new_session(&self) -> Result<Value> {
        self.command(METHOD_SESSION_NEW, json!({ "capabilities": {} }))
            .await
    }
}

/// Connects to a BiDi `/session` WebSocket endpoint and starts its handler task.
pub async fn connect(ws_url: &str) -> Result<BidiClient> {
    let (ws, _) = tokio_tungstenite::connect_async(ws_url).await?;
    let (cmd_tx, cmd_rx) = mpsc::unbounded_channel();
    let (event_tx, _) = broadcast::channel(2048);
    tokio::spawn(run(ws, cmd_rx, event_tx.clone()));
    Ok(BidiClient {
        next_id: AtomicU64::new(1),
        cmd_tx,
        event_tx,
    })
}

async fn run(
    mut ws: tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >,
    mut cmd_rx: mpsc::UnboundedReceiver<(u64, Value, oneshot::Sender<Result<Value>>)>,
    event_tx: broadcast::Sender<Value>,
) {
    let mut pending: HashMap<u64, oneshot::Sender<Result<Value>>> = HashMap::new();
    loop {
        tokio::select! {
            Some((id, message, tx)) = cmd_rx.recv() => {
                if let Ok(text) = serde_json::to_string(&message) {
                    if ws.send(Message::Text(text.into())).await.is_err() {
                        break;
                    }
                    pending.insert(id, tx);
                }
            }
            incoming = ws.next() => {
                let Some(incoming) = incoming else { break };
                let Ok(Message::Text(text)) = incoming else { continue };
                let Ok(reply): std::result::Result<Value, _> = serde_json::from_str(&text) else { continue };

                if let Some(id) = reply["id"].as_u64() {
                    if let Some(tx) = pending.remove(&id) {
                        let result = match reply["type"].as_str() {
                            Some("success") => Ok(reply["result"].clone()),
                            _ => Err(Error::Bidi {
                                error: reply["error"].as_str().unwrap_or("unknown error").to_string(),
                                message: reply["message"].as_str().unwrap_or_default().to_string(),
                            }),
                        };
                        let _ = tx.send(result);
                    }
                } else if reply["type"].as_str() == Some("event") {
                    let _ = event_tx.send(reply);
                }
            }
        }
    }
}
