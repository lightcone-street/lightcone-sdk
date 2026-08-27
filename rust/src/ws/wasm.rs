//! WASM WebSocket client using `web-sys::WebSocket`.
//!
//! Full implementation with:
//! - `web-sys::WebSocket` + `wasm-bindgen` closures
//! - Application-level ping/pong health check
//! - Exponential backoff reconnection
//! - Subscription tracking + auto-resubscribe on reconnect
//! - Message queue when disconnected
//! - Callback-based event system (`on_event: impl Fn(WsEvent)`)

use std::cell::RefCell;

use futures_util::future::{AbortHandle, Abortable};
use futures_util::stream::StreamExt;
use gloo_timers::callback::Timeout;
use gloo_timers::future::IntervalStream;
use tracing;
use wasm_bindgen::prelude::*;
use web_sys::{CloseEvent, ErrorEvent, MessageEvent, WebSocket};

use crate::ws::subscriptions::Subscription;
use crate::ws::{
    Kind, MessageIn, MessageOut, ReadyState, SubscribeParams, UnsubscribeParams, WsConfig, WsEvent,
};

thread_local! {
    static WS: RefCell<Option<WebSocket>> = RefCell::new(None);
    static CONFIG: RefCell<Option<WsConfig>> = RefCell::new(None);
    static ON_EVENT: RefCell<Option<Box<dyn Fn(WsEvent)>>> = RefCell::new(None);
    static HEALTH_CHECK_ABORT: RefCell<Option<AbortHandle>> = RefCell::new(None);
    static PONG_TIMEOUT: RefCell<Option<Timeout>> = RefCell::new(None);
    static RECONNECT_TIMEOUT: RefCell<Option<Timeout>> = RefCell::new(None);
    static RECONNECT_ATTEMPTS: RefCell<u32> = RefCell::new(0);
    static RECONNECT_SCHEDULED: RefCell<bool> = RefCell::new(false);
    // When true the connection stays down: every reconnect path funnels
    // through schedule_reconnect(), which becomes a no-op until the next
    // explicit connect()/restart_connection() clears the flag. See stop().
    static STOPPED: RefCell<bool> = RefCell::new(false);
    static PENDING_MESSAGES: RefCell<Vec<MessageOut>> = RefCell::new(Vec::new());
    static ACTIVE_SUBSCRIPTIONS: RefCell<Vec<SubscribeParams>> = RefCell::new(Vec::new());
}

/// WASM WebSocket client — unit struct with static methods.
///
/// All state lives in `thread_local!` statics (WASM is single-threaded).
/// The consumer provides an `on_event` callback that receives `WsEvent`s
/// for connection lifecycle and incoming messages.
pub struct WsClient;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum SendDisposition {
    SendImmediately,
    QueueUntilOpen,
    QueueAndReconnect,
}

fn send_disposition(ready_state: ReadyState) -> SendDisposition {
    match ready_state {
        ReadyState::Open => SendDisposition::SendImmediately,
        ReadyState::Connecting => SendDisposition::QueueUntilOpen,
        ReadyState::Closing | ReadyState::Closed => SendDisposition::QueueAndReconnect,
    }
}

impl WsClient {
    /// Initialize and connect the WebSocket.
    ///
    /// The `on_event` callback will be called for every connection event
    /// (connected, disconnected, message, error, max reconnect reached).
    pub fn connect(config: WsConfig, on_event: impl Fn(WsEvent) + 'static) {
        STOPPED.with(|s| {
            let _ = s.try_borrow_mut().map(|mut v| *v = false);
        });
        CONFIG.with(|c| *c.borrow_mut() = Some(config));
        ON_EVENT.with(|cb| *cb.borrow_mut() = Some(Box::new(on_event)));
        Self::do_connect();
    }

    /// Send a message through the WebSocket.
    ///
    /// If open, sends immediately. While connecting, queues the message for
    /// the in-flight connection. If closing or closed, queues the message and
    /// triggers reconnection. Subscribe/unsubscribe messages are tracked for
    /// automatic resubscription on reconnect.
    pub fn send(message: MessageOut) {
        Self::track_subscription(&message);

        let ready_state = Self::ready_state();
        match send_disposition(ready_state) {
            SendDisposition::SendImmediately => {
                WS.with(|ws| match ws.try_borrow() {
                    Err(e) => {
                        tracing::error!("WebSocket borrow failed: {}", e);
                    }
                    Ok(ws_ref) => match ws_ref.as_ref() {
                        Some(w) => {
                            if let Err(e) = w.send_with_str(&message.to_string()) {
                                tracing::error!(
                                    "Failed to send message ({}): {}",
                                    message,
                                    extract_js_error(&e)
                                );
                            }
                        }
                        None => {
                            tracing::error!("WebSocket is open but no connection is available");
                        }
                    },
                });
            }
            SendDisposition::QueueUntilOpen => {
                tracing::info!("WebSocket is connecting; queueing message until it opens");
                Self::queue_message(message);
            }
            SendDisposition::QueueAndReconnect => {
                tracing::warn!(
                    "Cannot send message ({}) - WebSocket not open (state: {:?}). Triggering reconnect...",
                    message,
                    ready_state
                );
                Self::queue_message(message);
                Self::reconnect();
            }
        }
    }

    /// Subscribe to a channel.
    pub fn subscribe(params: SubscribeParams) {
        Self::send(MessageOut::Subscribe(params));
    }

    /// Unsubscribe from a channel.
    pub fn unsubscribe(params: UnsubscribeParams) {
        Self::send(MessageOut::Unsubscribe(params));
    }

    /// Force a fresh connection attempt.
    ///
    /// Closes any existing connection, cancels pending reconnection,
    /// resets the attempt counter, and initiates a new connection.
    pub fn restart_connection() {
        tracing::info!("Manual reconnection requested");
        STOPPED.with(|s| {
            let _ = s.try_borrow_mut().map(|mut v| *v = false);
        });
        // A browser WebSocket upgrade captures cookies when it is created. An
        // auth transition must therefore abort even a Connecting socket; letting
        // that pre-login handshake finish would keep the new session anonymous.
        Self::cleanup_connection();
        Self::cancel_reconnect();

        RECONNECT_ATTEMPTS.with(|a| {
            let _ = a.try_borrow_mut().map(|mut v| *v = 0);
        });
        RECONNECT_SCHEDULED.with(|s| {
            let _ = s.try_borrow_mut().map(|mut v| *v = false);
        });

        Self::do_connect();
    }

    /// Take the connection down and KEEP it down: closes the socket, cancels
    /// any pending reconnect, and suppresses every automatic reconnect path
    /// (close events, health checks, queued sends) until the next explicit
    /// [`connect`](Self::connect) or
    /// [`restart_connection`](Self::restart_connection).
    ///
    /// For callers whose credential teardown failed: the browser attaches
    /// cookies to the WS handshake automatically, so reconnecting after an
    /// incomplete logout would authenticate the socket as the user who just
    /// logged out. Stopping is the only way to guarantee that cannot happen.
    ///
    /// Messages already sitting in the send queue are DROPPED — they belong
    /// to the session being stopped and must not flush down the next
    /// session's socket. Tracked subscriptions deliberately survive: they
    /// describe the app's current interest (public books) and are replayed
    /// on the next explicit restart; session-scoped subscriptions must be
    /// pruned by the caller before stopping (see
    /// [`clear_authed_subscriptions`](Self::clear_authed_subscriptions)),
    /// the same way an app prunes them on logout.
    pub fn stop() {
        tracing::info!("WebSocket stopped; auto-reconnect suppressed until an explicit restart");
        STOPPED.with(|s| {
            let _ = s.try_borrow_mut().map(|mut v| *v = true);
        });
        Self::cancel_reconnect();
        RECONNECT_SCHEDULED.with(|s| {
            let _ = s.try_borrow_mut().map(|mut v| *v = false);
        });
        // Drop messages queued BEFORE the stop: they were addressed to the
        // session being torn down (e.g. an unsubscribe queued while the
        // socket was already down during logout) and would otherwise flush
        // down the next session's freshly authenticated socket at the next
        // explicit restart. Messages queued WHILE stopped are unaffected —
        // those are current app intent (e.g. book subscriptions from
        // browsing while signed out) and should flush after the restart.
        PENDING_MESSAGES.with(|pending| {
            if let Ok(mut queue) = pending.try_borrow_mut() {
                if !queue.is_empty() {
                    tracing::info!(
                        "Dropping {} queued message(s) with the stopped session",
                        queue.len()
                    );
                    queue.clear();
                }
            }
        });
        Self::cleanup_connection();
    }

    /// True after [`stop`](Self::stop), until the next explicit
    /// [`connect`](Self::connect) or
    /// [`restart_connection`](Self::restart_connection). App-level recovery
    /// flows (e.g. an auth-refresh loop that restarts the socket on success)
    /// must check this before restarting: their restart would otherwise
    /// cancel a deliberate stop and reconnect with whatever credentials the
    /// stop was protecting against.
    pub fn is_stopped() -> bool {
        STOPPED.with(|s| s.try_borrow().map(|v| *v).unwrap_or(false))
    }

    pub fn is_connected() -> bool {
        WS.with(|ws| {
            ws.try_borrow()
                .ok()
                .map(|ws_ref| {
                    ws_ref
                        .as_ref()
                        .map(|w| ReadyState::from(w.ready_state()) == ReadyState::Open)
                        .unwrap_or(false)
                })
                .unwrap_or(false)
        })
    }

    pub fn ready_state() -> ReadyState {
        WS.with(|ws| {
            ws.try_borrow()
                .ok()
                .map(|ws_ref| match ws_ref.as_ref() {
                    Some(w) => ReadyState::from(w.ready_state()),
                    None => ReadyState::Closed,
                })
                .unwrap_or(ReadyState::Closed)
        })
    }

    /// Purge authenticated user and wallet-balance replay state and queued messages.
    ///
    /// Cleanup is best-effort when browser callback state is already borrowed.
    /// It does not send wire unsubscriptions or close the active socket, so callers
    /// must unsubscribe or stop the connection to end current server delivery.
    pub fn clear_authed_subscriptions() {
        ACTIVE_SUBSCRIPTIONS.with(|subs| {
            if let Ok(mut subs_ref) = subs.try_borrow_mut() {
                let initial_len = subs_ref.len();
                subs_ref.retain(|sub| {
                    !matches!(
                        sub,
                        SubscribeParams::User { .. }
                            | SubscribeParams::WalletDepositBalances { .. }
                    )
                });
                let removed = initial_len - subs_ref.len();
                if removed > 0 {
                    tracing::info!("Cleared {} authenticated subscription(s)", removed);
                }
            }
        });
        // This is local bookkeeping only. Queued auth messages must not survive
        // logout and flush after replay tracking is cleared; live socket teardown
        // remains the caller's separate responsibility.
        PENDING_MESSAGES.with(|pending| {
            if let Ok(mut queue) = pending.try_borrow_mut() {
                queue.retain(|message| !message.is_authed_subscription());
            }
        });
    }

    /// Clean up all connection resources and reset state.
    pub fn cleanup() {
        Self::cleanup_connection();
        Self::cancel_reconnect();
        ON_EVENT.with(|cb| *cb.borrow_mut() = None);
        CONFIG.with(|c| *c.borrow_mut() = None);
    }

    // ── Internal ──────────────────────────────────────────────────────────

    fn emit(event: WsEvent) {
        ON_EVENT.with(|cb| {
            if let Ok(cb_ref) = cb.try_borrow() {
                if let Some(f) = cb_ref.as_ref() {
                    f(event);
                }
            }
        });
    }

    fn get_url() -> String {
        CONFIG.with(|c| {
            c.borrow()
                .as_ref()
                .map(|cfg| cfg.url.clone())
                .unwrap_or_else(|| crate::env::LightconeEnv::default().ws_url().to_string())
        })
    }

    fn get_config_val<T>(f: impl Fn(&WsConfig) -> T, default: T) -> T {
        CONFIG.with(|c| c.borrow().as_ref().map(|cfg| f(cfg)).unwrap_or(default))
    }

    fn do_connect() {
        match Self::ready_state() {
            ReadyState::Connecting | ReadyState::Open => {
                tracing::info!("Already connected or connecting, skipping");
                return;
            }
            _ => {}
        }

        tracing::info!("Creating WebSocket connection");

        let url = Self::get_url();
        match WebSocket::new(&url) {
            Err(err) => {
                tracing::error!("Failed to create WebSocket: {:?}", err);
                Self::emit(WsEvent::Error(format!(
                    "Failed to create WebSocket: {}",
                    extract_js_error(&err)
                )));
                Self::schedule_reconnect(false);
            }
            Ok(ws) => {
                Self::setup_connection(ws);
            }
        }
    }

    fn setup_connection(ws: WebSocket) {
        let onopen = Closure::<dyn FnMut()>::new(move || {
            tracing::info!("WebSocket opened");

            RECONNECT_SCHEDULED.with(|s| {
                let _ = s.try_borrow_mut().map(|mut f| *f = false);
            });

            Self::cancel_reconnect();
            Self::start_health_check();
            Self::flush_pending_messages();
            Self::resubscribe_all();
            Self::emit(WsEvent::Connected);
        });
        ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
        onopen.forget();

        let onmessage = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
            if let Ok(txt) = e.data().dyn_into::<js_sys::JsString>() {
                let txt: String = txt.into();

                match serde_json::from_str::<MessageIn>(&txt) {
                    Ok(msg) => {
                        if matches!(msg.kind, Kind::Pong(_)) {
                            Self::handle_pong_received();
                        }
                        Self::emit(WsEvent::Message(msg.kind));
                    }
                    Err(err) => {
                        tracing::error!("Failed to parse WS message: {:?} — raw: {}", err, txt);
                        Self::emit(WsEvent::Error(format!("Deserialization error: {}", err)));
                    }
                }
            }
        });
        ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        onmessage.forget();

        let onerror = Closure::<dyn FnMut(_)>::new(move |e: ErrorEvent| {
            let error = e.error();
            // Browser transport errors are followed by onclose, which owns
            // lifecycle details and reconnect behavior. Keep useful payloads
            // at one telemetry site instead of forwarding a duplicate event.
            if has_websocket_error_payload(error.is_undefined(), error.is_null()) {
                tracing::error!("WebSocket error: {}", extract_js_error(&error));
            } else {
                tracing::info!("WebSocket error had no diagnostic payload; awaiting close details");
            }
        });
        ws.set_onerror(Some(onerror.as_ref().unchecked_ref()));
        onerror.forget();

        let onclose = Closure::<dyn FnMut(_)>::new(move |e: CloseEvent| {
            let close_code = e.code();
            let reason = e.reason();
            tracing::info!("WebSocket closed: code={}, reason={}", close_code, reason);

            Self::cleanup_connection();
            Self::emit(WsEvent::Disconnected {
                code: Some(close_code),
                reason: reason.clone(),
            });

            if close_code != 1000 {
                let is_rate_limit = close_code == 1008;
                Self::schedule_reconnect(is_rate_limit);
            }
        });
        ws.set_onclose(Some(onclose.as_ref().unchecked_ref()));
        onclose.forget();

        WS.with(|ws_cell| {
            if let Ok(mut ws_ref) = ws_cell.try_borrow_mut() {
                *ws_ref = Some(ws);
            } else {
                tracing::error!("Could not store WebSocket - cell already borrowed");
            }
        });
    }

    // ── Health check (ping/pong) ──────────────────────────────────────────

    fn start_health_check() {
        Self::send(MessageOut::ping());
        Self::set_pong_timeout();

        HEALTH_CHECK_ABORT.with(|abort| {
            if let Ok(mut abort_ref) = abort.try_borrow_mut() {
                if let Some(handle) = abort_ref.take() {
                    handle.abort();
                }
            }
        });

        let ping_interval_ms = Self::get_config_val(|c| c.ping_interval_ms, 30_000);
        let (abort_handle, abort_reg) = AbortHandle::new_pair();

        wasm_bindgen_futures::spawn_local({
            let health_check = async move {
                let mut interval = IntervalStream::new(ping_interval_ms);

                while interval.next().await.is_some() {
                    if !WsClient::is_connected() {
                        tracing::info!("WebSocket not connected, stopping health check");
                        break;
                    }

                    WsClient::send(MessageOut::ping());
                    WsClient::set_pong_timeout();
                }
            };

            async move {
                let _ = Abortable::new(health_check, abort_reg).await;
            }
        });

        HEALTH_CHECK_ABORT.with(|abort| {
            if let Ok(mut abort_ref) = abort.try_borrow_mut() {
                *abort_ref = Some(abort_handle);
            }
        });
    }

    fn set_pong_timeout() {
        PONG_TIMEOUT.with(|timeout| {
            if let Ok(mut timeout_ref) = timeout.try_borrow_mut() {
                timeout_ref.take();

                let pong_timeout_ms = Self::get_config_val(|c| c.pong_timeout_ms, 10_000);
                *timeout_ref = Some(Timeout::new(pong_timeout_ms, || {
                    tracing::info!("Pong timeout - no response to ping");
                    WsClient::reconnect();
                }));
            }
        });
    }

    fn handle_pong_received() {
        PONG_TIMEOUT.with(|timeout| {
            if let Ok(mut timeout_ref) = timeout.try_borrow_mut() {
                timeout_ref.take();
            }
        });
        RECONNECT_ATTEMPTS.with(|a| {
            let _ = a.try_borrow_mut().map(|mut v| *v = 0);
        });
    }

    // ── Reconnection ──────────────────────────────────────────────────────

    fn reconnect() {
        tracing::info!("Handling WS disconnection");
        Self::cleanup_connection();
        Self::schedule_reconnect(false);
    }

    fn schedule_reconnect(is_rate_limit: bool) {
        let stopped = STOPPED.with(|s| s.try_borrow().map(|v| *v).unwrap_or(false));
        if stopped {
            tracing::info!("WebSocket is stopped; suppressing reconnect");
            return;
        }

        let already_scheduled = RECONNECT_SCHEDULED.with(|s| {
            s.try_borrow_mut()
                .map(|mut flag| {
                    if *flag {
                        true
                    } else {
                        *flag = true;
                        false
                    }
                })
                .unwrap_or(true)
        });

        if already_scheduled {
            tracing::info!("Reconnect already scheduled, skipping");
            return;
        }

        RECONNECT_ATTEMPTS.with(|attempts| {
            if let Ok(mut attempts_ref) = attempts.try_borrow_mut() {
                *attempts_ref += 1;

                let max_attempts = Self::get_config_val(|c| c.max_reconnect_attempts, 10);

                if *attempts_ref > max_attempts {
                    tracing::warn!("Max reconnection attempts ({}) exceeded", max_attempts);

                    RECONNECT_SCHEDULED.with(|s| {
                        let _ = s.try_borrow_mut().map(|mut f| *f = false);
                    });

                    Self::emit(WsEvent::MaxReconnectReached);
                    return;
                }

                let delay = if is_rate_limit {
                    calculate_backoff_delay(*attempts_ref, 1000, 300_000)
                } else {
                    calculate_backoff_delay(*attempts_ref, 500, 60_000)
                };

                tracing::info!(
                    "Scheduling reconnect attempt {} in {}ms (rate_limit: {})",
                    *attempts_ref,
                    delay,
                    is_rate_limit
                );

                RECONNECT_TIMEOUT.with(|timeout| {
                    if let Ok(mut timeout_ref) = timeout.try_borrow_mut() {
                        timeout_ref.take();

                        *timeout_ref = Some(Timeout::new(delay, || {
                            tracing::info!("Reconnect timeout fired");

                            RECONNECT_SCHEDULED.with(|s| {
                                let _ = s.try_borrow_mut().map(|mut f| *f = false);
                            });

                            WsClient::do_connect();
                        }));
                    }
                });
            }
        });
    }

    fn cancel_reconnect() {
        RECONNECT_TIMEOUT.with(|timeout| {
            if let Ok(mut timeout_ref) = timeout.try_borrow_mut() {
                timeout_ref.take();
            }
        });
    }

    // ── Connection cleanup ────────────────────────────────────────────────

    fn cleanup_connection() {
        HEALTH_CHECK_ABORT.with(|abort| {
            if let Ok(mut abort_ref) = abort.try_borrow_mut() {
                if let Some(handle) = abort_ref.take() {
                    handle.abort();
                }
            }
        });

        PONG_TIMEOUT.with(|timeout| {
            if let Ok(mut timeout_ref) = timeout.try_borrow_mut() {
                timeout_ref.take();
            }
        });

        WS.with(|ws| {
            if let Ok(mut ws_ref) = ws.try_borrow_mut() {
                if let Some(w) = ws_ref.take() {
                    w.set_onopen(None);
                    w.set_onmessage(None);
                    w.set_onerror(None);
                    w.set_onclose(None);

                    // Close Connecting sockets too, not just Open ones: a
                    // detached-but-unclosed handshake would complete in the
                    // background and leave the server with a live (possibly
                    // authenticated) connection nobody owns — close() during
                    // CONNECTING aborts the handshake per the WebSocket spec.
                    if matches!(
                        ReadyState::from(w.ready_state()),
                        ReadyState::Open | ReadyState::Connecting
                    ) {
                        let _ = w.close();
                    }
                }
            }
        });
    }

    // ── Message queue ─────────────────────────────────────────────────────

    fn queue_message(message: MessageOut) {
        PENDING_MESSAGES.with(|queue| {
            if let Ok(mut q) = queue.try_borrow_mut() {
                tracing::info!("Queueing message for delivery when connected: {}", message);
                q.push(message);
                tracing::info!("Queue size: {} messages", q.len());
            } else {
                tracing::error!("Failed to queue message - queue already borrowed");
            }
        });
    }

    fn flush_pending_messages() {
        PENDING_MESSAGES.with(|queue| {
            if let Ok(mut q) = queue.try_borrow_mut() {
                if q.is_empty() {
                    return;
                }

                tracing::info!("Flushing {} pending messages", q.len());
                let messages = std::mem::take(&mut *q);
                drop(q);

                for msg in messages {
                    tracing::info!("Sending queued message: {}", msg);
                    Self::send(msg);
                }
            } else {
                tracing::warn!("Could not flush pending messages - queue already borrowed");
            }
        });
    }

    // ── Subscription tracking ─────────────────────────────────────────────

    fn track_subscription(message: &MessageOut) {
        match message {
            MessageOut::Subscribe(params) => {
                ACTIVE_SUBSCRIPTIONS.with(|subs| {
                    if let Ok(mut subs_ref) = subs.try_borrow_mut() {
                        let key = params.subscription_key();
                        if !subs_ref.iter().any(|sub| sub.subscription_key() == key) {
                            tracing::info!("Tracking subscription: {:?}", params);
                            subs_ref.push(params.clone());
                        }
                    }
                });
            }
            MessageOut::Unsubscribe(unsub_params) => {
                ACTIVE_SUBSCRIPTIONS.with(|subs| {
                    if let Ok(mut subs_ref) = subs.try_borrow_mut() {
                        let initial_len = subs_ref.len();
                        subs_ref.retain(|sub| !sub.matches_unsubscribe(unsub_params));
                        let removed = initial_len - subs_ref.len();
                        if removed > 0 {
                            tracing::info!("Removed {} subscription(s) from tracking", removed);
                        }
                    }
                });
            }
            MessageOut::Ping => {}
        }
    }

    fn resubscribe_all() {
        ACTIVE_SUBSCRIPTIONS.with(|subs| {
            if let Ok(subs_ref) = subs.try_borrow() {
                if subs_ref.is_empty() {
                    return;
                }

                tracing::info!(
                    "Resubscribing to {} tracked subscription(s)",
                    subs_ref.len()
                );

                let subscriptions = subs_ref.clone();
                drop(subs_ref);

                for sub in subscriptions {
                    let msg = MessageOut::Subscribe(sub.clone());
                    tracing::info!("Resubscribing: {:?}", sub);
                    Self::send_without_tracking(msg);
                }
            }
        });
    }

    fn send_without_tracking(message: MessageOut) {
        WS.with(|ws| match ws.try_borrow() {
            Err(e) => {
                tracing::error!("WebSocket borrow failed: {}", e);
            }
            Ok(ws_ref) => match ws_ref.as_ref() {
                Some(w) if ReadyState::from(w.ready_state()) == ReadyState::Open => {
                    if let Err(e) = w.send_with_str(&message.to_string()) {
                        tracing::error!(
                            "Failed to send message ({}): {}",
                            message,
                            extract_js_error(&e)
                        );
                    }
                }
                _ => {
                    let state = Self::ready_state();
                    tracing::warn!(
                        "Cannot send message ({}) - WebSocket not open (state: {:?}). Queueing...",
                        message,
                        state
                    );
                    Self::queue_message(message);
                }
            },
        })
    }
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Whether `ErrorEvent.error` contains diagnostic data worth extracting.
///
/// Browsers commonly expose null or undefined for WebSocket transport errors;
/// the subsequent close event carries the actionable connection details.
fn has_websocket_error_payload(is_undefined: bool, is_null: bool) -> bool {
    !is_undefined && !is_null
}

fn extract_js_error(err: &JsValue) -> String {
    if let Some(error) = err.dyn_ref::<js_sys::Error>() {
        let name = (|| error.name().as_string())().unwrap_or_else(|| "Error".to_string());
        let message = (|| error.message().as_string())().unwrap_or_else(|| "".to_string());

        if !message.is_empty() {
            return format!("{}: {}", name, message);
        } else {
            return name;
        }
    }

    if let Ok(json_str) = js_sys::JSON::stringify(err) {
        if let Some(s) = json_str.as_string() {
            if !s.is_empty() && s != "null" && s != "undefined" {
                return s;
            }
        }
    }

    if let Some(s) = err.as_string() {
        if !s.is_empty() {
            return s;
        }
    }

    if err.is_undefined() {
        return "undefined error".to_string();
    }

    if err.is_null() {
        return "null error".to_string();
    }

    "Unknown WebSocket error".to_string()
}

fn calculate_backoff_delay(attempt: u32, jitter_max: u32, cap: u32) -> u32 {
    let base_delay = CONFIG.with(|c| {
        c.borrow()
            .as_ref()
            .map(|cfg| cfg.base_reconnect_delay_ms)
            .unwrap_or(1_000)
    });

    let exp = (attempt - 1).min(10);
    let base = base_delay * (1 << exp);
    let jitter = (js_sys::Math::random() * jitter_max as f64) as u32;
    base.saturating_add(jitter).min(cap)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn send_disposition_matches_connection_state() {
        let cases = [
            (ReadyState::Open, SendDisposition::SendImmediately),
            (ReadyState::Connecting, SendDisposition::QueueUntilOpen),
            (ReadyState::Closing, SendDisposition::QueueAndReconnect),
            (ReadyState::Closed, SendDisposition::QueueAndReconnect),
        ];

        for (ready_state, expected_disposition) in cases {
            assert_eq!(send_disposition(ready_state), expected_disposition);
        }
    }

    #[test]
    fn websocket_error_payload_requires_present_value() {
        assert!(!has_websocket_error_payload(true, false));
        assert!(!has_websocket_error_payload(false, true));
        assert!(has_websocket_error_payload(false, false));
    }
}
