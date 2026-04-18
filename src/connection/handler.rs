use crate::error::XcelerateResult;
use futures::{sink::SinkExt, stream::StreamExt};
use serde_json::Value;
use std::collections::HashMap;
use tokio::sync::{mpsc, oneshot};
use tokio_tungstenite::{tungstenite::protocol::Message, WebSocketStream, MaybeTlsStream};
use tokio::net::TcpStream;

pub struct CdpHandler {
    ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
    cmd_rx: mpsc::UnboundedReceiver<(u32, Value, oneshot::Sender<XcelerateResult<Value>>)>,
    pending: HashMap<u32, oneshot::Sender<XcelerateResult<Value>>>,
    pub(crate) event_tx: tokio::sync::broadcast::Sender<Value>,
}

impl CdpHandler {
    pub fn new(
        ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
        cmd_rx: mpsc::UnboundedReceiver<(u32, Value, oneshot::Sender<XcelerateResult<Value>>)>,
    ) -> (Self, tokio::sync::broadcast::Receiver<Value>) {
        let (event_tx, event_rx) = tokio::sync::broadcast::channel(100);
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
                    
                    let Ok(resp): Result<Value, _> = serde_json::from_str(&text) else { continue };
                    
                    if let Some(id) = resp["id"].as_u64() {
                        if let Some(tx) = self.pending.remove(&(id as u32)) {
                            let result = if resp["error"].is_null() {
                                Ok(resp["result"].clone())
                            } else {
                                Err(crate::error::XcelerateError::CdpResponseError {
                                    code: resp["error"]["code"].as_i64().unwrap_or(0) as i32,
                                    message: resp["error"]["message"].as_str().unwrap_or("Unknown").into(),
                                })
                            };
                            let _ = tx.send(result);
                        }
                    } else if resp["method"].is_string() {
                        // This is an event, broadcast it
                        let _ = self.event_tx.send(resp);
                    }
                }
            }
        }
    }
}
