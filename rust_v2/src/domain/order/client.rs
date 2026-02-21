//! Orders sub-client — submit, cancel, query.

use crate::client::LightconeClient;
use crate::error::SdkError;
use crate::http::RetryPolicy;
use crate::program::error::{SdkError as ProgramSdkError, SdkResult};
use serde::{Deserialize, Serialize};
use solana_signature::Signature;

#[cfg(feature = "native-auth")]
use solana_keypair::Keypair;
#[cfg(feature = "native-auth")]
use solana_signer::Signer;

// ─── Request types ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CancelBody {
    pub order_hash: String,
    pub maker: String,
    pub signature: String,
}

impl CancelBody {
    /// Build a cancel request with a base58-encoded signature (from a wallet adapter).
    /// Converts base58 to the hex encoding the backend expects.
    pub fn from_base58(order_hash: String, maker: String, sig_bs58: &str) -> SdkResult<Self> {
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
    pub fn signed(order_hash: String, maker: String, keypair: &Keypair) -> Self {
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
    pub user_pubkey: String,
    #[serde(default)]
    pub orderbook_id: String,
    pub signature: String,
    pub timestamp: i64,
}

impl CancelAllBody {
    /// Build a cancel-all request with a base58-encoded signature (from a wallet adapter).
    /// Converts base58 to the hex encoding the backend expects.
    pub fn from_base58(
        user_pubkey: String,
        orderbook_id: String,
        timestamp: i64,
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
        })
    }

    /// Build a signed cancel-all request using a native keypair.
    /// Signs `cancel_all_message(user_pubkey, timestamp)` and hex-encodes the result.
    #[cfg(feature = "native-auth")]
    pub fn signed(
        user_pubkey: String,
        orderbook_id: String,
        timestamp: i64,
        keypair: &Keypair,
    ) -> Self {
        let message =
            crate::program::orders::cancel_all_message(&user_pubkey, timestamp);
        let sig = keypair.sign_message(message.as_bytes());
        Self {
            user_pubkey,
            orderbook_id,
            signature: hex::encode(sig.as_ref()),
            timestamp,
        }
    }
}

// ─── Response types ──────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FillInfo {
    pub counterparty: String,
    pub counterparty_order_hash: String,
    pub fill_amount: String,
    pub price: String,
    pub is_maker: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubmitOrderResponse {
    pub order_hash: String,
    pub remaining: String,
    pub filled: String,
    pub fills: Vec<FillInfo>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum PlaceResponse {
    Accepted(SubmitOrderResponse),
    PartialFill(SubmitOrderResponse),
    Filled(SubmitOrderResponse),
    Rejected {
        error: String,
        #[serde(default)]
        details: Option<String>,
    },
    BadRequest {
        error: String,
        #[serde(default)]
        details: Option<String>,
    },
    InternalError {
        error: String,
        #[serde(default)]
        details: Option<String>,
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
    Error { message: String },
    NotFound { error: String },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CancelAllSuccess {
    pub cancelled_order_hashes: Vec<String>,
    pub count: u64,
    pub user_pubkey: String,
    pub orderbook_id: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum CancelAllResponse {
    Success(CancelAllSuccess),
    Error { message: String },
}

// ─── Sub-client ──────────────────────────────────────────────────────────────

pub struct Orders<'a> {
    pub(crate) client: &'a LightconeClient,
}

impl<'a> Orders<'a> {
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

        let place: PlaceResponse = serde_json::from_value(raw)?;

        match place {
            PlaceResponse::Accepted(data)
            | PlaceResponse::PartialFill(data)
            | PlaceResponse::Filled(data) => Ok(data),
            PlaceResponse::Rejected { error, details }
            | PlaceResponse::BadRequest { error, details }
            | PlaceResponse::InternalError { error, details } => {
                let msg = match details {
                    Some(d) => format!("{}: {}", error, d),
                    None => error,
                };
                Err(SdkError::Other(msg))
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
            CancelResponse::Error { message } => Err(SdkError::Other(message)),
            CancelResponse::NotFound { error } => Err(SdkError::Other(error)),
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
        }
    }

    pub async fn get_user_orders(
        &self,
        request: &impl serde::Serialize,
    ) -> Result<serde_json::Value, SdkError> {
        let url = format!("{}/api/users/orders", self.client.http.base_url());
        Ok(self
            .client
            .http
            .post(&url, request, RetryPolicy::Idempotent)
            .await?)
    }
}
