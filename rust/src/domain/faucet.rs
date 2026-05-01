//! Testnet faucet — used by `LightconeClient::claim` to request testnet SOL and
//! whitelisted deposit tokens for a wallet.
//!
//! There is no sub-client for faucet; the single entry point is the
//! `claim(wallet_address)` method on the root client.

use crate::shared::PubkeyStr;
use serde::{Deserialize, Serialize};

/// Request payload for `POST /api/claim`.
#[derive(Debug, Clone, Serialize)]
pub struct FaucetRequest {
    pub wallet_address: PubkeyStr,
}

/// A single token minted to the wallet by the faucet.
#[derive(Debug, Clone, Deserialize)]
pub struct FaucetToken {
    pub symbol: String,
    pub amount: u64,
}

/// Response from `POST /api/claim`.
#[derive(Debug, Clone, Deserialize)]
pub struct FaucetResponse {
    /// Signature of the on-chain mint transaction.
    pub signature: String,
    /// Amount of testnet SOL transferred to the wallet.
    pub sol: f64,
    /// Per-token amounts minted (e.g. USDC, cbBTC).
    pub tokens: Vec<FaucetToken>,
}
