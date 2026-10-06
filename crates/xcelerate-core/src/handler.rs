use futures::{sink::SinkExt, stream::StreamExt};
use serde_json::Value;
use std::collections::HashMap;
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, tungstenite::protocol::Message};

use crate::error::{Error, Result};

/// Owns the WebSocket and multiplexes commands/events.
///
/// A single task drives the socket: outgoing commands are written and tracked
/// by id, incoming replies are routed back to their `oneshot`, and incoming
/// events are broadcast to subscribers.
pub struct CdpHandler {
    ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
    cmd_rx: mpsc::UnboundedReceiver<(u32, Value, oneshot::Sender<Result<Value>>)>,
    pending: HashMap<u32, oneshot::Sender<Result<Value>>>,
    pub(crate) event_tx: broadcast::Sender<Value>,
}

impl CdpHandler {
    pub fn new(
        ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
        cmd_rx: mpsc::UnboundedReceiver<(u32, Value, oneshot::Sender<Result<Value>>)>,
    ) -> (Self, broadcast::Receiver<Value>) {
        let (event_tx, event_rx) = broadcast::channel(1024);
        let handler = Self {
            ws,
            cmd_rx,
            pending: HashMap::new(),
            event_tx,
        };
        (handler, event_rx)
    }

    pub async fn run(mut self) {
        loop {
            tokio::select! {
                Some((id, msg, tx)) = self.cmd_rx.recv() => {
                    if let Ok(text) = serde_json::to_string(&msg) {
                        if self.ws.send(Message::Text(text.into())).await.is_err() {
                            break;
                        }
                        self.pending.insert(id, tx);
                    }
                }
                msg = self.ws.next() => {
                    let Some(msg) = msg else { break };
                    let Ok(Message::Text(text)) = msg else { continue };

                    let Ok(mut resp): std::result::Result<Value, _> = serde_json::from_str(&text) else { continue };

                    if let Some(id) = resp["id"].as_u64() {
                        if let Some(tx) = self.pending.remove(&(id as u32)) {
                            let result = if resp["error"].is_null() {
                                // Move the result out instead of deep-cloning it;
                                // responses can be large (screenshots, DOM trees).
                                Ok(resp.get_mut("result").map(std::mem::take).unwrap_or(Value::Null))
                            } else {
                                Err(Error::Cdp {
                                    code: resp["error"]["code"].as_i64().unwrap_or(0) as i32,
                                    message: resp["error"]["message"].as_str().unwrap_or("Unknown").into(),
                                })
                            };
                            let _ = tx.send(result);
                        }
                    } else if resp["method"].is_string() {
                        // This is an event, broadcast it.
                        let _ = self.event_tx.send(resp);
                    }
                }
            }
        }
    }
}
