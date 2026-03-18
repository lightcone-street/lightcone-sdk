//! Orders sub-client — submit, cancel, query, and on-chain order operations.

use super::wire::{UserSnapshotBalance, UserSnapshotOrder};
use crate::client::LightconeClient;
use crate::error::SdkError;
use crate::http::RetryPolicy;
use crate::program::error::{SdkError as ProgramSdkError, SdkResult};
use crate::program::instructions;
use crate::program::orders::OrderPayload;
use crate::shared::{OrderBookId, PubkeyStr};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;
use solana_signature::Signature;
use solana_transaction::Transaction;

#[cfg(feature = "native-auth")]
use solana_keypair::Keypair;
#[cfg(feature = "native-auth")]
use solana_signer::Signer;

// ─── Request types ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CancelBody {
    pub order_hash: String,
    pub maker: PubkeyStr,
    pub signature: String,
}

impl CancelBody {
    /// Build a cancel request with a base58-encoded signature (from a wallet adapter).
    /// Converts base58 to the hex encoding the backend expects.
    pub fn from_base58(order_hash: String, maker: PubkeyStr, sig_bs58: &str) -> SdkResult<Self> {
        let sig = sig_bs58
            .parse::<Signature>()
            .map_err(|_| ProgramSdkError::InvalidSignature)?;
        Ok(Self {
            order_hash,
            maker,
            signature: hex::encode(sig.as_ref()),
        })
    }

    /// Build a signed cancel request using a keypair.
    /// Signs `cancel_order_message(order_hash)` and hex-encodes the result.
    #[cfg(feature = "native-auth")]
    pub fn signed(order_hash: String, maker: PubkeyStr, keypair: &Keypair) -> Self {
        let message = crate::program::orders::cancel_order_message(&order_hash);
        let sig = keypair.sign_message(&message);
        Self {
            order_hash,
            maker,
            signature: hex::encode(sig.as_ref()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CancelAllBody {
    pub user_pubkey: PubkeyStr,
    #[serde(default)]
    pub orderbook_id: OrderBookId,
    pub signature: String,
    pub timestamp: i64,
    pub salt: String,
}

impl CancelAllBody {
    /// Build a cancel-all request with a base58-encoded signature (from a wallet adapter).
    /// Converts base58 to the hex encoding the backend expects.
    pub fn from_base58(
        user_pubkey: PubkeyStr,
        orderbook_id: OrderBookId,
        timestamp: i64,
        salt: String,
        sig_bs58: &str,
    ) -> SdkResult<Self> {
        let sig = sig_bs58
            .parse::<Signature>()
            .map_err(|_| ProgramSdkError::InvalidSignature)?;
        Ok(Self {
            user_pubkey,
            orderbook_id,
            signature: hex::encode(sig.as_ref()),
            timestamp,
            salt,
        })
    }

    /// Build a signed cancel-all request using a native keypair.
    /// Signs `cancel_all_message(user_pubkey, orderbook_id, timestamp, salt)` and hex-encodes the result.
    #[cfg(feature = "native-auth")]
    pub fn signed(
        user_pubkey: PubkeyStr,
        orderbook_id: OrderBookId,
        timestamp: i64,
        salt: String,
        keypair: &Keypair,
    ) -> Self {
        let message = crate::program::orders::cancel_all_message(
            user_pubkey.as_str(),
            orderbook_id.as_str(),
            timestamp,
            &salt,
        );
        let sig = keypair.sign_message(message.as_bytes());
        Self {
            user_pubkey,
            orderbook_id,
            signature: hex::encode(sig.as_ref()),
            timestamp,
            salt,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CancelTriggerBody {
    pub trigger_order_id: String,
    pub maker: PubkeyStr,
    pub signature: String,
}

impl CancelTriggerBody {
    /// Build a cancel-trigger request with a base58-encoded signature (from a wallet adapter).
    /// Converts base58 to the hex encoding the backend expects.
    pub fn from_base58(
        trigger_order_id: String,
        maker: PubkeyStr,
        sig_bs58: &str,
    ) -> SdkResult<Self> {
        let sig = sig_bs58
            .parse::<Signature>()
            .map_err(|_| ProgramSdkError::InvalidSignature)?;
        Ok(Self {
            trigger_order_id,
            maker,
            signature: hex::encode(sig.as_ref()),
        })
    }

    /// Build a signed cancel-trigger request using a native keypair.
    /// Signs `cancel_trigger_order_message(trigger_order_id)` and hex-encodes the result.
    #[cfg(feature = "native-auth")]
    pub fn signed(trigger_order_id: String, maker: PubkeyStr, keypair: &Keypair) -> Self {
        let message =
            crate::program::orders::cancel_trigger_order_message(&trigger_order_id);
        let sig = keypair.sign_message(&message);
        Self {
            trigger_order_id,
            maker,
            signature: hex::encode(sig.as_ref()),
        }
    }
}

// ─── Response types ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FillInfo {
    pub counterparty: PubkeyStr,
    pub counterparty_order_hash: String,
    pub fill_amount: Decimal,
    pub price: Decimal,
    pub is_maker: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubmitOrderResponse {
    pub order_hash: String,
    pub remaining: Decimal,
    pub filled: Decimal,
    pub fills: Vec<FillInfo>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum PlaceResponse {
    Accepted(SubmitOrderResponse),
    PartialFill(SubmitOrderResponse),
    Filled(SubmitOrderResponse),
    Rejected {
        error: Option<String>,
        details: Option<String>,
        order_hash: Option<String>,
        remaining: Option<Decimal>,
        filled: Option<Decimal>,
    },
    Error {
        error: Option<String>,
    },
    BadRequest {
        error: Option<String>,
        details: Option<String>,
    },
    NotFound {
        error: Option<String>,
    },
    Forbidden {
        error: Option<String>,
    },
    InternalError {
        error: Option<String>,
        details: Option<String>,
    },
    ConfigurationError {
        error: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CancelSuccess {
    pub order_hash: String,
    pub remaining: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum CancelResponse {
    Cancelled(CancelSuccess),
    Error { error: String },
    BadRequest { error: String },
    NotFound { error: String },
    Forbidden { error: String },
    InternalError {
        error: String,
        #[serde(default)]
        details: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CancelAllSuccess {
    pub cancelled_order_hashes: Vec<String>,
    pub count: u64,
    pub user_pubkey: PubkeyStr,
    pub orderbook_id: OrderBookId,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum CancelAllResponse {
    Success(CancelAllSuccess),
    Error { message: String },
    BadRequest { error: String },
    NotFound { error: String },
    Forbidden { error: String },
    InternalError {
        error: String,
        #[serde(default)]
        details: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TriggerOrderResponse {
    pub trigger_order_id: String,
    pub order_hash: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum TriggerSubmitResponse {
    Accepted(TriggerOrderResponse),
    Error {
        error: String,
    },
    BadRequest {
        error: String,
    },
    NotFound {
        error: String,
    },
    Forbidden {
        error: String,
    },
    InternalError {
        error: String,
        #[serde(default)]
        details: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CancelTriggerSuccess {
    pub trigger_order_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum CancelTriggerResponse {
    Cancelled(CancelTriggerSuccess),
    Error {
        error: String,
    },
    BadRequest {
        error: String,
    },
    NotFound {
        error: String,
    },
    Forbidden {
        error: String,
    },
    InternalError {
        error: String,
        #[serde(default)]
        details: Option<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserOrdersResponse {
    pub user_pubkey: PubkeyStr,
    /// All orders (both limit and trigger) in a single array.
    /// Discriminated by `order_type` field on each order.
    pub orders: Vec<UserSnapshotOrder>,
    pub balances: Vec<UserSnapshotBalance>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

// ─── Sub-client ──────────────────────────────────────────────────────────────

pub struct Orders<'a> {
    pub(crate) client: &'a LightconeClient,
}

impl<'a> Orders<'a> {
    // ── PDA helpers ──────────────────────────────────────────────────────

    /// Get the Order Status PDA.
    pub fn status_pda(&self, order_hash: &[u8; 32]) -> Pubkey {
        crate::program::pda::get_order_status_pda(order_hash, &self.client.program_id).0
    }

    /// Get the User Nonce PDA.
    pub fn nonce_pda(&self, user: &Pubkey) -> Pubkey {
        crate::program::pda::get_user_nonce_pda(user, &self.client.program_id).0
    }

    // ── Helpers ──────────────────────────────────────────────────────────

    /// Generate a random salt for cancel-all replay protection.
    pub fn generate_cancel_all_salt(&self) -> String {
        crate::program::orders::generate_cancel_all_salt()
    }

    pub async fn submit(
        &self,
        request: &impl serde::Serialize,
    ) -> Result<SubmitOrderResponse, SdkError> {
        let url = format!("{}/api/orders/submit", self.client.http.base_url());
        let raw: serde_json::Value = self
            .client
            .http
            .post(&url, request, RetryPolicy::None)
            .await?;

        let place: PlaceResponse = serde_json::from_value(raw.clone())
            .map_err(|e| SdkError::Other(format!("{e} — raw response: {raw}")))?;

        match place {
            PlaceResponse::Accepted(data)
            | PlaceResponse::PartialFill(data)
            | PlaceResponse::Filled(data) => Ok(data),
            PlaceResponse::Rejected {
                error,
                details,
                order_hash,
                remaining,
                filled,
            } => {
                let msg = match (error.as_deref(), details.as_deref()) {
                    (Some(e), Some(d)) => format!("{e}: {d}"),
                    (Some(e), None) => e.to_string(),
                    (None, Some(d)) => d.to_string(),
                    (None, None) => {
                        let mut parts = vec!["Rejected".to_string()];
                        if let Some(h) = &order_hash {
                            parts.push(format!("hash={h}"));
                        }
                        if let Some(f) = &filled {
                            parts.push(format!("filled={f}"));
                        }
                        if let Some(r) = &remaining {
                            parts.push(format!("remaining={r}"));
                        }
                        parts.join(", ")
                    }
                };
                Err(SdkError::Other(msg))
            }
            PlaceResponse::BadRequest { error, details }
            | PlaceResponse::InternalError { error, details } => {
                let msg = match (error.as_deref(), details.as_deref()) {
                    (Some(e), Some(d)) => format!("{e}: {d}"),
                    (Some(e), None) => e.to_string(),
                    (None, Some(d)) => d.to_string(),
                    (None, None) => format!("Unknown error — raw response: {raw}"),
                };
                Err(SdkError::Other(msg))
            }
            PlaceResponse::Error { error }
            | PlaceResponse::NotFound { error }
            | PlaceResponse::Forbidden { error }
            | PlaceResponse::ConfigurationError { error } => {
                Err(SdkError::Other(error.unwrap_or_else(|| format!("Unknown error — raw response: {raw}"))))
            }
        }
    }

    pub async fn cancel(&self, body: &CancelBody) -> Result<CancelSuccess, SdkError> {
        let url = format!("{}/api/orders/cancel", self.client.http.base_url());
        let raw: serde_json::Value = self
            .client
            .http
            .post(&url, body, RetryPolicy::None)
            .await?;

        let resp: CancelResponse = serde_json::from_value(raw)?;

        match resp {
            CancelResponse::Cancelled(data) => Ok(data),
            CancelResponse::InternalError { error, details } => {
                let msg = match details {
                    Some(d) => format!("{error}: {d}"),
                    None => error,
                };
                Err(SdkError::Other(msg))
            }
            CancelResponse::Error { error }
            | CancelResponse::BadRequest { error }
            | CancelResponse::NotFound { error }
            | CancelResponse::Forbidden { error } => Err(SdkError::Other(error)),
        }
    }

    pub async fn cancel_all(&self, body: &CancelAllBody) -> Result<CancelAllSuccess, SdkError> {
        let url = format!("{}/api/orders/cancel-all", self.client.http.base_url());
        let raw: serde_json::Value = self
            .client
            .http
            .post(&url, body, RetryPolicy::None)
            .await?;

        let resp: CancelAllResponse = serde_json::from_value(raw)?;

        match resp {
            CancelAllResponse::Success(data) => Ok(data),
            CancelAllResponse::Error { message } => Err(SdkError::Other(message)),
            CancelAllResponse::InternalError { error, details } => {
                let msg = match details {
                    Some(d) => format!("{error}: {d}"),
                    None => error,
                };
                Err(SdkError::Other(msg))
            }
            CancelAllResponse::BadRequest { error }
            | CancelAllResponse::NotFound { error }
            | CancelAllResponse::Forbidden { error } => Err(SdkError::Other(error)),
        }
    }

    pub async fn submit_trigger(
        &self,
        request: &impl serde::Serialize,
    ) -> Result<TriggerOrderResponse, SdkError> {
        let url = format!("{}/api/orders/submit", self.client.http.base_url());
        let raw: serde_json::Value = self
            .client
            .http
            .post(&url, request, RetryPolicy::None)
            .await?;

        let resp: TriggerSubmitResponse = serde_json::from_value(raw)?;

        match resp {
            TriggerSubmitResponse::Accepted(data) => Ok(data),
            TriggerSubmitResponse::InternalError { error, details } => {
                let msg = match details {
                    Some(d) => format!("{error}: {d}"),
                    None => error,
                };
                Err(SdkError::Other(msg))
            }
            TriggerSubmitResponse::Error { error }
            | TriggerSubmitResponse::BadRequest { error }
            | TriggerSubmitResponse::NotFound { error }
            | TriggerSubmitResponse::Forbidden { error } => Err(SdkError::Other(error)),
        }
    }

    pub async fn cancel_trigger(
        &self,
        body: &CancelTriggerBody,
    ) -> Result<CancelTriggerSuccess, SdkError> {
        let url = format!("{}/api/orders/cancel", self.client.http.base_url());
        let raw: serde_json::Value = self
            .client
            .http
            .post(&url, body, RetryPolicy::None)
            .await?;

        let resp: CancelTriggerResponse = serde_json::from_value(raw)?;

        match resp {
            CancelTriggerResponse::Cancelled(data) => Ok(data),
            CancelTriggerResponse::InternalError { error, details } => {
                let msg = match details {
                    Some(d) => format!("{error}: {d}"),
                    None => error,
                };
                Err(SdkError::Other(msg))
            }
            CancelTriggerResponse::Error { error }
            | CancelTriggerResponse::BadRequest { error }
            | CancelTriggerResponse::NotFound { error }
            | CancelTriggerResponse::Forbidden { error } => Err(SdkError::Other(error)),
        }
    }

    pub async fn get_user_orders(
        &self,
        wallet_address: &str,
        limit: Option<u32>,
        cursor: Option<&str>,
    ) -> Result<UserOrdersResponse, SdkError> {
        let mut url = format!(
            "{}/api/users/orders?wallet_address={}",
            self.client.http.base_url(),
            wallet_address
        );
        if let Some(limit) = limit {
            url.push_str(&format!("&limit={}", limit));
        }
        if let Some(cursor) = cursor {
            url.push_str(&format!("&cursor={}", cursor));
        }
        Ok(self
            .client
            .http
            .get(&url, RetryPolicy::Idempotent)
            .await?)
    }

    // ── On-chain instruction builders ───────────────────────────────────

    /// Build CancelOrder instruction (on-chain cancellation).
    pub fn cancel_order_ix(
        &self,
        maker: &Pubkey,
        market: &Pubkey,
        order: &OrderPayload,
    ) -> Instruction {
        let pid = &self.client.program_id;
        instructions::build_cancel_order_ix(maker, market, order, pid)
    }

    /// Build CancelOrder transaction (on-chain cancellation).
    pub fn cancel_order_tx(
        &self,
        maker: &Pubkey,
        market: &Pubkey,
        order: &OrderPayload,
    ) -> Result<Transaction, SdkError> {
        let ix = self.cancel_order_ix(maker, market, order);
        Ok(Transaction::new_with_payer(&[ix], Some(maker)))
    }

    /// Build IncrementNonce instruction.
    pub fn increment_nonce_ix(&self, user: &Pubkey) -> Instruction {
        let pid = &self.client.program_id;
        instructions::build_increment_nonce_ix(user, pid)
    }

    /// Build IncrementNonce transaction.
    pub fn increment_nonce_tx(&self, user: &Pubkey) -> Result<Transaction, SdkError> {
        let ix = self.increment_nonce_ix(user);
        Ok(Transaction::new_with_payer(&[ix], Some(user)))
    }

    // ── Order helpers ────────────────────────────────────────────────────

    /// Create an unsigned bid order.
    pub fn create_bid_order(
        &self,
        params: crate::program::types::BidOrderParams,
    ) -> OrderPayload {
        OrderPayload::new_bid(params)
    }

    /// Create an unsigned ask order.
    pub fn create_ask_order(
        &self,
        params: crate::program::types::AskOrderParams,
    ) -> OrderPayload {
        OrderPayload::new_ask(params)
    }

    /// Create and sign a bid order.
    #[cfg(feature = "native-auth")]
    pub fn create_signed_bid_order(
        &self,
        params: crate::program::types::BidOrderParams,
        keypair: &Keypair,
    ) -> OrderPayload {
        OrderPayload::new_bid_signed(params, keypair)
    }

    /// Create and sign an ask order.
    #[cfg(feature = "native-auth")]
    pub fn create_signed_ask_order(
        &self,
        params: crate::program::types::AskOrderParams,
        keypair: &Keypair,
    ) -> OrderPayload {
        OrderPayload::new_ask_signed(params, keypair)
    }

    /// Compute the hash of an order.
    pub fn hash_order(&self, order: &OrderPayload) -> [u8; 32] {
        order.hash()
    }

    /// Sign an order with the given keypair.
    #[cfg(feature = "native-auth")]
    pub fn sign_order(
        &self,
        order: &mut OrderPayload,
        keypair: &Keypair,
    ) {
        order.sign(keypair);
    }
}

// ═════════════════════════════════════════════════════════════════════════════
// On-chain account fetchers (require RPC)
// ═════════════════════════════════════════════════════════════════════════════

#[cfg(feature = "solana-rpc")]
impl<'a> Orders<'a> {
    /// Fetch an OrderStatus account (returns None if not found).
    pub async fn get_status(
        &self,
        order_hash: &[u8; 32],
    ) -> Result<Option<crate::program::accounts::OrderStatus>, SdkError> {
        let rpc = crate::rpc::require_solana_rpc(self.client)?;
        let pda = self.status_pda(order_hash);
        match rpc.get_account(&pda).await {
            Ok(account) => Ok(Some(
                crate::program::accounts::OrderStatus::deserialize(&account.data)?,
            )),
            Err(_) => Ok(None),
        }
    }

    /// Fetch a user's current nonce (returns 0 if not initialized).
    pub async fn get_nonce(&self, user: &Pubkey) -> Result<u64, SdkError> {
        let rpc = crate::rpc::require_solana_rpc(self.client)?;
        let pda = self.nonce_pda(user);
        match rpc.get_account(&pda).await {
            Ok(account) => {
                let user_nonce =
                    crate::program::accounts::UserNonce::deserialize(&account.data)?;
                Ok(user_nonce.nonce)
            }
            Err(_) => Ok(0),
        }
    }

    /// Get the current on-chain nonce for a user as u32.
    pub async fn current_nonce(&self, user: &Pubkey) -> Result<u32, SdkError> {
        let nonce = self.get_nonce(user).await?;
        u32::try_from(nonce)
            .map_err(|_| SdkError::Program(crate::program::error::SdkError::Overflow))
    }
}
