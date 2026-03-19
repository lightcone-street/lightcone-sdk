//! Fluent builder for Privy order submission.
//!
//! Created via `client.privy().limit_order().await` or `client.privy().trigger_order().await`.

use crate::client::LightconeClient;
use crate::error::SdkError;
use crate::privy::PrivyOrderEnvelope;
use crate::program::types::OrderSide;
use crate::shared::{DepositSource, TimeInForce, TriggerType};
use solana_pubkey::Pubkey;

/// Fluent builder for submitting orders via Privy embedded wallets.
///
/// Pre-seeded with the client's deposit source setting. Handles both limit
/// and trigger orders — trigger fields are optional and only included when set.
///
/// Prefer `client.privy().limit_order().await` or `client.privy().trigger_order().await`
/// to create builders. Direct construction of `PrivyOrderEnvelope` is also available
/// for power users.
///
/// # Example (limit order)
///
/// ```rust,ignore
/// let response = client.privy().limit_order().await
///     .wallet_id(&wallet_id)
///     .maker(keypair.pubkey())
///     .market(market_pubkey)
///     .base_mint(base_mint)
///     .quote_mint(quote_mint)
///     .bid()
///     .amount_in(1_000_000)
///     .amount_out(500_000)
///     .nonce(1)
///     .salt(12345)
///     .orderbook_id("ob_123")
///     .time_in_force(TimeInForce::Gtc)
///     .submit()
///     .await?;
/// ```
///
/// # Example (trigger order)
///
/// ```rust,ignore
/// let response = client.privy().trigger_order().await
///     .wallet_id(&wallet_id)
///     .maker(keypair.pubkey())
///     .market(market_pubkey)
///     .base_mint(base_mint)
///     .quote_mint(quote_mint)
///     .ask()
///     .amount_in(500_000)
///     .amount_out(1_000_000)
///     .nonce(1)
///     .salt(12345)
///     .orderbook_id("ob_123")
///     .take_profit(0.75)
///     .gtc()
///     .submit()
///     .await?;
/// ```
#[derive(Debug, Clone, Copy)]
pub(crate) enum PrivyOrderKind {
    Limit,
    Trigger,
}

pub struct PrivyOrderBuilder<'a> {
    client: &'a LightconeClient,
    kind: PrivyOrderKind,
    wallet_id: Option<String>,
    maker: Option<Pubkey>,
    market: Option<Pubkey>,
    base_mint: Option<Pubkey>,
    quote_mint: Option<Pubkey>,
    side: Option<OrderSide>,
    amount_in: Option<u64>,
    amount_out: Option<u64>,
    nonce: Option<u64>,
    salt: Option<u64>,
    expiration: i64,
    orderbook_id: Option<String>,
    time_in_force: Option<TimeInForce>,
    trigger_price: Option<f64>,
    trigger_type: Option<TriggerType>,
    deposit_source: Option<DepositSource>,
}

impl<'a> PrivyOrderBuilder<'a> {
    pub(crate) fn new(
        client: &'a LightconeClient,
        deposit_source: DepositSource,
        kind: PrivyOrderKind,
    ) -> Self {
        Self {
            client,
            kind,
            wallet_id: None,
            maker: None,
            market: None,
            base_mint: None,
            quote_mint: None,
            side: None,
            amount_in: None,
            amount_out: None,
            nonce: None,
            salt: None,
            expiration: 0,
            orderbook_id: None,
            time_in_force: None,
            trigger_price: None,
            trigger_type: None,
            deposit_source: Some(deposit_source),
        }
    }

    /// Set the Privy embedded wallet ID.
    pub fn wallet_id(mut self, wallet_id: impl Into<String>) -> Self {
        self.wallet_id = Some(wallet_id.into());
        self
    }

    /// Set the maker public key.
    pub fn maker(mut self, maker: Pubkey) -> Self {
        self.maker = Some(maker);
        self
    }

    /// Set the market public key.
    pub fn market(mut self, market: Pubkey) -> Self {
        self.market = Some(market);
        self
    }

    /// Set the base token mint.
    pub fn base_mint(mut self, base_mint: Pubkey) -> Self {
        self.base_mint = Some(base_mint);
        self
    }

    /// Set the quote token mint.
    pub fn quote_mint(mut self, quote_mint: Pubkey) -> Self {
        self.quote_mint = Some(quote_mint);
        self
    }

    /// Set the order side.
    pub fn side(mut self, side: OrderSide) -> Self {
        self.side = Some(side);
        self
    }

    /// Set side to Bid (buy).
    pub fn bid(self) -> Self {
        self.side(OrderSide::Bid)
    }

    /// Set side to Ask (sell).
    pub fn ask(self) -> Self {
        self.side(OrderSide::Ask)
    }

    /// Set the amount in (raw lamports).
    pub fn amount_in(mut self, amount: u64) -> Self {
        self.amount_in = Some(amount);
        self
    }

    /// Set the amount out (raw lamports).
    pub fn amount_out(mut self, amount: u64) -> Self {
        self.amount_out = Some(amount);
        self
    }

    /// Set the order nonce.
    pub fn nonce(mut self, nonce: u64) -> Self {
        self.nonce = Some(nonce);
        self
    }

    /// Set the order salt.
    pub fn salt(mut self, salt: u64) -> Self {
        self.salt = Some(salt);
        self
    }

    /// Set the expiration timestamp (0 = no expiration).
    pub fn expiration(mut self, expiration: i64) -> Self {
        self.expiration = expiration;
        self
    }

    /// Set the orderbook ID.
    pub fn orderbook_id(mut self, orderbook_id: impl Into<String>) -> Self {
        self.orderbook_id = Some(orderbook_id.into());
        self
    }

    /// Set the time-in-force policy.
    pub fn time_in_force(mut self, tif: TimeInForce) -> Self {
        self.time_in_force = Some(tif);
        self
    }

    /// Good-til-cancelled.
    pub fn gtc(self) -> Self { self.time_in_force(TimeInForce::Gtc) }

    /// Immediate-or-cancel.
    pub fn ioc(self) -> Self { self.time_in_force(TimeInForce::Ioc) }

    /// Fill-or-kill.
    pub fn fok(self) -> Self { self.time_in_force(TimeInForce::Fok) }

    /// Add-liquidity-only (post-only).
    pub fn alo(self) -> Self { self.time_in_force(TimeInForce::Alo) }

    /// Set the trigger price.
    pub fn trigger_price(mut self, price: f64) -> Self {
        self.trigger_price = Some(price);
        self
    }

    /// Set the trigger type.
    pub fn trigger_type(mut self, trigger_type: TriggerType) -> Self {
        self.trigger_type = Some(trigger_type);
        self
    }

    /// Take-profit shorthand: sets trigger_price and trigger_type in one call.
    pub fn take_profit(self, price: f64) -> Self {
        self.trigger_price(price).trigger_type(TriggerType::TakeProfit)
    }

    /// Stop-loss shorthand: sets trigger_price and trigger_type in one call.
    pub fn stop_loss(self, price: f64) -> Self {
        self.trigger_price(price).trigger_type(TriggerType::StopLoss)
    }

    /// Override the deposit source for this order.
    pub fn deposit_source(mut self, source: DepositSource) -> Self {
        self.deposit_source = Some(source);
        self
    }

    /// Submit the order via Privy.
    ///
    /// Validates required fields, resolves the deposit source, builds the
    /// wire envelope, and sends it to the backend for Privy signing + submission.
    pub async fn submit(self) -> Result<serde_json::Value, SdkError> {
        let wallet_id = self
            .wallet_id
            .ok_or_else(|| SdkError::Validation("wallet_id is required".into()))?;
        let maker = self
            .maker
            .ok_or_else(|| SdkError::Validation("maker is required".into()))?;
        let market = self
            .market
            .ok_or_else(|| SdkError::Validation("market is required".into()))?;
        let base_mint = self
            .base_mint
            .ok_or_else(|| SdkError::Validation("base_mint is required".into()))?;
        let quote_mint = self
            .quote_mint
            .ok_or_else(|| SdkError::Validation("quote_mint is required".into()))?;
        let side = self
            .side
            .ok_or_else(|| SdkError::Validation("side is required".into()))?;
        let amount_in = self
            .amount_in
            .ok_or_else(|| SdkError::Validation("amount_in is required".into()))?;
        let amount_out = self
            .amount_out
            .ok_or_else(|| SdkError::Validation("amount_out is required".into()))?;
        let nonce = self
            .nonce
            .ok_or_else(|| SdkError::Validation("nonce is required".into()))?;
        let salt = self
            .salt
            .ok_or_else(|| SdkError::Validation("salt is required".into()))?;
        let orderbook_id = self
            .orderbook_id
            .ok_or_else(|| SdkError::Validation("orderbook_id is required".into()))?;

        // Kind-aware validation
        let (trigger_price, trigger_type) = match self.kind {
            PrivyOrderKind::Trigger => {
                let trigger_price = self.trigger_price.ok_or_else(|| {
                    SdkError::Validation(
                        "trigger_price is required for trigger orders".into(),
                    )
                })?;
                let trigger_type = self.trigger_type.ok_or_else(|| {
                    SdkError::Validation(
                        "trigger_type is required for trigger orders".into(),
                    )
                })?;
                (Some(trigger_price), Some(trigger_type))
            }
            PrivyOrderKind::Limit => (None, None),
        };

        let resolved_deposit_source = self
            .client
            .resolve_deposit_source(self.deposit_source)
            .await;

        let envelope = PrivyOrderEnvelope {
            maker: maker.to_string(),
            nonce,
            salt,
            market_pubkey: market.to_string(),
            base_token: base_mint.to_string(),
            quote_token: quote_mint.to_string(),
            side: side as u32,
            amount_in,
            amount_out,
            expiration: self.expiration,
            orderbook_id,
            time_in_force: self.time_in_force,
            trigger_price,
            trigger_type,
            deposit_source: Some(resolved_deposit_source),
        };

        self.client
            .privy()
            .sign_and_send_order(&wallet_id, envelope)
            .await
    }
}
