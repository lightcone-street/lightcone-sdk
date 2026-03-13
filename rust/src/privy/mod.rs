//! Privy sub-client — embedded wallet RPC operations.
//!
//! Embedded wallets are provisioned during login by passing
//! `use_embedded_wallet: true` to `login_with_message()`. This works on any
//! platform (WASM, native, CLI). Once provisioned, the wallet is tied to the
//! user's account and accessible through the Privy sub-client.
//!
//! Wraps the backend's Privy endpoints for embedded wallet operations:
//! - Sign and send Solana transactions
//! - Sign and submit orders (signs via Privy, submits to exchange engine)
//! - Export embedded wallet private key (HPKE encrypted)
//!
//! Native and CLI clients that sign transactions with their own keypair
//! typically do not need an embedded wallet, but provisioning and using one
//! is fully supported.

#[cfg(feature = "http")]
pub mod client;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignAndSendTxRequest {
    pub wallet_id: String,
    pub base64_tx: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignAndSendTxResponse {
    pub hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignAndSendOrderRequest {
    pub wallet_id: String,
    pub order: PrivyOrderEnvelope,
}

/// Wire type for the backend's Privy sign-and-send-order endpoint.
///
/// Matches the backend's `OrderForSigning` struct exactly.
/// Prefer using the `from_limit()` / `from_trigger()` constructors
/// over building this manually.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivyOrderEnvelope {
    pub maker: String,
    pub nonce: u64,
    pub market_pubkey: String,
    pub base_token: String,
    pub quote_token: String,
    pub side: u32,
    pub amount_in: u64,
    pub amount_out: u64,
    #[serde(default)]
    pub expiration: i64,
    pub orderbook_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "tif")]
    pub time_in_force: Option<crate::shared::TimeInForce>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger_price: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub trigger_type: Option<crate::shared::TriggerType>,
}

impl PrivyOrderEnvelope {
    /// Build from a `LimitOrderEnvelope`. Trigger fields are left `None`.
    pub fn from_limit(
        envelope: &crate::program::envelope::LimitOrderEnvelope,
        orderbook_id: impl Into<String>,
    ) -> Self {
        Self {
            maker: envelope
                .fields_maker()
                .expect("maker is required")
                .to_string(),
            nonce: envelope.fields_nonce().expect("nonce is required") as u64,
            market_pubkey: envelope
                .fields_market()
                .expect("market is required")
                .to_string(),
            base_token: envelope
                .fields_base_mint()
                .expect("base_mint is required")
                .to_string(),
            quote_token: envelope
                .fields_quote_mint()
                .expect("quote_mint is required")
                .to_string(),
            side: envelope.fields_side().expect("side is required") as u32,
            amount_in: envelope.fields_amount_in().expect("amount_in is required"),
            amount_out: envelope
                .fields_amount_out()
                .expect("amount_out is required"),
            expiration: envelope.fields_expiration(),
            orderbook_id: orderbook_id.into(),
            time_in_force: None,
            trigger_price: None,
            trigger_type: None,
        }
    }

    /// Build from a `TriggerOrderEnvelope`.
    pub fn from_trigger(
        envelope: &crate::program::envelope::TriggerOrderEnvelope,
        orderbook_id: impl Into<String>,
    ) -> Self {
        Self {
            maker: envelope
                .fields_maker()
                .expect("maker is required")
                .to_string(),
            nonce: envelope.fields_nonce().expect("nonce is required") as u64,
            market_pubkey: envelope
                .fields_market()
                .expect("market is required")
                .to_string(),
            base_token: envelope
                .fields_base_mint()
                .expect("base_mint is required")
                .to_string(),
            quote_token: envelope
                .fields_quote_mint()
                .expect("quote_mint is required")
                .to_string(),
            side: envelope.fields_side().expect("side is required") as u32,
            amount_in: envelope.fields_amount_in().expect("amount_in is required"),
            amount_out: envelope
                .fields_amount_out()
                .expect("amount_out is required"),
            expiration: envelope.fields_expiration(),
            orderbook_id: orderbook_id.into(),
            time_in_force: envelope.fields_time_in_force(),
            trigger_price: envelope.fields_trigger_price(),
            trigger_type: envelope.fields_trigger_type(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignAndCancelOrderRequest {
    pub wallet_id: String,
    pub maker: String,
    #[serde(flatten)]
    pub cancel: CancelTarget,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "cancel_type")]
pub enum CancelTarget {
    #[serde(rename = "limit")]
    Limit { order_hash: String },
    #[serde(rename = "trigger")]
    Trigger { trigger_order_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignAndCancelAllRequest {
    pub wallet_id: String,
    pub user_pubkey: String,
    #[serde(default)]
    pub orderbook_id: String,
    pub timestamp: i64,
    pub salt: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportWalletRequest {
    pub wallet_id: String,
    pub decode_pubkey_base64: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportWalletResponse {
    pub encryption_type: String,
    pub ciphertext: String,
    pub encapsulated_key: String,
}
