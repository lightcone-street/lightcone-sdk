//! Native WebSocket client — `tokio-tungstenite`.
//!
//! Full implementation with:
//! - Background tokio task for connection management
//! - Application-level ping/pong health check
//! - Exponential backoff reconnection with jitter
//! - Subscription tracking + auto-resubscribe on reconnect
//! - Message queue when disconnected (pending messages flushed on reconnect)
//! - Stream-based event delivery to consumer

use std::pin::Pin;
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::Arc;
use std::time::Duration;

use futures_util::stream::{SplitSink, SplitStream, Stream};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

use crate::error::WsError;
use crate::ws::subscriptions::Subscription;
use crate::ws::{Kind, MessageIn, MessageOut, ReadyState, SubscribeParams, UnsubscribeParams, WsConfig, WsEvent};

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

// ─── Commands from public API to background task ─────────────────────────────

enum Command {
    Send(MessageOut),
    ClearAuthedSubs,
    Disconnect,
}

// ─── Disconnect reasons for reconnection decision ────────────────────────────

enum DisconnectReason {
    UserRequested,
    NormalClose,
    PongTimeout,
    RateLimited,
    Error(String),
}

// ─── Background task state ───────────────────────────────────────────────────

struct TaskState {
    config: WsConfig,
    event_tx: mpsc::Sender<WsEvent>,
    cmd_rx: mpsc::Receiver<Command>,
    active_subscriptions: Vec<SubscribeParams>,
    pending_messages: Vec<MessageOut>,
    reconnect_attempts: u32,
    ready_state: Arc<AtomicU16>,
}

impl TaskState {
    fn emit(&self, event: WsEvent) {
        let _ = self.event_tx.try_send(event);
    }

    fn should_reconnect(&self) -> bool {
        self.config.reconnect
            && self.reconnect_attempts < self.config.max_reconnect_attempts
    }
}

// ─── Public WsClient ─────────────────────────────────────────────────────────

/// Native WebSocket client using `tokio-tungstenite`.
///
/// Uses a background tokio task for connection management.
/// The public API communicates with it via mpsc channels.
pub struct WsClient {
    config: WsConfig,
    cmd_tx: Option<mpsc::Sender<Command>>,
    event_rx: tokio::sync::Mutex<mpsc::Receiver<WsEvent>>,
    event_tx: mpsc::Sender<WsEvent>,
    task_handle: Option<JoinHandle<()>>,
    ready_state: Arc<AtomicU16>,
}

impl WsClient {
    /// Create a new WS client. Does not connect yet.
    pub fn new(config: WsConfig) -> Self {
        let (event_tx, event_rx) = mpsc::channel(256);
        Self {
            config,
            cmd_tx: None,
            event_rx: tokio::sync::Mutex::new(event_rx),
            event_tx,
            task_handle: None,
            ready_state: Arc::new(AtomicU16::new(ReadyState::Closed as u16)),
        }
    }

    /// Connect to the WebSocket server.
    ///
    /// Spawns a background tokio task that manages the connection,
    /// ping/pong keepalive, reconnection, and subscription tracking.
    pub async fn connect(&mut self) -> Result<(), WsError> {
        if self.cmd_tx.is_some() {
            return Ok(());
        }

        let (cmd_tx, cmd_rx) = mpsc::channel(64);
        self.cmd_tx = Some(cmd_tx);
        self.ready_state.store(ReadyState::Connecting as u16, Ordering::SeqCst);

        let state = TaskState {
            config: self.config.clone(),
            event_tx: self.event_tx.clone(),
            cmd_rx,
            active_subscriptions: Vec::new(),
            pending_messages: Vec::new(),
            reconnect_attempts: 0,
            ready_state: Arc::clone(&self.ready_state),
        };

        let handle = tokio::spawn(run_task(state));
        self.task_handle = Some(handle);

        Ok(())
    }

    /// Disconnect from the WebSocket server.
    ///
    /// Sends a graceful close to the background task and waits for it to finish.
    pub async fn disconnect(&mut self) -> Result<(), WsError> {
        if let Some(tx) = self.cmd_tx.take() {
            let _ = tx.send(Command::Disconnect).await;
        }

        if let Some(handle) = self.task_handle.take() {
            let _ = tokio::time::timeout(Duration::from_secs(5), handle).await;
        }

        self.ready_state.store(ReadyState::Closed as u16, Ordering::SeqCst);
        Ok(())
    }

    /// Send a message to the server.
    ///
    /// If connected, sends immediately via the background task.
    /// Returns `WsError::NotConnected` if no connection is active.
    pub fn send(&self, msg: MessageOut) -> Result<(), WsError> {
        match &self.cmd_tx {
            Some(tx) => tx.try_send(Command::Send(msg)).map_err(|e| match e {
                mpsc::error::TrySendError::Full(_) => {
                    WsError::SendFailed("Command channel full".into())
                }
                mpsc::error::TrySendError::Closed(_) => WsError::NotConnected,
            }),
            None => Err(WsError::NotConnected),
        }
    }

    /// Subscribe to a channel.
    pub fn subscribe(&self, params: SubscribeParams) -> Result<(), WsError> {
        self.send(MessageOut::Subscribe(params))
    }

    /// Unsubscribe from a channel.
    pub fn unsubscribe(&self, params: UnsubscribeParams) -> Result<(), WsError> {
        self.send(MessageOut::Unsubscribe(params))
    }

    /// Whether the WebSocket is currently open.
    pub fn is_connected(&self) -> bool {
        self.ready_state() == ReadyState::Open
    }

    /// Current connection state.
    pub fn ready_state(&self) -> ReadyState {
        ReadyState::from(self.ready_state.load(Ordering::SeqCst))
    }

    /// Force a fresh connection attempt.
    ///
    /// Tears down the current connection (if any), resets the reconnect
    /// counter, and spawns a new background task.
    pub async fn restart_connection(&mut self) {
        if self.ready_state() == ReadyState::Connecting {
            tracing::info!("Already connecting, skipping restart");
            return;
        }

        tracing::info!("Manual reconnection requested");
        self.disconnect().await.ok();
        self.connect().await.ok();
    }

    /// Remove authenticated subscriptions (e.g. User channel) from tracking.
    pub fn clear_authed_subscriptions(&self) {
        if let Some(tx) = &self.cmd_tx {
            let _ = tx.try_send(Command::ClearAuthedSubs);
        }
    }

    /// Get a stream of events from the WebSocket connection.
    ///
    /// The returned stream borrows `self`, so it must be dropped
    /// before calling `disconnect()`.
    pub fn events(&self) -> Pin<Box<dyn Stream<Item = WsEvent> + Send + '_>> {
        Box::pin(futures_util::stream::unfold(
            &self.event_rx,
            |rx| async move {
                let mut guard = rx.lock().await;
                guard.recv().await.map(|event| (event, rx))
            },
        ))
    }
}

impl Drop for WsClient {
    fn drop(&mut self) {
        if let Some(handle) = self.task_handle.take() {
            handle.abort();
        }
    }
}

// ─── Background task ─────────────────────────────────────────────────────────

async fn run_task(mut state: TaskState) {
    loop {
        // ── 1. Attempt connection ────────────────────────────────────────
        let (sink, stream) = match attempt_connect(&state.config.url).await {
            Ok(parts) => parts,
            Err(e) => {
                tracing::error!("WebSocket connection failed: {}", e);
                state.emit(WsEvent::Error(format!("Connection failed: {}", e)));

                if state.should_reconnect() {
                    backoff_sleep(&mut state, false).await;
                    // Drain any commands that arrived during backoff into pending
                    drain_commands_to_pending(&mut state);
                    continue;
                } else {
                    state.emit(WsEvent::MaxReconnectReached);
                    return;
                }
            }
        };

        // ── 2. Connected ─────────────────────────────────────────────────
        state.reconnect_attempts = 0;
        state.ready_state.store(ReadyState::Open as u16, Ordering::SeqCst);
        state.emit(WsEvent::Connected);

        // ── 3. Flush pending messages and resubscribe ────────────────────
        let mut sink = sink;
        flush_pending(&mut sink, &mut state.pending_messages).await;
        resubscribe_all(&mut sink, &state.active_subscriptions).await;

        // ── 4. Inner select! loop ────────────────────────────────────────
        let reason = run_connected(&mut state, sink, stream).await;

        // ── 5. Post-disconnect decision ──────────────────────────────────
        state.ready_state.store(ReadyState::Closed as u16, Ordering::SeqCst);

        match reason {
            DisconnectReason::UserRequested | DisconnectReason::NormalClose => return,
            DisconnectReason::RateLimited => {
                if state.should_reconnect() {
                    state.ready_state.store(ReadyState::Connecting as u16, Ordering::SeqCst);
                    backoff_sleep(&mut state, true).await;
                    drain_commands_to_pending(&mut state);
                    continue;
                }
                state.emit(WsEvent::MaxReconnectReached);
                return;
            }
            DisconnectReason::PongTimeout | DisconnectReason::Error(_) => {
                if state.should_reconnect() {
                    state.ready_state.store(ReadyState::Connecting as u16, Ordering::SeqCst);
                    backoff_sleep(&mut state, false).await;
                    drain_commands_to_pending(&mut state);
                    continue;
                }
                state.emit(WsEvent::MaxReconnectReached);
                return;
            }
        }
    }
}

/// The inner connected loop — runs until the connection breaks.
async fn run_connected(
    state: &mut TaskState,
    mut sink: SplitSink<WsStream, Message>,
    mut stream: SplitStream<WsStream>,
) -> DisconnectReason {
    let ping_dur = Duration::from_millis(state.config.ping_interval_ms as u64);
    let pong_dur = Duration::from_millis(state.config.pong_timeout_ms as u64);

    let mut ping_interval = tokio::time::interval(ping_dur);
    ping_interval.reset(); // skip immediate first tick

    let mut pong_deadline: Option<tokio::time::Instant> = None;

    // Create a sleep future that we reset when a pong deadline is set.
    // When no deadline is active, we set it far in the future.
    let far_future = tokio::time::Instant::now() + Duration::from_secs(86400);
    let pong_sleep = tokio::time::sleep_until(far_future);
    tokio::pin!(pong_sleep);

    loop {
        tokio::select! {
            // ── a) Incoming WS message ───────────────────────────────────
            msg = stream.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        let text_str: &str = text.as_ref();
                        match serde_json::from_str::<MessageIn>(text_str) {
                            Ok(msg_in) => {
                                if matches!(msg_in.kind, Kind::Pong(_)) {
                                    pong_deadline = None;
                                    pong_sleep.as_mut().reset(far_future);
                                    state.reconnect_attempts = 0;
                                }
                                state.emit(WsEvent::Message(msg_in.kind));
                            }
                            Err(e) => {
                                tracing::warn!(
                                    "WS deserialization error: {} — raw: {}",
                                    e,
                                    text_str
                                );
                                state.emit(WsEvent::Error(format!(
                                    "Deserialization error: {}",
                                    e
                                )));
                            }
                        }
                    }
                    Some(Ok(Message::Ping(data))) => {
                        let _ = sink.send(Message::Pong(data)).await;
                    }
                    Some(Ok(Message::Pong(_))) => {
                        // WS-level pong — harmless, ignore
                    }
                    Some(Ok(Message::Close(frame))) => {
                        let (code, reason) = extract_close(frame.as_ref());
                        state.emit(WsEvent::Disconnected {
                            code: Some(code),
                            reason: reason.clone(),
                        });
                        return match code {
                            1000 => DisconnectReason::NormalClose,
                            1008 => DisconnectReason::RateLimited,
                            _ => DisconnectReason::Error(reason),
                        };
                    }
                    Some(Ok(_)) => {} // Binary, Frame — ignore
                    Some(Err(e)) => {
                        let reason = e.to_string();
                        tracing::error!("WebSocket error: {}", reason);
                        state.emit(WsEvent::Disconnected {
                            code: None,
                            reason: reason.clone(),
                        });
                        return DisconnectReason::Error(reason);
                    }
                    None => {
                        state.emit(WsEvent::Disconnected {
                            code: None,
                            reason: "Stream ended".into(),
                        });
                        return DisconnectReason::Error("Stream ended".into());
                    }
                }
            }

            // ── b) Command from public API ───────────────────────────────
            cmd = state.cmd_rx.recv() => {
                match cmd {
                    Some(Command::Send(msg_out)) => {
                        track_subscription(
                            &mut state.active_subscriptions,
                            &msg_out,
                        );
                        if let Err(e) = send_msg(&mut sink, &msg_out).await {
                            tracing::warn!("Send failed: {}", e);
                        }
                    }
                    Some(Command::ClearAuthedSubs) => {
                        let before = state.active_subscriptions.len();
                        state.active_subscriptions.retain(|s| {
                            !matches!(s, SubscribeParams::User { .. })
                        });
                        let removed = before - state.active_subscriptions.len();
                        if removed > 0 {
                            tracing::info!("Cleared {} authenticated subscription(s)", removed);
                        }
                    }
                    Some(Command::Disconnect) => {
                        let _ = sink.send(Message::Close(Some(CloseFrame {
                            code: CloseCode::Normal,
                            reason: "Client disconnect".into(),
                        }))).await;
                        return DisconnectReason::UserRequested;
                    }
                    None => {
                        // WsClient dropped — clean exit
                        return DisconnectReason::UserRequested;
                    }
                }
            }

            // ── c) Ping interval ─────────────────────────────────────────
            _ = ping_interval.tick() => {
                if let Err(e) = send_msg(&mut sink, &MessageOut::Ping).await {
                    tracing::warn!("Failed to send ping: {}", e);
                } else {
                    let deadline = tokio::time::Instant::now() + pong_dur;
                    pong_deadline = Some(deadline);
                    pong_sleep.as_mut().reset(deadline);
                }
            }

            // ── d) Pong timeout ──────────────────────────────────────────
            () = &mut pong_sleep, if pong_deadline.is_some() => {
                tracing::warn!(
                    "Pong timeout — no response within {}ms",
                    state.config.pong_timeout_ms
                );
                state.emit(WsEvent::Disconnected {
                    code: None,
                    reason: "Pong timeout".into(),
                });
                let _ = sink.close().await;
                return DisconnectReason::PongTimeout;
            }
        }
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Attempt to establish a WebSocket connection with a 30-second timeout.
async fn attempt_connect(
    url: &str,
) -> Result<(SplitSink<WsStream, Message>, SplitStream<WsStream>), String> {
    let (ws_stream, _) = tokio::time::timeout(Duration::from_secs(30), connect_async(url))
        .await
        .map_err(|_| "Connection timeout".to_string())?
        .map_err(|e| e.to_string())?;

    Ok(ws_stream.split())
}

/// Serialize and send a MessageOut over the sink.
async fn send_msg(
    sink: &mut SplitSink<WsStream, Message>,
    msg: &MessageOut,
) -> Result<(), String> {
    let json = serde_json::to_string(msg).map_err(|e| e.to_string())?;
    sink.send(Message::Text(json.into()))
        .await
        .map_err(|e| e.to_string())
}

/// Extract close code and reason from an optional CloseFrame.
fn extract_close(frame: Option<&CloseFrame>) -> (u16, String) {
    match frame {
        Some(f) => (f.code.into(), f.reason.to_string()),
        None => (1006, "No close frame".into()),
    }
}

// ─── Subscription tracking ──────────────────────────────────────────────────

fn track_subscription(subs: &mut Vec<SubscribeParams>, msg: &MessageOut) {
    match msg {
        MessageOut::Subscribe(params) => {
            if !subs.iter().any(|s| s == params) {
                tracing::debug!("Tracking subscription: {:?}", params);
                subs.push(params.clone());
            }
        }
        MessageOut::Unsubscribe(unsub_params) => {
            let before = subs.len();
            subs.retain(|s| !s.matches_unsubscribe(unsub_params));
            let removed = before - subs.len();
            if removed > 0 {
                tracing::debug!("Removed {} subscription(s) from tracking", removed);
            }
        }
        MessageOut::Ping => {}
    }
}

async fn resubscribe_all(
    sink: &mut SplitSink<WsStream, Message>,
    subs: &[SubscribeParams],
) {
    if subs.is_empty() {
        return;
    }
    tracing::info!("Resubscribing to {} tracked subscription(s)", subs.len());
    for sub in subs {
        let msg = MessageOut::Subscribe(sub.clone());
        if let Err(e) = send_msg(sink, &msg).await {
            tracing::warn!("Failed to resubscribe: {}", e);
        }
    }
}

// ─── Message queue ───────────────────────────────────────────────────────────

async fn flush_pending(
    sink: &mut SplitSink<WsStream, Message>,
    pending: &mut Vec<MessageOut>,
) {
    if pending.is_empty() {
        return;
    }
    tracing::info!("Flushing {} pending message(s)", pending.len());
    let messages = std::mem::take(pending);
    for msg in &messages {
        if let Err(e) = send_msg(sink, msg).await {
            tracing::warn!("Failed to flush pending message: {}", e);
        }
    }
}

/// Drain any commands that arrived during backoff into pending_messages.
fn drain_commands_to_pending(state: &mut TaskState) {
    while let Ok(cmd) = state.cmd_rx.try_recv() {
        match cmd {
            Command::Send(msg) => {
                track_subscription(&mut state.active_subscriptions, &msg);
                state.pending_messages.push(msg);
            }
            Command::ClearAuthedSubs => {
                state.active_subscriptions.retain(|s| {
                    !matches!(s, SubscribeParams::User { .. })
                });
            }
            Command::Disconnect => {
                return;
            }
        }
    }
}

// ─── Reconnection backoff ────────────────────────────────────────────────────

async fn backoff_sleep(state: &mut TaskState, rate_limited: bool) {
    state.reconnect_attempts += 1;

    let exp = (state.reconnect_attempts - 1).min(10);
    let base = state.config.base_reconnect_delay_ms.saturating_mul(1u32 << exp);

    let (jitter_max, cap) = if rate_limited {
        (1000u32, 300_000u32) // up to 5 minutes for rate limits
    } else {
        (500u32, 60_000u32) // up to 60 seconds normally
    };

    let jitter = rand::random::<u32>() % jitter_max;
    let delay = base.saturating_add(jitter).min(cap);

    tracing::info!(
        "Reconnect attempt {}/{} in {}ms{}",
        state.reconnect_attempts,
        state.config.max_reconnect_attempts,
        delay,
        if rate_limited { " (rate-limited)" } else { "" }
    );

    tokio::time::sleep(Duration::from_millis(delay as u64)).await;
}

// ─── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::OrderBookId;

    #[test]
    fn test_ws_client_new() {
        let client = WsClient::new(WsConfig::default());
        assert!(client.cmd_tx.is_none());
    }

    #[test]
    fn test_send_when_not_connected() {
        let client = WsClient::new(WsConfig::default());
        let result = client.send(MessageOut::Ping);
        assert!(matches!(result, Err(WsError::NotConnected)));
    }

    #[test]
    fn test_track_subscription_add() {
        let mut subs = Vec::new();
        let msg = MessageOut::subscribe_books(vec![OrderBookId::new("ob1")]);
        track_subscription(&mut subs, &msg);
        assert_eq!(subs.len(), 1);

        // Duplicate — should not add
        track_subscription(&mut subs, &msg);
        assert_eq!(subs.len(), 1);
    }

    #[test]
    fn test_track_subscription_remove() {
        let mut subs = Vec::new();
        let sub_msg = MessageOut::subscribe_books(vec![OrderBookId::new("ob1")]);
        track_subscription(&mut subs, &sub_msg);
        assert_eq!(subs.len(), 1);

        let unsub_msg = MessageOut::unsubscribe_books(vec![OrderBookId::new("ob1")]);
        track_subscription(&mut subs, &unsub_msg);
        assert_eq!(subs.len(), 0);
    }

    #[test]
    fn test_track_subscription_ping_noop() {
        let mut subs = Vec::new();
        track_subscription(&mut subs, &MessageOut::Ping);
        assert_eq!(subs.len(), 0);
    }

    #[test]
    fn test_extract_close_with_frame() {
        let frame = CloseFrame {
            code: CloseCode::Normal,
            reason: "goodbye".into(),
        };
        let (code, reason) = extract_close(Some(&frame));
        assert_eq!(code, 1000);
        assert_eq!(reason, "goodbye");
    }

    #[test]
    fn test_extract_close_no_frame() {
        let (code, reason) = extract_close(None);
        assert_eq!(code, 1006);
        assert_eq!(reason, "No close frame");
    }

    #[tokio::test]
    async fn test_disconnect_when_not_connected() {
        let mut client = WsClient::new(WsConfig::default());
        let result = client.disconnect().await;
        assert!(result.is_ok());
    }
}
