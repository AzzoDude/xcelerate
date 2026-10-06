use futures::{sink::SinkExt, stream::StreamExt};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc, oneshot};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, tungstenite::protocol::Message};

use crate::error::{Error, Result};

/// Capacity of every event channel (the catch-all and each session's).
pub(crate) const EVENT_CHANNEL_CAPACITY: usize = 1024;

/// Per-session event senders, shared between the handler task and the client so
/// any task can subscribe to a session.
pub(crate) type SessionSenders = Arc<Mutex<HashMap<String, broadcast::Sender<Value>>>>;

/// A command awaiting its reply. The method and session are recorded so the
/// reply can be attributed, and so the command can be failed promptly when its
/// session detaches instead of hanging until the connection dies.
pub(crate) struct Pending {
    pub(crate) method: String,
    pub(crate) session: Option<String>,
    pub(crate) tx: oneshot::Sender<Result<Value>>,
}

/// Mutex-protected state behind the socket: in-flight commands and the event
/// fan-out tables.
///
/// Kept separate from the socket so the routing rules can be unit-tested without
/// a live connection.
pub(crate) struct Router {
    pending: HashMap<u32, Pending>,
    event_tx: broadcast::Sender<Value>,
    sessions: SessionSenders,
}

impl Router {
    pub(crate) fn new(event_tx: broadcast::Sender<Value>, sessions: SessionSenders) -> Self {
        Self {
            pending: HashMap::new(),
            event_tx,
            sessions,
        }
    }

    /// Records a command that has just been written to the socket.
    pub(crate) fn register(
        &mut self,
        id: u32,
        method: &str,
        session: Option<&str>,
        tx: oneshot::Sender<Result<Value>>,
    ) {
        self.pending.insert(
            id,
            Pending {
                method: method.to_string(),
                session: session.map(str::to_string),
                tx,
            },
        );
    }

    /// Completes a command from its reply.
    pub(crate) fn complete(&mut self, id: u32, result: Result<Value>) {
        if let Some(pending) = self.pending.remove(&id) {
            let _ = pending.tx.send(result);
        }
    }

    /// Fails every command that is still waiting (the socket is gone).
    pub(crate) fn fail_all(&mut self, reason: &str) {
        for (_, pending) in self.pending.drain() {
            let _ = pending.tx.send(Err(Error::Ws(reason.to_string())));
        }
    }

    /// Routes one event to its subscribers and reacts to session teardown.
    ///
    /// Events carrying a `sessionId` go only to that session's subscribers.
    /// Browser-level events (no `sessionId`) go to every session subscriber, so a
    /// page still receives download and crash notifications. The catch-all backs
    /// the browser-level `wait_for_event`, which may wait on either scope.
    pub(crate) fn dispatch_event(&mut self, resp: Value) {
        // Skip the clone entirely when nothing is on the catch-all (the common
        // case): `receiver_count` is read without touching the payload.
        if self.event_tx.receiver_count() > 0 {
            let _ = self.event_tx.send(resp.clone());
        }

        // A detach can be announced either as a session-scoped event (its own
        // `sessionId`) or, for `Target.detachedFromTarget`, with the session id
        // in the params. Either way the session can no longer answer anything.
        if let Some(method) = resp.get("method").and_then(Value::as_str)
            && matches!(method, "Target.detachedFromTarget" | "Inspector.detached")
        {
            let detached = resp
                .get("sessionId")
                .and_then(Value::as_str)
                .or_else(|| resp.pointer("/params/sessionId").and_then(Value::as_str));
            if let Some(session_id) = detached.map(str::to_string) {
                self.reject_session(&session_id);
            }
        }

        let sessions = self.sessions.lock().unwrap();
        match resp.get("sessionId").and_then(Value::as_str) {
            Some(session_id) => {
                if let Some(tx) = sessions.get(session_id) {
                    let _ = tx.send(resp);
                }
            }
            // Browser-level: every session subscriber needs these too. Clone only
            // when there is more than one subscriber.
            None => match sessions.len() {
                0 => {}
                1 => {
                    if let Some(tx) = sessions.values().next() {
                        let _ = tx.send(resp);
                    }
                }
                _ => {
                    for tx in sessions.values() {
                        let _ = tx.send(resp.clone());
                    }
                }
            },
        }
    }

    /// Drops a detached session's event channel (so its subscribers end with
    /// `Closed` instead of waiting forever) and fails its in-flight commands.
    fn reject_session(&mut self, session_id: &str) {
        self.sessions.lock().unwrap().remove(session_id);

        let inflight: Vec<u32> = self
            .pending
            .iter()
            .filter(|(_, pending)| pending.session.as_deref() == Some(session_id))
            .map(|(id, _)| *id)
            .collect();
        for id in inflight {
            if let Some(pending) = self.pending.remove(&id) {
                let _ = pending.tx.send(Err(Error::Ws(format!(
                    "CDP session detached before `{}` responded",
                    pending.method
                ))));
            }
        }
    }
}

/// Owns the WebSocket and multiplexes commands/events.
///
/// A single task drives the socket: outgoing commands are written and tracked
/// by id, incoming replies are routed back to their `oneshot`, and incoming
/// events are routed by session (see [`Router`]).
pub struct CdpHandler {
    ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
    cmd_rx: mpsc::UnboundedReceiver<(u32, Value, oneshot::Sender<Result<Value>>)>,
    router: Router,
}

impl CdpHandler {
    pub fn new(
        ws: WebSocketStream<MaybeTlsStream<TcpStream>>,
        cmd_rx: mpsc::UnboundedReceiver<(u32, Value, oneshot::Sender<Result<Value>>)>,
        event_tx: broadcast::Sender<Value>,
        sessions: SessionSenders,
    ) -> Self {
        Self {
            ws,
            cmd_rx,
            router: Router::new(event_tx, sessions),
        }
    }

    pub async fn run(mut self) {
        loop {
            tokio::select! {
                Some((id, msg, tx)) = self.cmd_rx.recv() => {
                    if let Ok(text) = serde_json::to_string(&msg) {
                        if self.ws.send(Message::Text(text.into())).await.is_err() {
                            break;
                        }
                        let method = msg
                            .get("method")
                            .and_then(Value::as_str)
                            .unwrap_or_default();
                        let session = msg.get("sessionId").and_then(Value::as_str);
                        self.router.register(id, method, session, tx);
                    }
                }
                msg = self.ws.next() => {
                    let Some(msg) = msg else { break };
                    let Ok(Message::Text(text)) = msg else { continue };

                    let Ok(mut resp): std::result::Result<Value, _> = serde_json::from_str(&text) else { continue };

                    if let Some(id) = resp["id"].as_u64() {
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
                        self.router.complete(id as u32, result);
                    } else if resp["method"].is_string() {
                        self.router.dispatch_event(resp);
                    }
                }
            }
        }

        // The socket is gone: fail everything still waiting instead of leaving
        // the caller blocked on a `oneshot` that will never resolve.
        self.router.fail_all("CDP connection closed");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn router() -> Router {
        let (event_tx, _rx) = broadcast::channel(EVENT_CHANNEL_CAPACITY);
        Router::new(event_tx, Arc::new(Mutex::new(HashMap::new())))
    }

    /// Subscribes to `session_id`, creating the sender as `subscribe_session`
    /// would, and returns the receiver.
    fn subscribe(router: &Router, session_id: &str) -> broadcast::Receiver<Value> {
        router
            .sessions
            .lock()
            .unwrap()
            .entry(session_id.to_string())
            .or_insert_with(|| broadcast::channel(EVENT_CHANNEL_CAPACITY).0)
            .subscribe()
    }

    #[test]
    fn session_events_reach_only_their_own_subscriber() {
        let mut router = router();
        let mut a = subscribe(&router, "A");
        let mut b = subscribe(&router, "B");

        router.dispatch_event(serde_json::json!({ "method": "Network.x", "sessionId": "A" }));

        assert_eq!(a.try_recv().unwrap()["method"], "Network.x");
        assert!(b.try_recv().is_err());
    }

    #[test]
    fn browser_events_reach_every_session_subscriber() {
        let mut router = router();
        let mut a = subscribe(&router, "A");
        let mut b = subscribe(&router, "B");

        router.dispatch_event(serde_json::json!({ "method": "Browser.downloadProgress" }));

        assert_eq!(a.try_recv().unwrap()["method"], "Browser.downloadProgress");
        assert_eq!(b.try_recv().unwrap()["method"], "Browser.downloadProgress");
    }

    #[test]
    fn the_catch_all_sees_everything() {
        let (event_tx, mut rx) = broadcast::channel(EVENT_CHANNEL_CAPACITY);
        let mut router = Router::new(event_tx, Arc::new(Mutex::new(HashMap::new())));

        router.dispatch_event(serde_json::json!({ "method": "Network.x", "sessionId": "A" }));
        router.dispatch_event(serde_json::json!({ "method": "Target.targetCreated" }));

        assert_eq!(rx.try_recv().unwrap()["method"], "Network.x");
        assert_eq!(rx.try_recv().unwrap()["method"], "Target.targetCreated");
    }

    #[test]
    fn detaching_a_session_fails_its_inflight_commands_and_closes_subscribers() {
        let mut router = router();
        let mut receiver = subscribe(&router, "A");

        let (tx, mut rx) = oneshot::channel();
        router.register(7, "Runtime.evaluate", Some("A"), tx);
        // A command on another session must be left alone.
        let (_other_tx, _other_rx) = oneshot::channel();
        router.register(8, "Page.navigate", Some("B"), _other_tx);

        router.dispatch_event(serde_json::json!({
            "method": "Target.detachedFromTarget",
            "params": { "sessionId": "A" }
        }));

        let error = rx.try_recv().expect("inflight rejected").unwrap_err();
        assert!(
            error.to_string().contains("Runtime.evaluate"),
            "unexpected error: {error}"
        );
        // Session A's channel is gone, so its subscriber sees `Closed`.
        assert!(matches!(
            receiver.try_recv(),
            Err(broadcast::error::TryRecvError::Closed)
        ));
    }

    #[test]
    fn failing_all_rejects_every_pending_command() {
        let mut router = router();
        let (tx, mut rx) = oneshot::channel();
        router.register(1, "Page.navigate", None, tx);

        router.fail_all("CDP connection closed");

        assert!(
            rx.try_recv()
                .unwrap()
                .unwrap_err()
                .to_string()
                .contains("closed")
        );
    }
}
