//! WASM WebSocket client — `web-sys::WebSocket`.
//!
//! TODO: Full implementation with:
//! - web-sys::WebSocket + wasm-bindgen-futures
//! - Browser handles protocol-level ping/pong
//! - futures-timer for reconnect delays
//! - Subscription tracking + auto-resubscribe on reconnect
//! - Message queue when disconnected

use crate::error::WsError;
use crate::ws::{MessageOut, SubscribeParams, UnsubscribeParams, WsConfig, WsEvent};

/// WASM WebSocket client using `web-sys::WebSocket`.
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
        // TODO: Create web-sys::WebSocket, attach event handlers
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
        // TODO: Send via web-sys::WebSocket
        let _ = msg;
        Ok(())
    }

    pub fn subscribe(&self, params: SubscribeParams) -> Result<(), WsError> {
        self.send(MessageOut::Subscribe { params })
    }

    pub fn unsubscribe(&self, params: UnsubscribeParams) -> Result<(), WsError> {
        self.send(MessageOut::Unsubscribe { params })
    }
}
