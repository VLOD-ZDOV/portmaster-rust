//! Reactive WebSocket client for PortAPI (WASM).
//!
//! Connects to `ws://127.0.0.1:817/api/database/v1`, multiplexes requests by `opID`,
//! tracks connection state, ping/latency, and manages subscriptions.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use futures::channel::{mpsc, oneshot};
use futures::future::select;
use futures::pin_mut;
use futures::{SinkExt, StreamExt};
use gloo_net::websocket::futures::WebSocket;
use gloo_net::websocket::Message;
use leptos::prelude::*;
use pm_models::protocol::{reply_types, PortApiFrame, PortApiRequest, DB_API_PATH_DEFAULT};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;

/// Simple browser timeout future.
pub async fn sleep_ms(ms: i32) {
    let (tx, rx) = oneshot::channel();
    if let Some(window) = web_sys::window() {
        let cb = wasm_bindgen::closure::Closure::once_into_js(move || {
            let _ = tx.send(());
        });
        let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
            cb.as_ref().unchecked_ref(),
            ms,
        );
    }
    let _ = rx.await;
}

/// Connection status to the Portmaster core.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[allow(dead_code)]
pub enum ConnectionState {
    #[default]
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
}

#[allow(dead_code)]
impl ConnectionState {
    pub fn is_connected(&self) -> bool {
        matches!(self, Self::Connected)
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::Connected => "Connected",
            Self::Connecting => "Connecting...",
            Self::Reconnecting => "Reconnecting...",
            Self::Disconnected => "Offline",
        }
    }
}

type SubCallback = Arc<dyn Fn(PortApiFrame) + Send + Sync + 'static>;
type QueryStreamEntry = (Vec<PortApiFrame>, oneshot::Sender<Result<Vec<PortApiFrame>, String>>);

/// Shared state of active in-flight requests and subscriptions.
#[derive(Default)]
struct ClientInner {
    pending_oneshot: HashMap<String, oneshot::Sender<PortApiFrame>>,
    pending_query_stream: HashMap<String, QueryStreamEntry>,
    subscriptions: HashMap<String, (String, SubCallback)>,
}

/// The reactive PortAPI client instance.
#[derive(Clone)]
pub struct PortApiClient {
    pub url: Arc<String>,
    pub state: ReadSignal<ConnectionState>,
    set_state: WriteSignal<ConnectionState>,
    pub ping: ReadSignal<Option<u32>>,
    set_ping: WriteSignal<Option<u32>>,
    tx: mpsc::UnboundedSender<String>,
    counter: Arc<AtomicU64>,
    inner: Arc<Mutex<ClientInner>>,
}

#[allow(dead_code)]
impl PortApiClient {
    /// Connects to the local Portmaster core.
    pub fn new(ws_endpoint: Option<&str>) -> Self {
        let (state, set_state) = signal(ConnectionState::Disconnected);
        let (ping, set_ping) = signal(None);
        let (tx, rx) = mpsc::unbounded::<String>();

        let endpoint = ws_endpoint
            .map(|s| s.to_string())
            .unwrap_or_else(|| {
                if let Some(window) = web_sys::window() {
                    let loc = window.location();
                    if let Ok(host) = loc.host() {
                        let is_allowed_host = host == "127.0.0.1:817"
                            || host == "localhost:817"
                            || host.starts_with("127.0.0.1:")
                            || host.starts_with("localhost:");
                        if is_allowed_host {
                            let proto = if loc.protocol().unwrap_or_default() == "https:" { "wss:" } else { "ws:" };
                            return format!("{}//{}/api/database/v1", proto, host);
                        }
                    }
                }
                format!("ws://127.0.0.1:817{}", DB_API_PATH_DEFAULT)
            });

        let client = Self {
            url: Arc::new(endpoint),
            state,
            set_state,
            ping,
            set_ping,
            tx,
            counter: Arc::new(AtomicU64::new(1)),
            inner: Arc::new(Mutex::new(ClientInner::default())),
        };

        // Spawn background connection runner
        client.start_connection_loop(rx);

        client
    }

    /// Next unique operation ID.
    pub fn next_op_id(&self, prefix: &str) -> String {
        let id = self.counter.fetch_add(1, Ordering::Relaxed);
        format!("{prefix}-{id}")
    }

    /// Background task that maintains connection, sends frames, and handles incoming messages.
    fn start_connection_loop(&self, mut outgoing_rx: mpsc::UnboundedReceiver<String>) {
        let url = (*self.url).clone();
        let set_state = self.set_state;
        let set_ping = self.set_ping;
        let inner = Arc::clone(&self.inner);
        let tx = self.tx.clone();

        spawn_local(async move {
            let mut backoff_ms = 500u64;

            loop {
                set_state.set(ConnectionState::Connecting);

                match WebSocket::open(&url) {
                    Ok(ws) => {
                        set_state.set(ConnectionState::Connected);
                        set_ping.set(Some(1));
                        backoff_ms = 500;

                        // Re-subscribe active subscriptions on the newly established connection
                        let resend_msgs: Vec<String> = {
                            let inner_guard = inner.lock().expect("lock client inner");
                            inner_guard
                                .subscriptions
                                .values()
                                .map(|(wire, _)| wire.clone())
                                .collect()
                        };
                        for msg in resend_msgs {
                            let _ = tx.unbounded_send(msg);
                        }

                        let (mut ws_sink, mut ws_stream) = ws.split();

                        // Loop reading outgoing frames and writing to WS
                        let writer_fut = async {
                            while let Some(text) = outgoing_rx.next().await {
                                if ws_sink.send(Message::Text(text)).await.is_err() {
                                    break;
                                }
                            }
                        };

                        // Read incoming messages from WebSocket
                        let inner_reader = Arc::clone(&inner);
                        let reader_fut = async {
                            while let Some(msg) = ws_stream.next().await {
                                match msg {
                                    Ok(Message::Text(text)) => {
                                        if let Ok(frame) = PortApiFrame::parse(&text) {
                                            Self::dispatch_frame(&inner_reader, frame);
                                        }
                                    }
                                    Ok(Message::Bytes(bytes)) => {
                                        if let Ok(text) = String::from_utf8(bytes) {
                                            if let Ok(frame) = PortApiFrame::parse(&text) {
                                                Self::dispatch_frame(&inner_reader, frame);
                                            }
                                        }
                                    }
                                    Err(_) => {
                                        break;
                                    }
                                }
                            }
                        };

                        pin_mut!(writer_fut);
                        pin_mut!(reader_fut);

                        // Concurrently run reader and writer until either disconnects or encounters an error.
                        // Dropping select terminates both futures and safely releases all resources without leaking tasks or locks.
                        let _ = select(writer_fut, reader_fut).await;
                    }
                    Err(_) => {
                        // Failed to open socket
                    }
                }

                // Cleanup broken requests and notify waiters of socket disconnection
                {
                    let mut inner_guard = inner.lock().expect("lock client inner");
                    inner_guard.pending_oneshot.clear();
                    inner_guard.pending_query_stream.clear();
                }

                set_state.set(ConnectionState::Reconnecting);
                set_ping.set(None);

                // Wait before retrying
                sleep_ms(backoff_ms as i32).await;
                backoff_ms = (backoff_ms * 2).min(5000);
            }
        });
    }

    /// Dispatches an incoming frame to pending oneshots or subscriptions.
    fn dispatch_frame(inner_lock: &Arc<Mutex<ClientInner>>, frame: PortApiFrame) {
        let mut inner = inner_lock.lock().expect("lock client inner");

        // 1. Check for single-request oneshots (e.g. get, create, update, delete)
        if let Some(sender) = inner.pending_oneshot.remove(&frame.id) {
            let _ = sender.send(frame);
            return;
        }

        // 2. Check for query streams
        if let Some((records, _sender)) = inner.pending_query_stream.get_mut(&frame.id) {
            match frame.msg_type.as_str() {
                reply_types::OK => {
                    if records.len() < 10_000 {
                        records.push(frame);
                    }
                    return;
                }
                reply_types::DONE => {
                    if let Some((done_records, sender)) =
                        inner.pending_query_stream.remove(&frame.id)
                    {
                        let _ = sender.send(Ok(done_records));
                    }
                    return;
                }
                reply_types::ERROR => {
                    if let Some((_, sender)) =
                        inner.pending_query_stream.remove(&frame.id)
                    {
                        let err_msg = frame.key.unwrap_or_else(|| "database query error".to_string());
                        let _ = sender.send(Err(err_msg));
                    }
                    return;
                }
                _ => {}
            }
        }

        // 3. Check for subscriptions (upd, new, del)
        // Extract callback and drop mutex lock before invoking callback to prevent deadlocks
        let cb = inner.subscriptions.get(&frame.id).map(|(_, cb)| cb.clone());
        drop(inner);
        if let Some(cb) = cb {
            cb(frame);
        }
    }

    /// Sends a raw frame string.
    pub fn send_raw(&self, frame_str: String) {
        let _ = self.tx.unbounded_send(frame_str);
    }

    /// Performs a `get` request for a single database key.
    pub async fn get(&self, key: &str) -> Result<PortApiFrame, String> {
        let op_id = self.next_op_id("get");
        let req = PortApiRequest::get(&op_id, key);

        let (resp_tx, resp_rx) = oneshot::channel();
        {
            let mut inner = self.inner.lock().expect("lock inner");
            inner.pending_oneshot.insert(op_id, resp_tx);
        }

        self.send_raw(req.to_wire());

        match resp_rx.await {
            Ok(frame) => {
                if frame.msg_type == reply_types::ERROR {
                    Err(frame.key.unwrap_or_else(|| "unknown error".to_string()))
                } else {
                    Ok(frame)
                }
            }
            Err(_) => Err("request cancelled or connection closed".to_string()),
        }
    }

    /// Performs a `query` request and collects all records until `done`.
    pub async fn query(&self, query: &str) -> Result<Vec<PortApiFrame>, String> {
        let op_id = self.next_op_id("query");
        let req = PortApiRequest::query(&op_id, query);

        let (resp_tx, resp_rx) = oneshot::channel();
        {
            let mut inner = self.inner.lock().expect("lock inner");
            inner
                .pending_query_stream
                .insert(op_id, (Vec::new(), resp_tx));
        }

        self.send_raw(req.to_wire());

        match resp_rx.await {
            Ok(result) => result,
            Err(_) => Err("query cancelled or connection closed".to_string()),
        }
    }

    /// Subscribes to updates using `sub`.
    pub fn sub<F>(&self, query: &str, callback: F) -> String
    where
        F: Fn(PortApiFrame) + Send + Sync + 'static,
    {
        let op_id = self.next_op_id("sub");
        let req = PortApiRequest::sub(&op_id, query);
        let wire = req.to_wire();

        {
            let mut inner = self.inner.lock().expect("lock inner");
            inner
                .subscriptions
                .insert(op_id.clone(), (wire.clone(), Arc::new(callback)));
        }

        self.send_raw(wire);
        op_id
    }

    /// Subscribes to initial state and updates using `qsub`.
    pub fn qsub<F>(&self, query: &str, callback: F) -> String
    where
        F: Fn(PortApiFrame) + Send + Sync + 'static,
    {
        let op_id = self.next_op_id("qsub");
        let req = PortApiRequest::qsub(&op_id, query);
        let wire = req.to_wire();

        {
            let mut inner = self.inner.lock().expect("lock inner");
            inner
                .subscriptions
                .insert(op_id.clone(), (wire.clone(), Arc::new(callback)));
        }

        self.send_raw(wire);
        op_id
    }

    /// Cancels an active subscription.
    pub fn cancel(&self, op_id: &str) {
        {
            let mut inner = self.inner.lock().expect("lock inner");
            inner.subscriptions.remove(op_id);
        }
        let req = PortApiRequest::cancel(op_id);
        self.send_raw(req.to_wire());
    }

    /// Creates or updates a record.
    pub async fn put(
        &self,
        msg_type: &str,
        key: &str,
        data: serde_json::Value,
    ) -> Result<(), String> {
        let op_id = self.next_op_id(msg_type);
        let frame = PortApiFrame {
            id: op_id.clone(),
            msg_type: msg_type.to_string(),
            key: Some(key.to_string()),
            data: Some(serde_json::to_string(&data).unwrap_or_else(|_| "{}".to_string())),
        };

        let (resp_tx, resp_rx) = oneshot::channel();
        {
            let mut inner = self.inner.lock().expect("lock inner");
            inner.pending_oneshot.insert(op_id, resp_tx);
        }

        self.send_raw(frame.to_wire());

        match resp_rx.await {
            Ok(reply) => {
                if reply.msg_type == reply_types::SUCCESS || reply.msg_type == reply_types::OK {
                    Ok(())
                } else {
                    Err(reply.key.unwrap_or_else(|| "error".to_string()))
                }
            }
            Err(_) => Err("request cancelled or dropped".to_string()),
        }
    }

    /// Deletes a record.
    pub async fn delete(&self, key: &str) -> Result<(), String> {
        let op_id = self.next_op_id("delete");
        let req = PortApiRequest::delete(&op_id, key);

        let (resp_tx, resp_rx) = oneshot::channel();
        {
            let mut inner = self.inner.lock().expect("lock inner");
            inner.pending_oneshot.insert(op_id, resp_tx);
        }

        self.send_raw(req.to_wire());

        match resp_rx.await {
            Ok(reply) => {
                if reply.msg_type == reply_types::SUCCESS {
                    Ok(())
                } else {
                    Err(reply.key.unwrap_or_else(|| "error deleting record".to_string()))
                }
            }
            Err(_) => Err("request cancelled".to_string()),
        }
    }
}
