use crate::error::{XcelerateResult, XcelerateError};
use std::sync::atomic::{AtomicU32, Ordering};
use tokio::sync::{mpsc, oneshot};
use serde_json::{Value, json};
use serde::Serialize;

pub trait CdpCommand: Serialize {
    type Response: for<'de> serde::Deserialize<'de>;
    const METHOD: &'static str;
}

pub struct CdpClient {
    pub(crate) next_id: AtomicU32,
    pub(crate) cmd_tx: mpsc::UnboundedSender<(u32, Value, oneshot::Sender<XcelerateResult<Value>>)>,
    pub(crate) event_tx: tokio::sync::broadcast::Sender<Value>,
}

impl CdpClient {
    pub fn new(
        cmd_tx: mpsc::UnboundedSender<(u32, Value, oneshot::Sender<XcelerateResult<Value>>)>,
        event_tx: tokio::sync::broadcast::Sender<Value>,
    ) -> Self {
        Self {
            next_id: AtomicU32::new(1),
            cmd_tx,
            event_tx,
        }
    }

    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<Value> {
        self.event_tx.subscribe()
    }

    pub async fn execute<T: CdpCommand>(&self, params: T) -> XcelerateResult<T::Response> {
        self.execute_with_session(None, params).await
    }

    pub async fn execute_with_session<T: CdpCommand>(
        &self, 
        session_id: Option<&str>, 
        params: T
    ) -> XcelerateResult<T::Response> {
        let id = self.next_id.fetch_add(1, Ordering::SeqCst);
        let params_val = serde_json::to_value(params)?;
        
        let mut envelope = json!({
            "id": id,
            "method": T::METHOD,
            "params": params_val,
        });

        if let Some(sid) = session_id {
            envelope.as_object_mut().unwrap().insert("sessionId".to_string(), json!(sid));
        }

        let (tx, rx) = oneshot::channel();
        self.cmd_tx.send((id, envelope, tx)).map_err(|_| XcelerateError::InternalError)?;

        let res = rx.await.map_err(|_| XcelerateError::InternalError)??;
        let response: T::Response = serde_json::from_value(res)?;
        Ok(response)
    }
}
