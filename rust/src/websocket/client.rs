//! Main WebSocket client implementation.
//!
//! Provides a WebSocket client for real-time data streaming.

use std::collections::HashMap;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use std::time::Duration;

use rand::Rng;

use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, Stream, StreamExt};
use pin_project_lite::pin_project;
use tokio::net::TcpStream;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{interval, Instant};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::tungstenite::protocol::CloseFrame;
use tokio_tungstenite::tungstenite::Message;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};

use solana_keypair::Keypair;

use crate::auth::{authenticate, AuthCredentials};

use crate::websocket::error::{WebSocketError, WsResult};
use crate::websocket::handlers::MessageHandler;
use crate::websocket::state::price::PriceHistoryKey;
use crate::websocket::state::{LocalOrderbook, PriceHistory, UserState};
use crate::websocket::subscriptions::SubscriptionManager;
use crate::network::DEFAULT_WS_URL;
use crate::websocket::types::{SubscribeParams, WsEvent, WsRequest};

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;
type WsSink = SplitSink<WsStream, Message>;
type WsSource = SplitStream<WsStream>;

/// Connection timeout duration for WebSocket connections
const CONNECTION_TIMEOUT: Duration = Duration::from_secs(30);

/// WebSocket client configuration
#[derive(Debug, Clone)]
pub struct WebSocketConfig {
    /// Number of reconnect attempts before giving up
    pub reconnect_attempts: u32,
    /// Base delay for exponential backoff (ms)
    pub base_delay_ms: u64,
    /// Maximum delay for exponential backoff (ms)
    pub max_delay_ms: u64,
    /// Interval for client ping (seconds)
    pub ping_interval_secs: u64,
    /// Timeout for pong response (seconds). Connection is considered dead if no pong received within this time.
    pub pong_timeout_secs: u64,
    /// Whether to automatically reconnect on disconnect
    pub auto_reconnect: bool,
    /// Whether to automatically re-subscribe after reconnect
    pub auto_resubscribe: bool,
    /// Optional authentication token for private user streams
    pub auth_token: Option<String>,
    /// Capacity of the event channel. Default: 1000
    pub event_channel_capacity: usize,
    /// Capacity of the command channel. Default: 100
    pub command_channel_capacity: usize,
}

impl Default for WebSocketConfig {
    fn default() -> Self {
        Self {
            reconnect_attempts: 10,
            base_delay_ms: 1000,
            max_delay_ms: 30000,
            ping_interval_secs: 30,
            pong_timeout_secs: 60,
            auto_reconnect: true,
            auto_resubscribe: true,
            auth_token: None,
            event_channel_capacity: 1000,
            command_channel_capacity: 100,
        }
    }
}

/// Connection state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Reconnecting,
    Disconnecting,
}

/// Internal command for the connection task
enum ConnectionCommand {
    Send(String),
    Disconnect,
    Ping,
}

pin_project! {
    /// Main WebSocket client for Lightcone
    ///
    /// # Example
    ///
    /// ```ignore
    /// use lightcone_sdk::websocket::*;
    /// use futures_util::StreamExt;
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), WebSocketError> {
    ///     let mut client = LightconeWebSocketClient::connect("ws://api.lightcone.xyz:8081/ws").await?;
    ///
    ///     client.subscribe_book_updates(vec!["market1:ob1".to_string()]).await?;
    ///
    ///     while let Some(event) = client.next().await {
    ///         match event {
    ///             WsEvent::BookUpdate { orderbook_id, is_snapshot } => {
    ///                 if let Some(book) = client.get_orderbook(&orderbook_id) {
    ///                     println!("Best bid: {:?}", book.best_bid());
    ///                 }
    ///             }
    ///             _ => {}
    ///         }
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub struct LightconeWebSocketClient {
        url: String,
        config: WebSocketConfig,
        state: ConnectionState,
        subscriptions: Arc<RwLock<SubscriptionManager>>,
        orderbooks: Arc<RwLock<HashMap<String, LocalOrderbook>>>,
        user_states: Arc<RwLock<HashMap<String, UserState>>>,
        price_histories: Arc<RwLock<HashMap<PriceHistoryKey, PriceHistory>>>,
        subscribed_user: Arc<RwLock<Option<String>>>,
        handler: Arc<MessageHandler>,
        cmd_tx: Option<mpsc::Sender<ConnectionCommand>>,
        #[pin]
        event_rx: mpsc::Receiver<WsEvent>,
        event_tx: mpsc::Sender<WsEvent>,
        reconnect_attempt: u32,
        connection_task_handle: Option<tokio::task::JoinHandle<()>>,
        auth_credentials: Option<AuthCredentials>,
    }
}

impl LightconeWebSocketClient {
    /// Connect to the default Lightcone WebSocket server.
    ///
    /// Uses the URL `wss://ws.lightcone.xyz/ws`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let client = LightconeWebSocketClient::connect_default().await?;
    /// client.subscribe_book_updates(vec!["ob1".to_string()]).await?;
    /// ```
    pub async fn connect_default() -> WsResult<Self> {
        Self::connect_with_config(DEFAULT_WS_URL, WebSocketConfig::default()).await
    }

    /// Connect to a WebSocket server with default configuration
    pub async fn connect(url: &str) -> WsResult<Self> {
        Self::connect_with_config(url, WebSocketConfig::default()).await
    }

    /// Connect to the default Lightcone WebSocket server with authentication.
    ///
    /// This method:
    /// 1. Authenticates with the Lightcone API using the provided keypair
    /// 2. Obtains an auth token
    /// 3. Connects to the WebSocket server with the auth token
    ///
    /// # Arguments
    ///
    /// * `keypair` - The Solana keypair for authentication
    ///
    /// # Example
    ///
    /// ```ignore
    /// use solana_keypair::Keypair;
    ///
    /// let keypair = Keypair::from_bytes(&keypair_bytes).unwrap();
    /// let client = LightconeWebSocketClient::connect_authenticated(&keypair).await?;
    /// client.subscribe_user(pubkey.to_string()).await?;
    /// ```
    pub async fn connect_authenticated(keypair: &Keypair) -> WsResult<Self> {
        Self::connect_authenticated_with_config(keypair, WebSocketConfig::default()).await
    }

    /// Connect to the default Lightcone WebSocket server with authentication and custom config.
    pub async fn connect_authenticated_with_config(
        keypair: &Keypair,
        mut config: WebSocketConfig,
    ) -> WsResult<Self> {
        // Authenticate and get credentials
        let credentials = authenticate(keypair).await?;
        config.auth_token = Some(credentials.auth_token.clone());

        Self::connect_with_config_and_credentials(
            DEFAULT_WS_URL,
            config,
            Some(credentials),
        )
        .await
    }

    /// Connect to a WebSocket server with a pre-obtained auth token.
    ///
    /// Use this if you already have an auth token from a previous authentication.
    ///
    /// # Arguments
    ///
    /// * `url` - The WebSocket URL to connect to
    /// * `auth_token` - The auth token obtained from authentication
    pub async fn connect_with_auth(url: &str, auth_token: String) -> WsResult<Self> {
        let trimmed = auth_token.trim();
        if trimmed.is_empty() {
            return Err(WebSocketError::InvalidAuthToken(
                "Auth token cannot be empty".to_string()
            ));
        }
        let config = WebSocketConfig {
            auth_token: Some(trimmed.to_string()),
            ..Default::default()
        };
        Self::connect_with_config(url, config).await
    }

    /// Connect to a WebSocket server with custom configuration
    pub async fn connect_with_config(url: &str, config: WebSocketConfig) -> WsResult<Self> {
        Self::connect_with_config_and_credentials(url, config, None).await
    }

    /// Internal method to connect with config and optional credentials
    async fn connect_with_config_and_credentials(
        url: &str,
        config: WebSocketConfig,
        auth_credentials: Option<AuthCredentials>,
    ) -> WsResult<Self> {
        let (event_tx, event_rx) = mpsc::channel(config.event_channel_capacity);

        let orderbooks = Arc::new(RwLock::new(HashMap::new()));
        let user_states = Arc::new(RwLock::new(HashMap::new()));
        let price_histories = Arc::new(RwLock::new(HashMap::new()));
        let subscribed_user = Arc::new(RwLock::new(None));
        let subscriptions = Arc::new(RwLock::new(SubscriptionManager::new()));

        let handler = Arc::new(MessageHandler::new(
            orderbooks.clone(),
            user_states.clone(),
            price_histories.clone(),
            subscribed_user.clone(),
        ));

        let mut client = Self {
            url: url.to_string(),
            config,
            state: ConnectionState::Disconnected,
            subscriptions,
            orderbooks,
            user_states,
            price_histories,
            subscribed_user,
            handler,
            cmd_tx: None,
            event_rx,
            event_tx,
            reconnect_attempt: 0,
            connection_task_handle: None,
            auth_credentials,
        };

        client.establish_connection().await?;
        Ok(client)
    }

    /// Establish the WebSocket connection
    async fn establish_connection(&mut self) -> WsResult<()> {
        self.state = ConnectionState::Connecting;

        // Build the WebSocket request, optionally with auth cookie
        let ws_stream = if let Some(ref auth_token) = self.config.auth_token {
            let mut request = self
                .url
                .as_str()
                .into_client_request()
                .map_err(|e| WebSocketError::InvalidUrl(e.to_string()))?;

            request.headers_mut().insert(
                "Cookie",
                format!("auth_token={}", auth_token)
                    .parse()
                    .map_err(|e| WebSocketError::Protocol(format!("Invalid cookie header: {}", e)))?,
            );

            let (stream, _) = tokio::time::timeout(CONNECTION_TIMEOUT, connect_async(request))
                .await
                .map_err(|_| WebSocketError::Timeout)?
                .map_err(WebSocketError::from)?;
            stream
        } else {
            let (stream, _) = tokio::time::timeout(CONNECTION_TIMEOUT, connect_async(&self.url))
                .await
                .map_err(|_| WebSocketError::Timeout)?
                .map_err(WebSocketError::from)?;
            stream
        };

        self.state = ConnectionState::Connected;
        self.reconnect_attempt = 0;

        let (sink, source) = ws_stream.split();
        let (cmd_tx, cmd_rx) = mpsc::channel(self.config.command_channel_capacity);
        self.cmd_tx = Some(cmd_tx);

        // Spawn the connection task
        let ctx = ConnectionContext {
            handler: self.handler.clone(),
            event_tx: self.event_tx.clone(),
            config: self.config.clone(),
            subscriptions: self.subscriptions.clone(),
            url: self.url.clone(),
        };

        let handle = tokio::spawn(connection_task(sink, source, cmd_rx, ctx));
        self.connection_task_handle = Some(handle);

        // Send connected event
        let _ = self.event_tx.send(WsEvent::Connected).await;

        Ok(())
    }

    /// Subscribe to orderbook updates
    pub async fn subscribe_book_updates(&mut self, orderbook_ids: Vec<String>) -> WsResult<()> {
        // Initialize state for each orderbook
        for id in &orderbook_ids {
            self.handler.init_orderbook(id).await;
        }

        // Track subscription
        self.subscriptions.write().await.add_book_update(orderbook_ids.clone());

        // Send subscribe request
        let params = SubscribeParams::book_update(orderbook_ids);
        self.send_subscribe(params).await
    }

    /// Subscribe to trade executions
    pub async fn subscribe_trades(&mut self, orderbook_ids: Vec<String>) -> WsResult<()> {
        self.subscriptions.write().await.add_trades(orderbook_ids.clone());
        let params = SubscribeParams::trades(orderbook_ids);
        self.send_subscribe(params).await
    }

    /// Subscribe to user events (requires authentication)
    pub async fn subscribe_user(&mut self, wallet_address: String) -> WsResult<()> {
        self.handler.init_user_state(&wallet_address).await;
        self.subscriptions.write().await.add_user(wallet_address.clone());
        let params = SubscribeParams::user(wallet_address);
        self.send_subscribe(params).await
    }

    /// Subscribe to price history
    pub async fn subscribe_price_history(
        &mut self,
        orderbook_id: String,
        resolution: String,
        include_ohlcv: bool,
    ) -> WsResult<()> {
        self.handler
            .init_price_history(&orderbook_id, &resolution, include_ohlcv)
            .await;
        self.subscriptions
            .write()
            .await
            .add_price_history(orderbook_id.clone(), resolution.clone(), include_ohlcv);
        let params = SubscribeParams::price_history(orderbook_id, resolution, include_ohlcv);
        self.send_subscribe(params).await
    }

    /// Subscribe to market events
    pub async fn subscribe_market(&mut self, market_pubkey: String) -> WsResult<()> {
        self.subscriptions.write().await.add_market(market_pubkey.clone());
        let params = SubscribeParams::market(market_pubkey);
        self.send_subscribe(params).await
    }

    /// Unsubscribe from orderbook updates
    pub async fn unsubscribe_book_updates(&mut self, orderbook_ids: Vec<String>) -> WsResult<()> {
        self.subscriptions.write().await.remove_book_update(&orderbook_ids);
        let params = SubscribeParams::book_update(orderbook_ids);
        self.send_unsubscribe(params).await
    }

    /// Unsubscribe from trades
    pub async fn unsubscribe_trades(&mut self, orderbook_ids: Vec<String>) -> WsResult<()> {
        self.subscriptions.write().await.remove_trades(&orderbook_ids);
        let params = SubscribeParams::trades(orderbook_ids);
        self.send_unsubscribe(params).await
    }

    /// Unsubscribe from user events
    pub async fn unsubscribe_user(&mut self, wallet_address: String) -> WsResult<()> {
        self.handler.clear_subscribed_user(&wallet_address).await;
        self.subscriptions.write().await.remove_user(&wallet_address);
        let params = SubscribeParams::user(wallet_address);
        self.send_unsubscribe(params).await
    }

    /// Subscribe to ticker (best bid/ask) updates
    pub async fn subscribe_ticker(&mut self, orderbook_ids: Vec<String>) -> WsResult<()> {
        self.subscriptions.write().await.add_ticker(orderbook_ids.clone());
        let params = SubscribeParams::ticker(orderbook_ids);
        self.send_subscribe(params).await
    }

    /// Unsubscribe from ticker updates
    pub async fn unsubscribe_ticker(&mut self, orderbook_ids: Vec<String>) -> WsResult<()> {
        self.subscriptions.write().await.remove_ticker(&orderbook_ids);
        let params = SubscribeParams::ticker(orderbook_ids);
        self.send_unsubscribe(params).await
    }

    /// Unsubscribe from price history
    pub async fn unsubscribe_price_history(
        &mut self,
        orderbook_id: String,
        resolution: String,
    ) -> WsResult<()> {
        self.subscriptions
            .write()
            .await
            .remove_price_history(&orderbook_id, &resolution);
        let params = SubscribeParams::price_history(orderbook_id, resolution, false);
        self.send_unsubscribe(params).await
    }

    /// Unsubscribe from market events
    pub async fn unsubscribe_market(&mut self, market_pubkey: String) -> WsResult<()> {
        self.subscriptions.write().await.remove_market(&market_pubkey);
        let params = SubscribeParams::market(market_pubkey);
        self.send_unsubscribe(params).await
    }

    /// Send a subscribe request
    async fn send_subscribe(&self, params: SubscribeParams) -> WsResult<()> {
        let request = WsRequest::subscribe(params);
        self.send_json(&request).await
    }

    /// Send an unsubscribe request
    async fn send_unsubscribe(&self, params: SubscribeParams) -> WsResult<()> {
        let request = WsRequest::unsubscribe(params);
        self.send_json(&request).await
    }

    /// Send a ping request
    pub async fn ping(&mut self) -> WsResult<()> {
        if let Some(tx) = &self.cmd_tx {
            tx.send(ConnectionCommand::Ping)
                .await
                .map_err(|_| WebSocketError::ChannelClosed)?;
        }
        Ok(())
    }

    /// Send a JSON message
    async fn send_json<T: serde::Serialize>(&self, msg: &T) -> WsResult<()> {
        let json = serde_json::to_string(msg)?;
        self.send_text(json).await
    }

    /// Send a text message
    async fn send_text(&self, text: String) -> WsResult<()> {
        if let Some(tx) = &self.cmd_tx {
            tx.send(ConnectionCommand::Send(text))
                .await
                .map_err(|_| WebSocketError::ChannelClosed)?;
            Ok(())
        } else {
            Err(WebSocketError::NotConnected)
        }
    }

    /// Disconnect from the server
    pub async fn disconnect(&mut self) -> WsResult<()> {
        self.state = ConnectionState::Disconnecting;

        // Send disconnect command to the connection task
        if let Some(tx) = self.cmd_tx.take() {
            let _ = tx.send(ConnectionCommand::Disconnect).await;
        }

        // Wait for the connection task to finish
        if let Some(handle) = self.connection_task_handle.take() {
            let _ = handle.await;
        }

        self.state = ConnectionState::Disconnected;
        Ok(())
    }

    /// Check if the connection task is still running
    pub fn is_task_running(&self) -> bool {
        self.connection_task_handle
            .as_ref()
            .map(|h| !h.is_finished())
            .unwrap_or(false)
    }

    /// Get the current connection state
    pub fn connection_state(&self) -> ConnectionState {
        self.state
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.state == ConnectionState::Connected
    }

    /// Check if the connection is authenticated
    pub fn is_authenticated(&self) -> bool {
        self.config.auth_token.is_some()
    }

    /// Get the authentication credentials if available
    pub fn auth_credentials(&self) -> Option<&AuthCredentials> {
        self.auth_credentials.as_ref()
    }

    /// Get the user's public key if authenticated
    pub fn user_pubkey(&self) -> Option<&str> {
        self.auth_credentials.as_ref().map(|c| c.user_pubkey.as_str())
    }

    /// Get a reference to the local orderbook state.
    pub async fn get_orderbook(&self, orderbook_id: &str) -> Option<LocalOrderbook> {
        let orderbooks = self.orderbooks.read().await;
        orderbooks.get(orderbook_id).cloned()
    }

    /// Get a reference to the local user state.
    pub async fn get_user_state(&self, user: &str) -> Option<UserState> {
        let states = self.user_states.read().await;
        states.get(user).cloned()
    }

    /// Get a reference to the price history state.
    pub async fn get_price_history(
        &self,
        orderbook_id: &str,
        resolution: &str,
    ) -> Option<PriceHistory> {
        let key = PriceHistoryKey::new(orderbook_id.to_string(), resolution.to_string());
        let histories = self.price_histories.read().await;
        histories.get(&key).cloned()
    }

    /// Get the WebSocket URL
    pub fn url(&self) -> &str {
        &self.url
    }

    /// Get the configuration
    pub fn config(&self) -> &WebSocketConfig {
        &self.config
    }
}

impl Stream for LightconeWebSocketClient {
    type Item = WsEvent;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let mut this = self.project();
        this.event_rx.poll_recv(cx)
    }
}

/// Shared context for the connection task
struct ConnectionContext {
    handler: Arc<MessageHandler>,
    event_tx: mpsc::Sender<WsEvent>,
    config: WebSocketConfig,
    subscriptions: Arc<RwLock<SubscriptionManager>>,
    url: String,
}

/// Connection task that handles the WebSocket connection
async fn connection_task(
    mut sink: WsSink,
    mut source: WsSource,
    mut cmd_rx: mpsc::Receiver<ConnectionCommand>,
    ctx: ConnectionContext,
) {
    let ConnectionContext {
        handler,
        event_tx,
        config,
        subscriptions,
        url,
    } = ctx;
    let ping_interval_duration = Duration::from_secs(config.ping_interval_secs);
    let pong_timeout_duration = Duration::from_secs(config.pong_timeout_secs);
    let mut ping_interval = interval(ping_interval_duration);
    ping_interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

    let mut reconnect_attempt = 0u32;
    let mut last_pong = Instant::now();
    let mut awaiting_pong = false;

    loop {
        tokio::select! {
            // Handle incoming WebSocket messages
            msg = source.next() => {
                match msg {
                    Some(Ok(Message::Text(text))) => {
                        let events = handler.handle_message(&text).await;
                        for event in events {
                            // Handle JSON pong - update timeout tracking
                            if matches!(event, WsEvent::Pong) {
                                last_pong = Instant::now();
                                awaiting_pong = false;
                            }

                            // Use try_send to avoid blocking the connection task if consumer is slow
                            match event_tx.try_send(event) {
                                Ok(_) => {}
                                Err(mpsc::error::TrySendError::Full(dropped_event)) => {
                                    tracing::warn!(
                                        "Event channel full, dropping event: {:?}",
                                        std::mem::discriminant(&dropped_event)
                                    );
                                }
                                Err(mpsc::error::TrySendError::Closed(_)) => {
                                    tracing::debug!("Event receiver dropped");
                                    return;
                                }
                            }
                        }
                    }
                    Some(Ok(Message::Ping(data))) => {
                        if let Err(e) = sink.send(Message::Pong(data)).await {
                            tracing::warn!("Failed to send pong: {}", e);
                        }
                    }
                    Some(Ok(Message::Pong(_))) => {
                        // Received pong response - update tracking
                        last_pong = Instant::now();
                        awaiting_pong = false;
                        let _ = event_tx.send(WsEvent::Pong).await;
                    }
                    Some(Ok(Message::Close(frame))) => {
                        let close_code: u16 = frame.as_ref().map(|f| f.code.into()).unwrap_or(0);
                        let reason = frame
                            .as_ref()
                            .map(|f| format!("code: {}, reason: {}", f.code, f.reason))
                            .unwrap_or_else(|| "no reason".to_string());

                        tracing::info!("WebSocket closed: {}", reason);
                        let _ = event_tx.send(WsEvent::Disconnected { reason: reason.clone() }).await;

                        // Check if rate limited (close code 1008)
                        if close_code == 1008 {
                            let _ = event_tx.send(WsEvent::Error {
                                error: WebSocketError::RateLimited,
                            }).await;
                        }

                        // Try to reconnect if enabled
                        if config.auto_reconnect && reconnect_attempt < config.reconnect_attempts {
                            reconnect_attempt += 1;
                            let _ = event_tx.send(WsEvent::Reconnecting {
                                attempt: reconnect_attempt,
                            }).await;

                            // Full jitter: randomize between 0 and exponential delay to prevent thundering herd
                            let max_delay = config.base_delay_ms * 2u64.pow(reconnect_attempt.saturating_sub(1));
                            let jittered_delay = rand::thread_rng().gen_range(0..=max_delay);
                            let delay = jittered_delay.min(config.max_delay_ms);
                            tokio::time::sleep(Duration::from_millis(delay)).await;

                            // Try to reconnect
                            match reconnect(&url, &handler, &subscriptions, &config).await {
                                Ok((new_sink, new_source)) => {
                                    sink = new_sink;
                                    source = new_source;
                                    reconnect_attempt = 0;
                                    last_pong = Instant::now();
                                    awaiting_pong = false;
                                    let _ = event_tx.send(WsEvent::Connected).await;
                                }
                                Err(e) => {
                                    tracing::error!("Reconnect failed: {:?}", e);
                                    let _ = event_tx.send(WsEvent::Error { error: e }).await;
                                }
                            }
                        } else {
                            return;
                        }
                    }
                    Some(Ok(Message::Binary(_))) => {
                        // Ignore binary messages
                    }
                    Some(Ok(Message::Frame(_))) => {
                        // Ignore raw frames
                    }
                    Some(Err(e)) => {
                        tracing::error!("WebSocket error: {}", e);
                        let _ = event_tx.send(WsEvent::Error {
                            error: WebSocketError::from(e),
                        }).await;
                    }
                    None => {
                        tracing::info!("WebSocket stream ended");
                        let _ = event_tx.send(WsEvent::Disconnected {
                            reason: "Stream ended".to_string(),
                        }).await;
                        return;
                    }
                }
            }

            // Handle commands from the client
            cmd = cmd_rx.recv() => {
                match cmd {
                    Some(ConnectionCommand::Send(text)) => {
                        if let Err(e) = sink.send(Message::Text(text.into())).await {
                            tracing::warn!("Failed to send message: {}", e);
                        }
                    }
                    Some(ConnectionCommand::Ping) => {
                        let request = WsRequest::ping();
                        if let Ok(json) = serde_json::to_string(&request) {
                            if let Err(e) = sink.send(Message::Text(json.into())).await {
                                tracing::warn!("Failed to send ping: {}", e);
                            }
                        }
                    }
                    Some(ConnectionCommand::Disconnect) => {
                        let _ = sink.send(Message::Close(Some(CloseFrame {
                            code: tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode::Normal,
                            reason: "Client disconnect".into(),
                        }))).await;
                        return;
                    }
                    None => {
                        // Command channel closed
                        return;
                    }
                }
            }

            // Periodic ping with timeout check
            _ = ping_interval.tick() => {
                // Check if we're still waiting for a pong from the previous ping
                if awaiting_pong && last_pong.elapsed() > pong_timeout_duration {
                    tracing::warn!("Pong timeout: no response received within {:?}", pong_timeout_duration);
                    let _ = event_tx.send(WsEvent::Error {
                        error: WebSocketError::PingTimeout,
                    }).await;

                    // Treat this as a disconnect and try to reconnect
                    let _ = event_tx.send(WsEvent::Disconnected {
                        reason: "Ping timeout".to_string(),
                    }).await;

                    if config.auto_reconnect && reconnect_attempt < config.reconnect_attempts {
                        reconnect_attempt += 1;
                        let _ = event_tx.send(WsEvent::Reconnecting {
                            attempt: reconnect_attempt,
                        }).await;

                        let max_delay = config.base_delay_ms * 2u64.pow(reconnect_attempt.saturating_sub(1));
                        let jittered_delay = rand::thread_rng().gen_range(0..=max_delay);
                        let delay = jittered_delay.min(config.max_delay_ms);
                        tokio::time::sleep(Duration::from_millis(delay)).await;

                        match reconnect(&url, &handler, &subscriptions, &config).await {
                            Ok((new_sink, new_source)) => {
                                sink = new_sink;
                                source = new_source;
                                reconnect_attempt = 0;
                                last_pong = Instant::now();
                                awaiting_pong = false;
                                let _ = event_tx.send(WsEvent::Connected).await;
                            }
                            Err(e) => {
                                tracing::error!("Reconnect failed: {:?}", e);
                                let _ = event_tx.send(WsEvent::Error { error: e }).await;
                            }
                        }
                    } else {
                        return;
                    }
                } else {
                    // Send ping
                    let request = WsRequest::ping();
                    if let Ok(json) = serde_json::to_string(&request) {
                        if let Err(e) = sink.send(Message::Text(json.into())).await {
                            tracing::warn!("Failed to send periodic ping: {}", e);
                        } else {
                            awaiting_pong = true;
                        }
                    }
                }
            }
        }
    }
}

/// Reconnect to the WebSocket server
async fn reconnect(
    url: &str,
    handler: &Arc<MessageHandler>,
    subscriptions: &Arc<RwLock<SubscriptionManager>>,
    config: &WebSocketConfig,
) -> WsResult<(WsSink, WsSource)> {
    // Build the WebSocket request, optionally with auth cookie
    let ws_stream = if let Some(ref auth_token) = config.auth_token {
        let mut request = url
            .into_client_request()
            .map_err(|e| WebSocketError::InvalidUrl(e.to_string()))?;

        request.headers_mut().insert(
            "Cookie",
            format!("auth_token={}", auth_token)
                .parse()
                .map_err(|e| WebSocketError::Protocol(format!("Invalid cookie header: {}", e)))?,
        );

        let (stream, _) = tokio::time::timeout(CONNECTION_TIMEOUT, connect_async(request))
            .await
            .map_err(|_| WebSocketError::Timeout)?
            .map_err(WebSocketError::from)?;
        stream
    } else {
        let (stream, _) = tokio::time::timeout(CONNECTION_TIMEOUT, connect_async(url))
            .await
            .map_err(|_| WebSocketError::Timeout)?
            .map_err(WebSocketError::from)?;
        stream
    };

    let (mut sink, source) = ws_stream.split();

    // Clear state
    handler.clear_all().await;

    // Re-subscribe if enabled
    if config.auto_resubscribe {
        let subs = subscriptions.read().await.get_all_subscriptions();
        for sub in subs {
            let request = WsRequest::subscribe(sub.to_params());
            if let Ok(json) = serde_json::to_string(&request) {
                if let Err(e) = sink.send(Message::Text(json.into())).await {
                    tracing::warn!("Failed to re-subscribe after reconnect: {}", e);
                }
            }
        }
    }

    Ok((sink, source))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = WebSocketConfig::default();
        assert_eq!(config.reconnect_attempts, 10);
        assert_eq!(config.base_delay_ms, 1000);
        assert_eq!(config.max_delay_ms, 30000);
        assert_eq!(config.ping_interval_secs, 30);
        assert_eq!(config.pong_timeout_secs, 60);
        assert!(config.auto_reconnect);
        assert!(config.auto_resubscribe);
        assert_eq!(config.event_channel_capacity, 1000);
        assert_eq!(config.command_channel_capacity, 100);
    }

    #[test]
    fn test_backoff_calculation() {
        let config = WebSocketConfig::default();

        // First attempt
        let delay = config.base_delay_ms * 2u64.pow(0);
        assert_eq!(delay, 1000);

        // Second attempt
        let delay = config.base_delay_ms * 2u64.pow(1);
        assert_eq!(delay, 2000);

        // Third attempt
        let delay = config.base_delay_ms * 2u64.pow(2);
        assert_eq!(delay, 4000);

        // Should cap at max
        let delay = config.base_delay_ms * 2u64.pow(10);
        let capped = delay.min(config.max_delay_ms);
        assert_eq!(capped, 30000);
    }
}
