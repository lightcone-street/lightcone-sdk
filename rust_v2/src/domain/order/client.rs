//! Orders sub-client — submit, cancel, query.

use crate::client::LightconeClient;
use crate::error::SdkError;
use crate::http::RetryPolicy;
use serde::{Deserialize, Serialize};

// ─── Request types ───────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CancelBody {
    pub order_hash: String,
    pub maker: String,
    pub signature: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CancelAllBody {
    pub user_pubkey: String,
    #[serde(default)]
    pub orderbook_id: String,
    pub signature: String,
    pub timestamp: i64,
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
