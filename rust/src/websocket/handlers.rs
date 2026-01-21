//! Message handlers for WebSocket events.
//!
//! Routes incoming messages to appropriate handlers and emits events.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::websocket::error::WebSocketError;
use crate::websocket::state::{LocalOrderbook, PriceHistory, UserState};
use crate::websocket::state::price::PriceHistoryKey;
use crate::websocket::types::{
    BookUpdateData, ErrorData, MarketEventData, MessageType, PriceHistoryData, RawWsMessage,
    TradeData, UserEventData, WsEvent,
};

/// Handles incoming WebSocket messages
pub struct MessageHandler {
    /// Local orderbook state
    orderbooks: Arc<RwLock<HashMap<String, LocalOrderbook>>>,
    /// Local user state
    user_states: Arc<RwLock<HashMap<String, UserState>>>,
    /// Price history state
    price_histories: Arc<RwLock<HashMap<PriceHistoryKey, PriceHistory>>>,
    /// Currently subscribed user (single user per connection)
    subscribed_user: Arc<RwLock<Option<String>>>,
}

impl MessageHandler {
    /// Create a new message handler with shared state
    pub fn new(
        orderbooks: Arc<RwLock<HashMap<String, LocalOrderbook>>>,
        user_states: Arc<RwLock<HashMap<String, UserState>>>,
        price_histories: Arc<RwLock<HashMap<PriceHistoryKey, PriceHistory>>>,
        subscribed_user: Arc<RwLock<Option<String>>>,
    ) -> Self {
        Self {
            orderbooks,
            user_states,
            price_histories,
            subscribed_user,
        }
    }

    /// Handle an incoming message and return events
    pub async fn handle_message(&self, text: &str) -> Vec<WsEvent> {
        // Parse the raw message first
        let raw_msg: RawWsMessage = match serde_json::from_str(text) {
            Ok(msg) => msg,
            Err(e) => {
                tracing::warn!("Failed to parse WebSocket message: {}", e);
                return vec![WsEvent::Error {
                    error: WebSocketError::MessageParseError(e.to_string()),
                }];
            }
        };

        // Route by message type
        let msg_type = MessageType::from(raw_msg.type_.as_str());
        match msg_type {
            MessageType::BookUpdate => self.handle_book_update(&raw_msg).await,
            MessageType::Trades => self.handle_trade(&raw_msg).await,
            MessageType::User => self.handle_user_event(&raw_msg).await,
            MessageType::PriceHistory => self.handle_price_history(&raw_msg).await,
            MessageType::Market => self.handle_market_event(&raw_msg).await,
            MessageType::Error => self.handle_error(&raw_msg).await,
            MessageType::Pong => vec![WsEvent::Pong],
            MessageType::Unknown => {
                tracing::warn!("Unknown message type: {}", raw_msg.type_);
                vec![]
            }
        }
    }

    /// Handle book update message
    async fn handle_book_update(&self, raw_msg: &RawWsMessage) -> Vec<WsEvent> {
        let data: BookUpdateData = match serde_json::from_value(raw_msg.data.clone()) {
            Ok(data) => data,
            Err(e) => {
                tracing::warn!("Failed to parse book update: {}", e);
                return vec![WsEvent::Error {
                    error: WebSocketError::MessageParseError(e.to_string()),
                }];
            }
        };

        // Check for resync signal
        if data.resync {
            tracing::info!("Resync required for orderbook: {}", data.orderbook_id);
            return vec![WsEvent::ResyncRequired {
                orderbook_id: data.orderbook_id.clone(),
            }];
        }

        let orderbook_id = data.orderbook_id.clone();
        let is_snapshot = data.is_snapshot;

        // Update local state
        let mut orderbooks = self.orderbooks.write().await;
        let book = orderbooks
            .entry(orderbook_id.clone())
            .or_insert_with(|| LocalOrderbook::new(orderbook_id.clone()));

        match book.apply_update(&data) {
            Ok(()) => {
                vec![WsEvent::BookUpdate {
                    orderbook_id,
                    is_snapshot,
                }]
            }
            Err(WebSocketError::SequenceGap { expected, received }) => {
                tracing::warn!(
                    "Sequence gap in orderbook {}: expected {}, received {}",
                    orderbook_id,
                    expected,
                    received
                );
                // Clear the orderbook state on sequence gap
                book.clear();
                vec![WsEvent::ResyncRequired { orderbook_id }]
            }
            Err(e) => {
                vec![WsEvent::Error { error: e }]
            }
        }
    }

    /// Handle trade message
    async fn handle_trade(&self, raw_msg: &RawWsMessage) -> Vec<WsEvent> {
        let data: TradeData = match serde_json::from_value(raw_msg.data.clone()) {
            Ok(data) => data,
            Err(e) => {
                tracing::warn!("Failed to parse trade: {}", e);
                return vec![WsEvent::Error {
                    error: WebSocketError::MessageParseError(e.to_string()),
                }];
            }
        };

        vec![WsEvent::Trade {
            orderbook_id: data.orderbook_id.clone(),
            trade: data,
        }]
    }

    /// Handle user event message
    async fn handle_user_event(&self, raw_msg: &RawWsMessage) -> Vec<WsEvent> {
        let data: UserEventData = match serde_json::from_value(raw_msg.data.clone()) {
            Ok(data) => data,
            Err(e) => {
                tracing::warn!("Failed to parse user event: {}", e);
                return vec![WsEvent::Error {
                    error: WebSocketError::MessageParseError(e.to_string()),
                }];
            }
        };

        let event_type = data.event_type.clone();

        // Use the tracked subscribed user (single user per connection)
        let subscribed_user = self.subscribed_user.read().await;
        let user = subscribed_user
            .clone()
            .unwrap_or_else(|| "unknown".to_string());

        // Update local state for the subscribed user
        let mut user_states = self.user_states.write().await;
        if let Some(state) = user_states.get_mut(&user) {
            state.apply_event(&data);
        } else {
            tracing::warn!(
                "Received user event '{}' for user '{}' but no subscription exists. \
                 Call subscribe_user() before receiving events to avoid data loss.",
                event_type,
                user
            );
        }

        vec![WsEvent::UserUpdate { event_type, user }]
    }

    /// Handle price history message
    async fn handle_price_history(&self, raw_msg: &RawWsMessage) -> Vec<WsEvent> {
        let data: PriceHistoryData = match serde_json::from_value(raw_msg.data.clone()) {
            Ok(data) => data,
            Err(e) => {
                tracing::warn!("Failed to parse price history: {}", e);
                return vec![WsEvent::Error {
                    error: WebSocketError::MessageParseError(e.to_string()),
                }];
            }
        };

        // Heartbeats don't have orderbook_id
        if data.event_type == "heartbeat" {
            // Update all price histories with heartbeat
            let mut histories = self.price_histories.write().await;
            for history in histories.values_mut() {
                history.apply_heartbeat(&data);
            }
            return vec![];
        }

        let orderbook_id = match &data.orderbook_id {
            Some(id) => id.clone(),
            None => {
                tracing::warn!("Price history message missing orderbook_id");
                return vec![];
            }
        };

        let resolution = data.resolution.clone().unwrap_or_else(|| "1m".to_string());

        // Update local state
        let mut histories = self.price_histories.write().await;
        let key = PriceHistoryKey::new(orderbook_id.clone(), resolution.clone());

        if let Some(history) = histories.get_mut(&key) {
            history.apply_event(&data);
        } else {
            // Create new history if this is a snapshot
            if data.event_type == "snapshot" {
                let mut history = PriceHistory::new(
                    orderbook_id.clone(),
                    resolution.clone(),
                    data.include_ohlcv.unwrap_or(false),
                );
                history.apply_event(&data);
                histories.insert(key, history);
            } else {
                tracing::warn!(
                    "Received price history event '{}' for orderbook '{}' resolution '{}' \
                     but no subscription exists. Event dropped.",
                    data.event_type,
                    orderbook_id,
                    resolution
                );
            }
        }

        vec![WsEvent::PriceUpdate {
            orderbook_id,
            resolution,
        }]
    }

    /// Handle market event message
    async fn handle_market_event(&self, raw_msg: &RawWsMessage) -> Vec<WsEvent> {
        let data: MarketEventData = match serde_json::from_value(raw_msg.data.clone()) {
            Ok(data) => data,
            Err(e) => {
                tracing::warn!("Failed to parse market event: {}", e);
                return vec![WsEvent::Error {
                    error: WebSocketError::MessageParseError(e.to_string()),
                }];
            }
        };

        vec![WsEvent::MarketEvent {
            event_type: data.event_type,
            market_pubkey: data.market_pubkey,
        }]
    }

    /// Handle error message from server
    async fn handle_error(&self, raw_msg: &RawWsMessage) -> Vec<WsEvent> {
        let data: ErrorData = match serde_json::from_value(raw_msg.data.clone()) {
            Ok(data) => data,
            Err(e) => {
                tracing::warn!("Failed to parse error: {}", e);
                return vec![WsEvent::Error {
                    error: WebSocketError::MessageParseError(e.to_string()),
                }];
            }
        };

        tracing::error!("Server error: {} (code: {})", data.error, data.code);

        vec![WsEvent::Error {
            error: WebSocketError::ServerError {
                code: data.code,
                message: data.error,
            },
        }]
    }

    /// Initialize orderbook state for a subscription.
    ///
    /// Uses atomic entry API to avoid race conditions with message handlers.
    pub async fn init_orderbook(&self, orderbook_id: &str) {
        let mut orderbooks = self.orderbooks.write().await;
        orderbooks
            .entry(orderbook_id.to_string())
            .or_insert_with(|| LocalOrderbook::new(orderbook_id.to_string()));
    }

    /// Initialize user state for a subscription.
    ///
    /// Uses atomic entry API to avoid race conditions with message handlers.
    pub async fn init_user_state(&self, user: &str) {
        // Track the subscribed user
        *self.subscribed_user.write().await = Some(user.to_string());

        let mut user_states = self.user_states.write().await;
        user_states
            .entry(user.to_string())
            .or_insert_with(|| UserState::new(user.to_string()));
    }

    /// Clear the subscribed user
    pub async fn clear_subscribed_user(&self, user: &str) {
        let mut subscribed = self.subscribed_user.write().await;
        if subscribed.as_deref() == Some(user) {
            *subscribed = None;
        }
    }

    /// Initialize price history state for a subscription
    pub async fn init_price_history(
        &self,
        orderbook_id: &str,
        resolution: &str,
        include_ohlcv: bool,
    ) {
        let key = PriceHistoryKey::new(orderbook_id.to_string(), resolution.to_string());
        let mut histories = self.price_histories.write().await;
        histories.entry(key).or_insert_with(|| {
            PriceHistory::new(orderbook_id.to_string(), resolution.to_string(), include_ohlcv)
        });
    }

    /// Clear orderbook state
    pub async fn clear_orderbook(&self, orderbook_id: &str) {
        let mut orderbooks = self.orderbooks.write().await;
        if let Some(book) = orderbooks.get_mut(orderbook_id) {
            book.clear();
        }
    }

    /// Clear user state
    pub async fn clear_user_state(&self, user: &str) {
        let mut user_states = self.user_states.write().await;
        if let Some(state) = user_states.get_mut(user) {
            state.clear();
        }
    }

    /// Clear price history state
    pub async fn clear_price_history(&self, orderbook_id: &str, resolution: &str) {
        let key = PriceHistoryKey::new(orderbook_id.to_string(), resolution.to_string());
        let mut histories = self.price_histories.write().await;
        if let Some(history) = histories.get_mut(&key) {
            history.clear();
        }
    }

    /// Clear all state
    pub async fn clear_all(&self) {
        self.orderbooks.write().await.clear();
        self.user_states.write().await.clear();
        self.price_histories.write().await.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn create_handler() -> MessageHandler {
        MessageHandler::new(
            Arc::new(RwLock::new(HashMap::new())),
            Arc::new(RwLock::new(HashMap::new())),
            Arc::new(RwLock::new(HashMap::new())),
            Arc::new(RwLock::new(None)),
        )
    }

    #[tokio::test]
    async fn test_handle_book_update_snapshot() {
        let handler = create_handler().await;

        let msg = r#"{
            "type": "book_update",
            "version": 0.1,
            "data": {
                "orderbook_id": "ob1",
                "timestamp": "2024-01-01T00:00:00.000Z",
                "seq": 0,
                "bids": [{"side": "bid", "price": "0.500000", "size": "0.001000"}],
                "asks": [{"side": "ask", "price": "0.510000", "size": "0.000500"}],
                "is_snapshot": true
            }
        }"#;

        let events = handler.handle_message(msg).await;
        assert_eq!(events.len(), 1);

        match &events[0] {
            WsEvent::BookUpdate { orderbook_id, is_snapshot } => {
                assert_eq!(orderbook_id, "ob1");
                assert!(*is_snapshot);
            }
            _ => panic!("Expected BookUpdate event"),
        }
    }

    #[tokio::test]
    async fn test_handle_resync() {
        let handler = create_handler().await;

        let msg = r#"{
            "type": "book_update",
            "version": 0.1,
            "data": {
                "orderbook_id": "ob1",
                "resync": true,
                "message": "Please re-subscribe to get fresh snapshot"
            }
        }"#;

        let events = handler.handle_message(msg).await;
        assert_eq!(events.len(), 1);

        match &events[0] {
            WsEvent::ResyncRequired { orderbook_id } => {
                assert_eq!(orderbook_id, "ob1");
            }
            _ => panic!("Expected ResyncRequired event"),
        }
    }

    #[tokio::test]
    async fn test_handle_trade() {
        let handler = create_handler().await;

        let msg = r#"{
            "type": "trades",
            "version": 0.1,
            "data": {
                "orderbook_id": "ob1",
                "price": "0.505000",
                "size": "0.000250",
                "side": "bid",
                "timestamp": "2024-01-01T00:00:00.000Z",
                "trade_id": "trade123"
            }
        }"#;

        let events = handler.handle_message(msg).await;
        assert_eq!(events.len(), 1);

        match &events[0] {
            WsEvent::Trade { orderbook_id, trade } => {
                assert_eq!(orderbook_id, "ob1");
                assert_eq!(trade.price, "0.505000");
                assert_eq!(trade.size, "0.000250");
            }
            _ => panic!("Expected Trade event"),
        }
    }

    #[tokio::test]
    async fn test_handle_pong() {
        let handler = create_handler().await;

        let msg = r#"{
            "type": "pong",
            "version": 0.1,
            "data": {}
        }"#;

        let events = handler.handle_message(msg).await;
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], WsEvent::Pong));
    }

    #[tokio::test]
    async fn test_handle_error() {
        let handler = create_handler().await;

        let msg = r#"{
            "type": "error",
            "version": 0.1,
            "data": {
                "error": "Engine unavailable",
                "code": "ENGINE_UNAVAILABLE"
            }
        }"#;

        let events = handler.handle_message(msg).await;
        assert_eq!(events.len(), 1);

        match &events[0] {
            WsEvent::Error { error } => {
                match error {
                    WebSocketError::ServerError { code, message } => {
                        assert_eq!(code, "ENGINE_UNAVAILABLE");
                        assert_eq!(message, "Engine unavailable");
                    }
                    _ => panic!("Expected ServerError"),
                }
            }
            _ => panic!("Expected Error event"),
        }
    }

    #[tokio::test]
    async fn test_handle_invalid_json() {
        let handler = create_handler().await;

        let msg = "not valid json";

        let events = handler.handle_message(msg).await;
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0], WsEvent::Error { .. }));
    }
}
