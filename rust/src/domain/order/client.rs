//! Orders sub-client — submit, cancel, query, and on-chain order operations.

use super::wire::{UserSnapshotBalance, UserSnapshotOrder};
use crate::client::LightconeClient;
use crate::domain::order::UserOrderFillsResponse;
use crate::error::SdkError;
use crate::http::RetryPolicy;
use crate::program::envelope::{LimitOrderEnvelope, OrderEnvelope, TriggerOrderEnvelope};
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
pub struct CancelSuccess {
    pub order_hash: String,
    pub remaining: u64,
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
pub struct TriggerOrderResponse {
    pub trigger_order_id: String,
    pub order_hash: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CancelTriggerSuccess {
    pub trigger_order_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    pub async fn trigger_order(&self) -> TriggerOrderEnvelope {
        let deposit_source = self.client.deposit_source().await;
        TriggerOrderEnvelope::new().deposit_source(deposit_source)
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

    pub async fn submit_trigger(
        &self,
        request: &impl serde::Serialize,
    ) -> Result<TriggerOrderResponse, SdkError> {
        let url = format!("{}/api/orders/submit", self.client.http.base_url());
        self.client
            .http
            .post(&url, request, RetryPolicy::None)
            .await
    }

    pub async fn cancel_trigger(
        &self,
        body: &CancelTriggerBody,
    ) -> Result<CancelTriggerSuccess, SdkError> {
        let url = format!("{}/api/orders/cancel", self.client.http.base_url());
        self.client.http.post(&url, body, RetryPolicy::None).await
    }

    /// Fetch the authenticated user's open orders. Wallet is resolved
    /// server-side from the `auth_token` cookie, so no parameter is required.
    pub async fn get_user_orders(
        &self,
        limit: Option<u32>,
        cursor: Option<&str>,
    ) -> Result<UserOrdersResponse, SdkError> {
        let url = build_user_orders_authenticated_url(self.client.http.base_url(), limit, cursor);
        self.client.http.get(&url, RetryPolicy::Idempotent).await
    }

    /// Same as [`Self::get_user_orders`], but uses the supplied `auth_token`
    /// for this call instead of the SDK's process-wide token store. Intended
    /// for server-side cookie forwarding (SSR / server functions).
    pub async fn get_user_orders_with_auth(
        &self,
        limit: Option<u32>,
        cursor: Option<&str>,
        auth_token: &str,
    ) -> Result<UserOrdersResponse, SdkError> {
        let url = build_user_orders_authenticated_url(self.client.http.base_url(), limit, cursor);
        self.client
            .http
            .get_with_auth(&url, RetryPolicy::Idempotent, auth_token)
            .await
    }

    /// Fetch the authenticated user's filled orders with nested fill events.
    /// Wallet is resolved server-side from the `auth_token` cookie.
    ///
    /// Includes orders where the user was either maker or taker.
    /// Optionally filter by market. Returns orders sorted by most recent fill first.
    pub async fn get_user_order_fills(
        &self,
        market_pubkey: Option<&str>,
        limit: Option<u32>,
        cursor: Option<&str>,
    ) -> Result<UserOrderFillsResponse, SdkError> {
        let url = build_user_order_fills_authenticated_url(
            self.client.http.base_url(),
            market_pubkey,
            limit,
            cursor,
        );
        self.client.http.get(&url, RetryPolicy::Idempotent).await
    }

    /// Same as [`Self::get_user_order_fills`], but uses the supplied
    /// `auth_token` for this call instead of the SDK's process-wide token
    /// store. Intended for server-side cookie forwarding (SSR / server functions).
    pub async fn get_user_order_fills_with_auth(
        &self,
        market_pubkey: Option<&str>,
        limit: Option<u32>,
        cursor: Option<&str>,
        auth_token: &str,
    ) -> Result<UserOrderFillsResponse, SdkError> {
        let url = build_user_order_fills_authenticated_url(
            self.client.http.base_url(),
            market_pubkey,
            limit,
            cursor,
        );
        self.client
            .http
            .get_with_auth(&url, RetryPolicy::Idempotent, auth_token)
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
    ) -> Result<UserOrderFillsResponse, SdkError> {
        let url = build_user_order_fills_by_wallet_url(
            self.client.http.base_url(),
            wallet_address,
            market_pubkey,
            limit,
            cursor,
        );
        self.client.http.get(&url, RetryPolicy::Idempotent).await
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
            SigningStrategy::Privy { wallet_id } => {
                let result = self
                    .client
                    .privy()
                    .sign_and_cancel_order(&wallet_id, order_hash, maker.as_str())
                    .await?;
                serde_json::from_value(result).map_err(|error| {
                    SdkError::Other(format!("failed to parse cancel response: {error}"))
                })
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
            SigningStrategy::Privy { wallet_id } => {
                let result = self
                    .client
                    .privy()
                    .sign_and_cancel_all_orders(
                        &wallet_id,
                        user_pubkey.as_str(),
                        orderbook_id_str,
                        timestamp,
                        salt,
                    )
                    .await?;
                serde_json::from_value(result).map_err(|error| {
                    SdkError::Other(format!("failed to parse cancel-all response: {error}"))
                })
            }
        }
    }

    /// Cancel a trigger order using the client's signing strategy.
    ///
    /// Signs the cancel message and submits the cancellation request.
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
            SigningStrategy::Privy { wallet_id } => {
                let result = self
                    .client
                    .privy()
                    .sign_and_cancel_trigger_order(&wallet_id, trigger_order_id, maker.as_str())
                    .await?;
                serde_json::from_value(result).map_err(|error| {
                    SdkError::Other(format!("failed to parse cancel-trigger response: {error}"))
                })
            }
        }
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
    pub fn sign_order(&self, order: &mut OrderPayload, keypair: &Keypair) {
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
            Ok(account) => Ok(Some(crate::program::accounts::OrderStatus::deserialize(
                &account.data,
            )?)),
            Err(_) => Ok(None),
        }
    }

    /// Fetch a user's current nonce (returns 0 if not initialized).
    pub async fn get_nonce(&self, user: &Pubkey) -> Result<u64, SdkError> {
        let rpc = crate::rpc::require_solana_rpc(self.client)?;
        let pda = self.nonce_pda(user);
        match rpc.get_account(&pda).await {
            Ok(account) => {
                let user_nonce = crate::program::accounts::UserNonce::deserialize(&account.data)?;
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

fn build_user_orders_authenticated_url(
    base_url: &str,
    limit: Option<u32>,
    cursor: Option<&str>,
) -> String {
    let mut url = format!("{}/api/users/orders", base_url);
    let mut separator = '?';
    if let Some(limit) = limit {
        url.push_str(&format!("{}limit={}", separator, limit));
        separator = '&';
    }
    if let Some(cursor) = cursor {
        url.push_str(&format!("{}cursor={}", separator, cursor));
    }
    url
}

fn build_user_order_fills_authenticated_url(
    base_url: &str,
    market_pubkey: Option<&str>,
    limit: Option<u32>,
    cursor: Option<&str>,
) -> String {
    let mut url = format!("{}/api/users/order-fills", base_url);
    let mut separator = '?';
    if let Some(market_pubkey) = market_pubkey {
        url.push_str(&format!("{}market_pubkey={}", separator, market_pubkey));
        separator = '&';
    }
    if let Some(limit) = limit {
        url.push_str(&format!("{}limit={}", separator, limit));
        separator = '&';
    }
    if let Some(cursor) = cursor {
        url.push_str(&format!("{}cursor={}", separator, cursor));
    }
    url
}

fn build_user_order_fills_by_wallet_url(
    base_url: &str,
    wallet_address: &str,
    market_pubkey: Option<&str>,
    limit: Option<u32>,
    cursor: Option<&str>,
) -> String {
    let mut url = format!("{}/api/users/{}/order-fills", base_url, wallet_address);
    let mut separator = '?';
    if let Some(market_pubkey) = market_pubkey {
        url.push_str(&format!("{}market_pubkey={}", separator, market_pubkey));
        separator = '&';
    }
    if let Some(limit) = limit {
        url.push_str(&format!("{}limit={}", separator, limit));
        separator = '&';
    }
    if let Some(cursor) = cursor {
        url.push_str(&format!("{}cursor={}", separator, cursor));
    }
    url
}
