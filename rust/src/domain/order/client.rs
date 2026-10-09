//! Orders sub-client — submit, cancel, query, and on-chain order operations.

use super::wire::{InitialCohort, OrderState, UserOrder, UserOrderFillsResponse};
use crate::client::LightconeClient;
use crate::domain::position::wire::FundingAccount;
use crate::error::SdkError;
use crate::http::RetryPolicy;
#[cfg(feature = "trigger_orders")]
use crate::program::envelope::TriggerOrderEnvelope;
use crate::program::envelope::{LimitOrderEnvelope, OrderEnvelope};
use crate::program::error::{SdkError as ProgramSdkError, SdkResult};
use crate::program::instructions;
use crate::program::orders::OrderPayload;
use crate::program::transaction::{V1Transaction, V1TransactionContext};
use crate::program::types::{CloseOrderStatusParams, OrderSide};
use crate::shared::{
    validate_raw_amounts, OrderBookId, OrderbookRules, PubkeyStr, SubmitOrderRequest,
};
#[cfg(feature = "trigger_orders")]
use crate::shared::{validate_trigger_price, SubmitTriggerOrderRequest};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;
use solana_signature::Signature;

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
    /// Unix seconds; must be within the engine's window (at most 30
    /// seconds) of server time.
    pub timestamp: i64,
    /// Replay-protection salt: non-empty, at most 128 bytes (the engine may
    /// allow fewer), never reused.
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

#[cfg(feature = "trigger_orders")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CancelTriggerBody {
    pub trigger_order_id: String,
    pub maker: PubkeyStr,
    pub signature: String,
}

#[cfg(feature = "trigger_orders")]
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
        let message = crate::program::orders::cancel_trigger_order_message(&trigger_order_id);
        let sig = keypair.sign_message(&message);
        Self {
            trigger_order_id,
            maker,
            signature: hex::encode(sig.as_ref()),
        }
    }
}

// ─── Response types ──────────────────────────────────────────────────────────

/// Outcome of an accepted order submission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubmitOrderStatus {
    /// Accepted; no fill awaits confirmation. The order may rest, or (IOC)
    /// its unfilled remainder is already cancelled.
    Accepted,
    /// Accepted with matched base awaiting on-chain confirmation, or the
    /// acceptance is durable but its committed view was not yet readable
    /// (then `state` and `initial_cohort` are `None`).
    AcceptedPending,
    /// The whole original base is confirmed filled.
    Filled,
    /// An accepted status this SDK version does not know.
    #[serde(other)]
    Unknown,
}

/// An authenticated fill captured with a submission response.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FillInfo {
    /// `"<execution_id>:<leg_index>:<projection_generation>"`, the same id as
    /// REST `trade_id`.
    pub fill_id: String,
    pub execution_id: String,
    pub counterparty: PubkeyStr,
    pub counterparty_order_hash: String,
    /// Filled size in base-token units.
    pub base_amount: Decimal,
    /// Filled notional in quote-token units.
    pub quote_amount: Decimal,
    /// Whether the submitted order was the maker of this fill.
    pub is_maker: bool,
    /// Signed fee estimate in raw atoms of `fee_mint` (negative = rebate).
    #[serde(with = "crate::shared::serde_util::i128_text")]
    pub fee_estimate_atoms: i128,
    /// Captured fee rate in basis points (negative = rebate).
    pub fee_bps: i16,
    pub fee_mint: PubkeyStr,
}

impl FillInfo {
    /// Fill price in quote units per base unit (`quote_amount / base_amount`).
    pub fn price(&self) -> Option<Decimal> {
        crate::shared::price::quote_per_base(self.quote_amount, self.base_amount)
    }
}

/// Response of `POST /api/orders/submit` for an accepted order.
///
/// Business rejections arrive as [`SdkError::ApiRejected`] with a
/// [`RejectionCode`](crate::shared::RejectionCode). An
/// `ENGINE_UNAVAILABLE` error means the outcome is unknown: see
/// [`Orders::submit`] for how to resolve it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SubmitOrderResponse {
    pub order_hash: String,
    pub status: SubmitOrderStatus,
    /// Committed cumulative state; `None` when the view is not yet readable.
    #[serde(default)]
    pub state: Option<OrderState>,
    /// Executions selected at acceptance; `None` alongside `state`.
    #[serde(default)]
    pub initial_cohort: Option<InitialCohort>,
    /// Up to 16 authenticated fills, oldest first.
    #[serde(default)]
    pub fills: Vec<FillInfo>,
    /// False when more fills exist or a newer revision prevented capture.
    #[serde(default)]
    pub fills_complete: bool,
    /// Continue with `Orders::get_order_fill_page` (order hash + this cursor).
    #[serde(default)]
    pub fills_next_cursor: Option<String>,
}

impl SubmitOrderResponse {
    /// Base filled by confirmed fills, when the committed view is available.
    pub fn filled_base(&self) -> Option<Decimal> {
        self.state.as_ref().map(|state| state.confirmed_base)
    }

    /// Base still resting on the book, when the committed view is available.
    pub fn open_base(&self) -> Option<Decimal> {
        self.state.as_ref().map(|state| state.open_base)
    }
}

/// Disposition of a single-order cancellation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CancelStatus {
    /// The open remainder was cancelled by this request.
    Cancelled,
    /// The order was already closed (cancelled, expired, or cut off).
    AlreadyClosed,
    /// The order was already completely filled.
    AlreadyFilled,
    /// A disposition this SDK version does not know.
    #[serde(other)]
    Unknown,
}

/// Cancellation quantities in raw base-token atoms (`quantity_unit` is
/// `"base_atoms"`), sent as integer strings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CancelQuantities {
    /// Base atoms cancelled by this request.
    #[serde(with = "crate::shared::serde_util::u64_text")]
    pub newly_cancelled_base: u64,
    /// Base atoms confirmed filled.
    #[serde(with = "crate::shared::serde_util::u64_text")]
    pub confirmed_base: u64,
    /// Base atoms matched and awaiting on-chain confirmation.
    #[serde(with = "crate::shared::serde_util::u64_text")]
    pub pending_base: u64,
    /// Base atoms still open after the request.
    #[serde(with = "crate::shared::serde_util::u64_text")]
    pub remaining_open_base: u64,
}

/// Response of `POST /api/orders/cancel`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CancelSuccess {
    pub status: CancelStatus,
    pub order_hash: String,
    /// Present on every successful cancellation (an unknown hash is the
    /// `ORDER_NOT_FOUND` rejection instead); `None` only if a backend omits
    /// it, never invented zeroes.
    #[serde(default)]
    pub quantities: Option<CancelQuantities>,
    /// Unit of `quantities`: always `"base_atoms"`.
    pub quantity_unit: String,
    /// Committed revision of the cancellation.
    #[serde(with = "crate::shared::serde_util::u64_text")]
    pub revision: u64,
    /// Why the order is closed; empty on the wire when none.
    #[serde(
        default,
        deserialize_with = "crate::shared::serde_util::deserialize_nonempty_string"
    )]
    pub closed_reason: Option<String>,
}

/// Committed accepted-order cutoff returned by cancel-all.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClosureAck {
    /// Closure operation id (UUID).
    pub operation_id: String,
    #[serde(with = "crate::shared::serde_util::u64_text")]
    pub committed_revision: u64,
    /// `"Wallet:<wallet>"` or `"WalletBook:<wallet>:<orderbook>"`.
    pub scope: String,
    /// Orders with `accepted_seq` at or below this are closed; `None` when
    /// the scope had no accepted orders.
    #[serde(default, with = "crate::shared::serde_util::opt_u64_text")]
    pub accepted_seq_cutoff: Option<u64>,
    /// True while bounded per-order cleanup is still running; individual
    /// order facts follow on the `user` channel.
    pub cleanup_pending: bool,
}

/// Response of `POST /api/orders/cancel-all`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CancelAllSuccess {
    /// Always `"success"`.
    pub status: String,
    pub user_pubkey: PubkeyStr,
    /// Empty for a wallet-wide cancel-all.
    #[serde(default)]
    pub orderbook_id: OrderBookId,
    #[serde(default)]
    pub closure: Option<ClosureAck>,
    pub message: String,
}

#[cfg(feature = "trigger_orders")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TriggerOrderResponse {
    pub trigger_order_id: String,
    pub order_hash: String,
}

#[cfg(feature = "trigger_orders")]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CancelTriggerSuccess {
    pub trigger_order_id: String,
}

/// Response of `GET /api/users/orders`: one page of the wallet's open and
/// pending orders plus one page of its funding accounts.
///
/// A page can be empty while `has_more` is true. Funding pages continue via
/// `positions().positions_page(None, next_funding_cursor, ..)`, which reads the
/// same committed funding view.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserOrdersResponse {
    pub user_pubkey: PubkeyStr,
    pub orders: Vec<UserOrder>,
    #[serde(default)]
    pub funding_accounts: Vec<FundingAccount>,
    /// Opaque order cursor (`"<accepted_seq>:<hash>"`).
    pub next_cursor: Option<String>,
    pub has_more: bool,
    #[serde(default)]
    pub next_funding_cursor: Option<String>,
    #[serde(default)]
    pub funding_has_more: bool,
    /// Database revision of this page; later pages may be newer.
    #[serde(with = "crate::shared::serde_util::u64_text")]
    pub committed_revision: u64,
    #[serde(with = "crate::shared::serde_util::u64_text")]
    pub projection_generation: u64,
}

/// Query for `GET /api/users[/{wallet}]/order-fills`. The backend rejects
/// unknown parameters, combining `cursor` with `order_hash`, and a
/// `fill_cursor` with `fills` of `0`.
fn order_fills_query(
    market_pubkey: Option<&str>,
    limit: Option<u32>,
    cursor: Option<&str>,
    order_hash: Option<&str>,
    fill_cursor: Option<&str>,
    fills: Option<u32>,
) -> Vec<(&'static str, String)> {
    let mut query = Vec::new();
    if let Some(market_pubkey) = market_pubkey {
        query.push(("market_pubkey", market_pubkey.to_string()));
    }
    if let Some(limit) = limit {
        query.push(("limit", limit.to_string()));
    }
    if let Some(cursor) = cursor {
        query.push(("cursor", cursor.to_string()));
    }
    if let Some(order_hash) = order_hash {
        query.push(("order_hash", order_hash.to_string()));
    }
    if let Some(fill_cursor) = fill_cursor {
        query.push(("fill_cursor", fill_cursor.to_string()));
    }
    if let Some(fills) = fills {
        query.push(("fills", fills.to_string()));
    }
    query
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

    // ── Envelope factories ────────────────────────────────────────────────

    /// Create a `LimitOrderEnvelope` pre-seeded with the client's deposit source.
    ///
    /// Users can still override the deposit source on the returned envelope
    /// by calling `.deposit_source()` before signing.
    pub async fn limit_order(&self) -> LimitOrderEnvelope {
        let deposit_source = self.client.deposit_source().await;
        LimitOrderEnvelope::new().deposit_source(deposit_source)
    }

    /// Create a `TriggerOrderEnvelope` pre-seeded with the client's deposit source.
    ///
    /// Users can still override the deposit source on the returned envelope
    /// by calling `.deposit_source()` before signing.
    #[cfg(feature = "trigger_orders")]
    pub async fn trigger_order(&self) -> TriggerOrderEnvelope {
        let deposit_source = self.client.deposit_source().await;
        TriggerOrderEnvelope::new().deposit_source(deposit_source)
    }

    // ── Helpers ──────────────────────────────────────────────────────────

    /// Generate a random salt for cancel-all replay protection.
    pub fn generate_cancel_all_salt(&self) -> String {
        crate::program::orders::generate_cancel_all_salt()
    }

    /// Submit a signed limit order. Not retried.
    ///
    /// An `ENGINE_UNAVAILABLE` error means the outcome is unknown. Resolve it
    /// by submitting the identical signed `request` again: a [`RejectionCode::DuplicateOrder`](crate::shared::RejectionCode::DuplicateOrder)
    /// rejection or an [`ErrorCode::AlreadyExists`](crate::shared::ErrorCode::AlreadyExists)
    /// error proves the first submission was accepted; any other result is the
    /// outcome of this one. Never sign a replacement with a new salt, which is
    /// a different order. Order reads list only open and pending orders, so
    /// they cannot prove an order was never accepted.
    pub async fn submit(
        &self,
        request: &SubmitOrderRequest,
    ) -> Result<SubmitOrderResponse, SdkError> {
        self.preflight_submit(request).await?;
        let url = format!("{}/api/orders/submit", self.client.http.base_url());
        self.client
            .http
            .post(&url, request, RetryPolicy::None)
            .await
    }

    pub async fn cancel(&self, body: &CancelBody) -> Result<CancelSuccess, SdkError> {
        let url = format!("{}/api/orders/cancel", self.client.http.base_url());
        self.client.http.post(&url, body, RetryPolicy::None).await
    }

    pub async fn cancel_all(&self, body: &CancelAllBody) -> Result<CancelAllSuccess, SdkError> {
        let url = format!("{}/api/orders/cancel-all", self.client.http.base_url());
        self.client.http.post(&url, body, RetryPolicy::None).await
    }

    /// Submit a signed trigger order.
    ///
    /// The current backend has no trigger orders and rejects this request.
    #[cfg(feature = "trigger_orders")]
    pub async fn submit_trigger(
        &self,
        request: &SubmitTriggerOrderRequest,
    ) -> Result<TriggerOrderResponse, SdkError> {
        let rules = self.preflight_submit(&request.order).await?;
        validate_trigger_price(request.trigger_price.as_str(), rules.price_decimals)
            .map_err(ProgramSdkError::from)?;
        let url = format!("{}/api/orders/submit", self.client.http.base_url());
        self.client
            .http
            .post(&url, request, RetryPolicy::None)
            .await
    }

    /// Check the signed amounts against the orderbook's fetched admission rules.
    async fn preflight_submit(
        &self,
        request: &SubmitOrderRequest,
    ) -> Result<OrderbookRules, SdkError> {
        let rules = self
            .client
            .orderbooks()
            .decimals(&request.orderbook_id)
            .await?;
        let side = match request.side {
            0 => OrderSide::Bid,
            1 => OrderSide::Ask,
            value => return Err(ProgramSdkError::InvalidSide(value as u8).into()),
        };
        validate_raw_amounts(request.amount_in, request.amount_out, side, &rules)
            .map_err(ProgramSdkError::from)?;
        Ok(rules)
    }

    #[cfg(feature = "trigger_orders")]
    pub async fn cancel_trigger(
        &self,
        body: &CancelTriggerBody,
    ) -> Result<CancelTriggerSuccess, SdkError> {
        let url = format!("{}/api/orders/cancel", self.client.http.base_url());
        self.client.http.post(&url, body, RetryPolicy::None).await
    }

    /// Fetch the authenticated user's open and pending orders, with a first
    /// page of funding accounts. Wallet is resolved server-side from the auth
    /// cookie, so no parameter is required. `limit` defaults to 200 and is
    /// clamped to 1..=256 server-side; `cursor` is a previous `next_cursor`.
    pub async fn get_user_orders(
        &self,
        limit: Option<u32>,
        cursor: Option<&str>,
    ) -> Result<UserOrdersResponse, SdkError> {
        let url = format!("{}/api/users/orders", self.client.http.base_url());
        let mut query = Vec::new();
        if let Some(limit) = limit {
            query.push(("limit", limit.to_string()));
        }
        if let Some(cursor) = cursor {
            query.push(("cursor", cursor.to_string()));
        }
        self.client
            .http
            .get_with_query(&url, &query, RetryPolicy::Idempotent)
            .await
    }

    /// Same as [`Self::get_user_orders`], but forwards the supplied raw `Cookie`
    /// header (`privy-token` and/or `lightcone-token`) for this call instead of
    /// the SDK's process-wide token store. Intended for server-side cookie
    /// forwarding (SSR / server functions).
    pub async fn get_user_orders_with_cookies(
        &self,
        limit: Option<u32>,
        cursor: Option<&str>,
        cookie_header: &str,
    ) -> Result<UserOrdersResponse, SdkError> {
        let url = format!("{}/api/users/orders", self.client.http.base_url());
        let mut query = Vec::new();
        if let Some(limit) = limit {
            query.push(("limit", limit.to_string()));
        }
        if let Some(cursor) = cursor {
            query.push(("cursor", cursor.to_string()));
        }
        self.client
            .http
            .get_with_cookies_and_query(&url, &query, RetryPolicy::Idempotent, cookie_header)
            .await
    }

    /// Fetch the authenticated user's filled orders with nested fill events.
    /// Wallet is resolved server-side from the auth cookie.
    ///
    /// Includes orders where the user was either maker or taker. Optionally
    /// filter by market. Orders are sorted by most recent fill first; each
    /// carries its oldest-first page of `fills` fills (default and maximum 16;
    /// `0` reads none and only reports `fills_has_more`). `limit` is clamped
    /// to 1..=100 server-side.
    pub async fn get_user_order_fills(
        &self,
        market_pubkey: Option<&str>,
        limit: Option<u32>,
        cursor: Option<&str>,
        fills: Option<u32>,
    ) -> Result<UserOrderFillsResponse, SdkError> {
        let url = format!("{}/api/users/order-fills", self.client.http.base_url());
        let query = order_fills_query(market_pubkey, limit, cursor, None, None, fills);
        self.client
            .http
            .get_with_query(&url, &query, RetryPolicy::Idempotent)
            .await
    }

    /// Same as [`Self::get_user_order_fills`], but forwards the supplied raw
    /// `Cookie` header (`privy-token` and/or `lightcone-token`) for this call
    /// instead of the SDK's process-wide token store. Intended for server-side
    /// cookie forwarding (SSR / server functions).
    pub async fn get_user_order_fills_with_cookies(
        &self,
        market_pubkey: Option<&str>,
        limit: Option<u32>,
        cursor: Option<&str>,
        fills: Option<u32>,
        cookie_header: &str,
    ) -> Result<UserOrderFillsResponse, SdkError> {
        let url = format!("{}/api/users/order-fills", self.client.http.base_url());
        let query = order_fills_query(market_pubkey, limit, cursor, None, None, fills);
        self.client
            .http
            .get_with_cookies_and_query(&url, &query, RetryPolicy::Idempotent, cookie_header)
            .await
    }

    /// Public variant of [`Self::get_user_order_fills`]. Takes the user's
    /// wallet via the URL path (`GET /api/users/{wallet}/order-fills`) and
    /// requires no auth.
    pub async fn get_user_order_fills_by_wallet(
        &self,
        wallet_address: &str,
        market_pubkey: Option<&str>,
        limit: Option<u32>,
        cursor: Option<&str>,
        fills: Option<u32>,
    ) -> Result<UserOrderFillsResponse, SdkError> {
        let url = format!(
            "{}/api/users/{}/order-fills",
            self.client.http.base_url(),
            wallet_address
        );
        let query = order_fills_query(market_pubkey, limit, cursor, None, None, fills);
        self.client
            .http
            .get_with_query(&url, &query, RetryPolicy::Idempotent)
            .await
    }

    /// Fetch one order of the authenticated user with the next page of its
    /// fills. Pass the order's `fills_next_cursor` (or a submission's
    /// `fills_next_cursor`) as `fill_cursor`; `None` starts at the first fill.
    /// `fills` is the page size (default and maximum 16; a `fill_cursor`
    /// needs at least 1). `order_hash` must be 64 lowercase hex characters.
    pub async fn get_order_fill_page(
        &self,
        order_hash: &str,
        fill_cursor: Option<&str>,
        fills: Option<u32>,
    ) -> Result<UserOrderFillsResponse, SdkError> {
        let url = format!("{}/api/users/order-fills", self.client.http.base_url());
        let query = order_fills_query(None, None, None, Some(order_hash), fill_cursor, fills);
        self.client
            .http
            .get_with_query(&url, &query, RetryPolicy::Idempotent)
            .await
    }

    /// Same as [`Self::get_order_fill_page`], forwarding the supplied raw
    /// `Cookie` header for server-side cookie forwarding.
    pub async fn get_order_fill_page_with_cookies(
        &self,
        order_hash: &str,
        fill_cursor: Option<&str>,
        fills: Option<u32>,
        cookie_header: &str,
    ) -> Result<UserOrderFillsResponse, SdkError> {
        let url = format!("{}/api/users/order-fills", self.client.http.base_url());
        let query = order_fills_query(None, None, None, Some(order_hash), fill_cursor, fills);
        self.client
            .http
            .get_with_cookies_and_query(&url, &query, RetryPolicy::Idempotent, cookie_header)
            .await
    }

    /// Public variant of [`Self::get_order_fill_page`] for any wallet.
    pub async fn get_order_fill_page_by_wallet(
        &self,
        wallet_address: &str,
        order_hash: &str,
        fill_cursor: Option<&str>,
        fills: Option<u32>,
    ) -> Result<UserOrderFillsResponse, SdkError> {
        let url = format!(
            "{}/api/users/{}/order-fills",
            self.client.http.base_url(),
            wallet_address
        );
        let query = order_fills_query(None, None, None, Some(order_hash), fill_cursor, fills);
        self.client
            .http
            .get_with_query(&url, &query, RetryPolicy::Idempotent)
            .await
    }

    // ── Unified cancel (dispatches based on client signing strategy) ────

    /// Cancel an order using the client's signing strategy.
    ///
    /// Signs the cancel message and submits the cancellation request.
    pub async fn cancel_order_signed(
        &self,
        order_hash: &str,
        maker: &PubkeyStr,
    ) -> Result<CancelSuccess, SdkError> {
        use crate::shared::signing::SigningStrategy;

        let strategy = self.client.signing_strategy().await.ok_or_else(|| {
            SdkError::Validation("signing strategy is not set on the client".into())
        })?;

        match strategy {
            #[cfg(feature = "native-auth")]
            SigningStrategy::Native(keypair) => {
                let body = CancelBody::signed(order_hash.to_string(), maker.clone(), &keypair);
                self.cancel(&body).await
            }
            SigningStrategy::WalletAdapter(signer) => {
                let message = crate::program::orders::cancel_order_message(order_hash);
                let sig_bytes = signer
                    .sign_message(&message)
                    .await
                    .map_err(crate::shared::signing::classify_signer_error)?;
                let sig_bs58 = bs58::encode(&sig_bytes).into_string();
                let body =
                    CancelBody::from_base58(order_hash.to_string(), maker.clone(), &sig_bs58)
                        .map_err(|error| SdkError::Program(error))?;
                self.cancel(&body).await
            }
        }
    }

    /// Cancel all orders using the client's signing strategy.
    ///
    /// Signs the cancel-all message and submits the cancellation request.
    pub async fn cancel_all_signed(
        &self,
        user_pubkey: &PubkeyStr,
        timestamp: i64,
        salt: &str,
        // Optional: limit to specific orderbook
        orderbook_id: Option<&OrderBookId>,
    ) -> Result<CancelAllSuccess, SdkError> {
        use crate::shared::signing::SigningStrategy;

        let strategy = self.client.signing_strategy().await.ok_or_else(|| {
            SdkError::Validation("signing strategy is not set on the client".into())
        })?;

        let resolved_orderbook_id = orderbook_id
            .cloned()
            .unwrap_or_else(|| OrderBookId::from(""));
        let orderbook_id_str = resolved_orderbook_id.as_str();

        match strategy {
            #[cfg(feature = "native-auth")]
            SigningStrategy::Native(keypair) => {
                let body = CancelAllBody::signed(
                    user_pubkey.clone(),
                    resolved_orderbook_id.clone(),
                    timestamp,
                    salt.to_string(),
                    &keypair,
                );
                self.cancel_all(&body).await
            }
            SigningStrategy::WalletAdapter(signer) => {
                let message = crate::program::orders::cancel_all_message(
                    user_pubkey.as_str(),
                    orderbook_id_str,
                    timestamp,
                    salt,
                );
                let sig_bytes = signer
                    .sign_message(message.as_bytes())
                    .await
                    .map_err(crate::shared::signing::classify_signer_error)?;
                let sig_bs58 = bs58::encode(&sig_bytes).into_string();
                let body = CancelAllBody::from_base58(
                    user_pubkey.clone(),
                    resolved_orderbook_id.clone(),
                    timestamp,
                    salt.to_string(),
                    &sig_bs58,
                )
                .map_err(|error| SdkError::Program(error))?;
                self.cancel_all(&body).await
            }
        }
    }

    /// Cancel a trigger order using the client's signing strategy.
    ///
    /// Signs the cancel message and submits the cancellation request.
    #[cfg(feature = "trigger_orders")]
    pub async fn cancel_trigger_signed(
        &self,
        trigger_order_id: &str,
        maker: &PubkeyStr,
    ) -> Result<CancelTriggerSuccess, SdkError> {
        use crate::shared::signing::SigningStrategy;

        let strategy = self.client.signing_strategy().await.ok_or_else(|| {
            SdkError::Validation("signing strategy is not set on the client".into())
        })?;

        match strategy {
            #[cfg(feature = "native-auth")]
            SigningStrategy::Native(keypair) => {
                let body = CancelTriggerBody::signed(
                    trigger_order_id.to_string(),
                    maker.clone(),
                    &keypair,
                );
                self.cancel_trigger(&body).await
            }
            SigningStrategy::WalletAdapter(signer) => {
                let message =
                    crate::program::orders::cancel_trigger_order_message(trigger_order_id);
                let sig_bytes = signer
                    .sign_message(&message)
                    .await
                    .map_err(crate::shared::signing::classify_signer_error)?;
                let sig_bs58 = bs58::encode(&sig_bytes).into_string();
                let body = CancelTriggerBody::from_base58(
                    trigger_order_id.to_string(),
                    maker.clone(),
                    &sig_bs58,
                )
                .map_err(|error| SdkError::Program(error))?;
                self.cancel_trigger(&body).await
            }
        }
    }

    // ── On-chain instruction builders ───────────────────────────────────

    /// Build CancelOrder instruction (on-chain cancellation).
    pub fn cancel_order_ix(
        &self,
        operator: &Pubkey,
        market: &Pubkey,
        order: &OrderPayload,
    ) -> Instruction {
        let pid = &self.client.program_id;
        instructions::build_cancel_order_ix(operator, market, order, pid)
    }

    /// Build CancelOrder transaction (on-chain cancellation).
    pub fn cancel_order_tx(
        &self,
        operator: &Pubkey,
        market: &Pubkey,
        order: &OrderPayload,
        context: &V1TransactionContext,
    ) -> Result<V1Transaction, SdkError> {
        let ix = self.cancel_order_ix(operator, market, order);
        Ok(V1Transaction::compile(&[ix], operator, context)?)
    }

    /// Build CloseOrderStatus instruction.
    pub fn close_order_status_ix(&self, params: &CloseOrderStatusParams) -> Instruction {
        let pid = &self.client.program_id;
        instructions::build_close_order_status_ix(params, pid)
    }

    /// Build CloseOrderStatus transaction.
    pub fn close_order_status_tx(
        &self,
        params: CloseOrderStatusParams,
        context: &V1TransactionContext,
    ) -> Result<V1Transaction, SdkError> {
        let ix = self.close_order_status_ix(&params);
        Ok(V1Transaction::compile(&[ix], &params.operator, context)?)
    }

    // ── Order helpers ────────────────────────────────────────────────────

    /// Create an unsigned bid order.
    pub fn create_bid_order(&self, params: crate::program::types::BidOrderParams) -> OrderPayload {
        OrderPayload::new_bid(params)
    }

    /// Create an unsigned ask order.
    pub fn create_ask_order(&self, params: crate::program::types::AskOrderParams) -> OrderPayload {
        OrderPayload::new_ask(params)
    }

    /// Create and sign a bid order.
    #[cfg(feature = "native-auth")]
    pub fn create_signed_bid_order(
        &self,
        params: crate::program::types::BidOrderParams,
        keypair: &Keypair,
        rules: &crate::shared::OrderbookRules,
    ) -> SdkResult<OrderPayload> {
        OrderPayload::new_bid_signed(params, keypair, rules)
    }

    /// Create and sign an ask order.
    #[cfg(feature = "native-auth")]
    pub fn create_signed_ask_order(
        &self,
        params: crate::program::types::AskOrderParams,
        keypair: &Keypair,
        rules: &crate::shared::OrderbookRules,
    ) -> SdkResult<OrderPayload> {
        OrderPayload::new_ask_signed(params, keypair, rules)
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
        rules: &crate::shared::OrderbookRules,
    ) -> SdkResult<()> {
        order.sign(keypair, rules)
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
        let rpc = crate::rpc::resolve_solana_rpc(self.client).await?;
        let pda = self.status_pda(order_hash);
        match rpc.get_account(&pda).await {
            Ok(account) => Ok(Some(crate::program::accounts::OrderStatus::deserialize(
                &account.data,
            )?)),
            Err(_) => Ok(None),
        }
    }
}

/// Deserialization of order mutation and query responses.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::SdkError;
    use crate::shared::FundingSource;
    use crate::shared::{ApiResponse, DecimalText, RejectionCode};
    use rust_decimal::Decimal;
    use serde_json::{json, Value};

    const HASH: &str = "4f1a4b1ab1c0c0ffee0000000000000000000000000000000000000000000001";

    #[test]
    fn order_fills_query_sends_only_the_given_parameters() {
        assert_eq!(
            order_fills_query(None, Some(20), Some("c"), None, None, Some(0)),
            [
                ("limit", "20".to_string()),
                ("cursor", "c".to_string()),
                ("fills", "0".to_string())
            ]
        );
        assert_eq!(
            order_fills_query(None, None, None, Some(HASH), Some("f"), None),
            [
                ("order_hash", HASH.to_string()),
                ("fill_cursor", "f".to_string())
            ]
        );
    }

    /// Unwrap a success envelope the way the HTTP client does.
    fn body<T: serde::de::DeserializeOwned>(envelope: Value) -> Result<T, SdkError> {
        match serde_json::from_value::<ApiResponse<T>>(envelope)? {
            ApiResponse::Success { body } => Ok(body),
            ApiResponse::Rejected { details } => Err(SdkError::ApiRejected(details)),
        }
    }

    #[test]
    fn submit_response_without_committed_view_is_accepted_pending() {
        let response: SubmitOrderResponse = body(json!({
            "status": "success",
            "body": {
                "order_hash": HASH,
                "status": "accepted_pending",
                "state": null,
                "initial_cohort": null,
                "fills": [],
                "fills_complete": false,
                "fills_next_cursor": null
            }
        }))
        .unwrap();
        assert_eq!(response.status, SubmitOrderStatus::AcceptedPending);
        assert!(response.state.is_none() && response.initial_cohort.is_none());
        assert_eq!(response.filled_base(), None);
    }

    #[test]
    fn submit_statuses_decode() {
        for (wire, status) in [
            ("filled", SubmitOrderStatus::Filled),
            ("accepted", SubmitOrderStatus::Accepted),
            ("accepted_pending", SubmitOrderStatus::AcceptedPending),
        ] {
            let decoded: SubmitOrderStatus = serde_json::from_value(json!(wire)).unwrap();
            assert_eq!(decoded, status);
        }
        // Retired or future statuses must not hide the accepted order hash.
        assert_eq!(
            serde_json::from_value::<SubmitOrderStatus>(json!("partial_fill")).unwrap(),
            SubmitOrderStatus::Unknown
        );
        let response: SubmitOrderResponse = body(json!({
            "status": "success",
            "body": {
                "order_hash": HASH,
                "status": "accepted_partial",
                "state": null,
                "initial_cohort": null,
                "fills": [],
                "fills_complete": true,
                "fills_next_cursor": null
            }
        }))
        .unwrap();
        assert_eq!(response.order_hash, HASH);
        assert_eq!(response.status, SubmitOrderStatus::Unknown);
    }

    #[test]
    fn submit_business_rejection_surfaces_the_code() {
        let error = body::<SubmitOrderResponse>(json!({
            "status": "error",
            "error_details": {
                "reason": "Requested funding cannot cover the complete order",
                "rejection_code": "INSUFFICIENT_BALANCE",
                "error_log_id": "LCERR_1"
            }
        }))
        .unwrap_err();
        let SdkError::ApiRejected(details) = error else {
            panic!("expected rejection");
        };
        assert_eq!(
            details.rejection_code,
            Some(RejectionCode::InsufficientBalance)
        );
    }

    #[test]
    fn cancel_response_decodes_raw_base_atoms() {
        let response: CancelSuccess = body(json!({
            "status": "success",
            "body": {
                "status": "cancelled",
                "order_hash": HASH,
                "quantities": {
                    "newly_cancelled_base": "18446744073709551615",
                    "confirmed_base": "0",
                    "pending_base": "40",
                    "remaining_open_base": "0"
                },
                "quantity_unit": "base_atoms",
                "revision": "812",
                "closed_reason": ""
            }
        }))
        .unwrap();
        assert_eq!(response.status, CancelStatus::Cancelled);
        let quantities = response.quantities.unwrap();
        assert_eq!(quantities.newly_cancelled_base, u64::MAX);
        assert_eq!(quantities.pending_base, 40);
        assert_eq!(response.quantity_unit, "base_atoms");
        assert_eq!(response.revision, 812);
        assert_eq!(response.closed_reason, None);

        let already: CancelSuccess = body(json!({
            "status": "success",
            "body": {
                "status": "already_closed",
                "order_hash": HASH,
                "quantities": null,
                "quantity_unit": "base_atoms",
                "revision": "813",
                "closed_reason": "expired"
            }
        }))
        .unwrap();
        assert_eq!(already.status, CancelStatus::AlreadyClosed);
        assert_eq!(already.quantities, None);
        assert_eq!(already.closed_reason.as_deref(), Some("expired"));

        assert_eq!(
            serde_json::from_value::<CancelStatus>(json!("superseded")).unwrap(),
            CancelStatus::Unknown
        );
    }

    #[test]
    fn cancel_unknown_hash_is_order_not_found() {
        let error = body::<CancelSuccess>(json!({
            "status": "error",
            "error_details": {
                "reason": "Order has not been accepted",
                "rejection_code": "ORDER_NOT_FOUND"
            }
        }))
        .unwrap_err();
        let SdkError::ApiRejected(details) = error else {
            panic!("expected rejection");
        };
        assert_eq!(details.rejection_code, Some(RejectionCode::OrderNotFound));
    }

    #[test]
    fn cancel_all_response_decodes_closure() {
        let response: CancelAllSuccess = body(json!({
            "status": "success",
            "body": {
                "status": "success",
                "user_pubkey": "Wallet1111111111111111111111111111111111111",
                "orderbook_id": "j749bQAbDsBAiyDs2Tj868heQj1b5KVp98ZrjiZd56a",
                "closure": {
                    "operation_id": "8c1d1c5e-2b8f-4bb1-9f53-3c3f5f0a1b2c",
                    "committed_revision": "900",
                    "scope": "WalletBook:Wallet1111111111111111111111111111111111111:j749bQAbDsBAiyDs2Tj868heQj1b5KVp98ZrjiZd56a",
                    "accepted_seq_cutoff": "44",
                    "cleanup_pending": true
                },
                "message": "Accepted-order cutoff committed"
            }
        }))
        .unwrap();
        let closure = response.closure.unwrap();
        assert_eq!(closure.committed_revision, 900);
        assert_eq!(closure.accepted_seq_cutoff, Some(44));
        assert!(closure.cleanup_pending);
        assert!(closure.scope.starts_with("WalletBook:"));

        let wallet_wide: CancelAllSuccess = body(json!({
            "status": "success",
            "body": {
                "status": "success",
                "user_pubkey": "Wallet1111111111111111111111111111111111111",
                "orderbook_id": "",
                "closure": {
                    "operation_id": "8c1d1c5e-2b8f-4bb1-9f53-3c3f5f0a1b2d",
                    "committed_revision": "901",
                    "scope": "Wallet:Wallet1111111111111111111111111111111111111",
                    "accepted_seq_cutoff": null,
                    "cleanup_pending": false
                },
                "message": "Accepted-order cutoff committed"
            }
        }))
        .unwrap();
        assert_eq!(wallet_wide.orderbook_id.as_str(), "");
        assert_eq!(wallet_wide.closure.unwrap().accepted_seq_cutoff, None);
    }

    #[test]
    fn user_orders_response_decodes_orders_and_funding_page() {
        let response: UserOrdersResponse = body(json!({
            "status": "success",
            "body": {
                "user_pubkey": "Wallet1111111111111111111111111111111111111",
                "orders": [{
                    "order_hash": HASH,
                    "market_pubkey": "A9Bxkkc4nah517EjgjwSafwspGnmU9s1Ei5PTo5ZkJd9",
                    "orderbook_id": "j749bQAbDsBAiyDs2Tj868heQj1b5KVp98ZrjiZd56a",
                    "side": "ask",
                    "amount_in": "4.00000000",
                    "amount_out": "2.200000",
                    "price": "0.5500",
                    "created_at": 1790685521784_i64,
                    "expiration": 0,
                    "base_mint": "BaseMint111111111111111111111111111111111111",
                    "quote_mint": "QuoteMint11111111111111111111111111111111111",
                    "outcome_index": -1,
                    "state": {
                        "original_base": "4.00000000",
                        "confirmed_base": "0.00000000",
                        "pending_base": "0.00000000",
                        "open_base": "4.00000000",
                        "cancelled_base": "0.00000000",
                        "closed_reason": null,
                        "committed_revision": "812",
                        "accepted_seq": "45"
                    },
                    "tif": "GTC",
                    "source": "conditional",
                    "order_type": "limit"
                }],
                "funding_accounts": [{
                    "account": "Acct1111111111111111111111111111111111111111",
                    "mint": "Cond1111111111111111111111111111111111111111",
                    "source": "conditional",
                    "market_pubkey": "A9Bxkkc4nah517EjgjwSafwspGnmU9s1Ei5PTo5ZkJd9",
                    "deposit_mint": "7SrxsoXjNR7Y8T3koJCt1yV4FrNUumoAUrJExDt6tQez",
                    "raw_observed": "4.00000000",
                    "order_reserved": "340282366920938463463374607431768.211455",
                    "execution_reserved": "0.00000000",
                    "signed_remaining_atoms": "0",
                    "accepted_boundary": "boundary",
                    "observed_slot": "123",
                    "observed_blockhash": "hash",
                    "observation_state": "present"
                }],
                "next_cursor": null,
                "has_more": false,
                "next_funding_cursor": null,
                "funding_has_more": false,
                "committed_revision": "812",
                "projection_generation": "1"
            }
        }))
        .unwrap();
        assert_eq!(response.committed_revision, 812);
        let order = response.orders[0].clone().into_limit_order().unwrap();
        assert_eq!(order.remaining_size, Decimal::from(4));
        assert_eq!(order.accepted_seq, 45);
        assert_eq!(order.funding_source, FundingSource::Conditional);
        let account = &response.funding_accounts[0];
        assert_eq!(
            account.order_reserved,
            "340282366920938463463374607431768.211455"
                .parse::<DecimalText>()
                .unwrap()
        );
        assert_eq!(account.observed_slot, Some(123));
    }

    #[test]
    fn empty_user_orders_page_may_still_have_more() {
        let response: UserOrdersResponse = body(json!({
            "status": "success",
            "body": {
                "user_pubkey": "Wallet1111111111111111111111111111111111111",
                "orders": [],
                "funding_accounts": [],
                "next_cursor": "45:4f1a",
                "has_more": true,
                "next_funding_cursor": null,
                "funding_has_more": false,
                "committed_revision": "812",
                "projection_generation": "1"
            }
        }))
        .unwrap();
        assert!(response.orders.is_empty() && response.has_more);
        assert_eq!(response.next_cursor.as_deref(), Some("45:4f1a"));
    }
}
