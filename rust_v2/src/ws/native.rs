//! Native WebSocket client — `tokio-tungstenite`.
//!
//! TODO: Full implementation with:
//! - tokio task for connection management
//! - ping/pong keepalive
//! - auto-reconnect with backoff
//! - subscription tracking + auto-resubscribe on reconnect
//! - message queue when disconnected

use crate::error::WsError;
use crate::ws::{MessageOut, SubscribeParams, UnsubscribeParams, WsConfig, WsEvent};
use futures_util::stream::Stream;
use std::pin::Pin;

/// Native WebSocket client using `tokio-tungstenite`.
pub struct WsClient {
    config: WsConfig,
    connected: bool,
}

impl WsClient {
    pub fn new(config: WsConfig) -> Self {
        Self {
            config,
            connected: false,
        }
    }

    pub async fn connect(&mut self) -> Result<(), WsError> {
        // TODO: Establish tokio-tungstenite connection
        self.connected = true;
        Ok(())
    }

    pub async fn disconnect(&mut self) -> Result<(), WsError> {
        self.connected = false;
        Ok(())
    }

    pub fn send(&self, msg: MessageOut) -> Result<(), WsError> {
        if !self.connected {
            return Err(WsError::NotConnected);
        }
        let _ = msg;
        Ok(())
    }

    pub fn subscribe(&self, params: SubscribeParams) -> Result<(), WsError> {
        self.send(MessageOut::Subscribe(params))
    }

    pub fn unsubscribe(&self, params: UnsubscribeParams) -> Result<(), WsError> {
        self.send(MessageOut::Unsubscribe(params))
    }

    pub fn events(&self) -> Pin<Box<dyn Stream<Item = WsEvent> + Send + '_>> {
        Box::pin(futures_util::stream::empty())
    }
}
