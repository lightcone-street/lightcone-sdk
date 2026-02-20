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
    pub order: OrderForSigning,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderForSigning {
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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignAndCancelOrderRequest {
    pub wallet_id: String,
    pub order_hash: String,
    pub maker: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignAndCancelAllRequest {
    pub wallet_id: String,
    pub user_pubkey: String,
    #[serde(default)]
    pub orderbook_id: String,
    pub timestamp: i64,
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
