//! Order envelope types: fluent builders that produce signed SubmitOrderRequests.

use solana_pubkey::Pubkey;

#[cfg(feature = "native-auth")]
use solana_keypair::Keypair;

use crate::domain::orderbook::OrderBookPair;
use crate::program::error::SdkError;
use crate::program::orders::{generate_salt, OrderPayload};
use crate::program::types::OrderSide;
use crate::shared::scaling::{
    scale_price_size, validate_raw_amounts, validate_signed_fields, OrderbookRules,
};
#[cfg(feature = "trigger_orders")]
use crate::shared::{validate_trigger_price, ExactDecimal, TriggerType};
use crate::shared::{DepositSource, SubmitOrderRequest, TimeInForce};

// ─── Shared base fields ─────────────────────────────────────────────────────

#[derive(Debug, Clone, Default)]
struct OrderFields {
    nonce: Option<u64>,
    salt: Option<u64>,
    maker: Option<Pubkey>,
    market: Option<Pubkey>,
    base_mint: Option<Pubkey>,
    quote_mint: Option<Pubkey>,
    side: Option<OrderSide>,
    amount_in: Option<u64>,
    amount_out: Option<u64>,
    expiration: i64,
    price_raw: Option<String>,
    size_raw: Option<String>,
    deposit_source: Option<DepositSource>,
}

impl OrderFields {
    fn to_payload(&self) -> Result<OrderPayload, SdkError> {
        let amount_in = self
            .amount_in
            .ok_or_else(|| SdkError::MissingField("amount_in".into()))?;
        let amount_out = self
            .amount_out
            .ok_or_else(|| SdkError::MissingField("amount_out".into()))?;
        if amount_in == 0 {
            return Err(SdkError::MissingField(
                "amount_in must be greater than 0".into(),
            ));
        }
        if amount_out == 0 {
            return Err(SdkError::MissingField(
                "amount_out must be greater than 0".into(),
            ));
        }

        let nonce = self.nonce.unwrap_or(0);
        let salt = self.salt.unwrap_or_else(generate_salt);
        validate_signed_fields(amount_in, amount_out, salt, nonce)?;

        Ok(OrderPayload {
            nonce,
            salt,
            maker: self
                .maker
                .ok_or_else(|| SdkError::MissingField("maker".into()))?,
            market: self
                .market
                .ok_or_else(|| SdkError::MissingField("market".into()))?,
            base_mint: self
                .base_mint
                .ok_or_else(|| SdkError::MissingField("base_mint".into()))?,
            quote_mint: self
                .quote_mint
                .ok_or_else(|| SdkError::MissingField("quote_mint".into()))?,
            side: self
                .side
                .ok_or_else(|| SdkError::MissingField("side (call .bid() or .ask())".into()))?,
            amount_in,
            amount_out,
            expiration: self.expiration,
            signature: [0u8; 64],
        })
    }

    /// Auto-fill market, base_mint, and quote_mint from the orderbook if not
    /// explicitly set by the caller.
    fn auto_fill_from_orderbook(&mut self, orderbook: &OrderBookPair) -> Result<(), SdkError> {
        use crate::domain::market::tokens::Token;

        if self.market.is_none() {
            self.market = Some(orderbook.market_pubkey.to_pubkey().map_err(|error| {
                SdkError::MissingField(format!("invalid market pubkey: {error}"))
            })?);
        }
        if self.salt.is_none() {
            self.salt = Some(generate_salt())
        }
        if self.base_mint.is_none() {
            self.base_mint = Some(orderbook.base.pubkey().to_pubkey().map_err(|error| {
                SdkError::MissingField(format!("invalid base mint pubkey: {error}"))
            })?);
        }
        if self.quote_mint.is_none() {
            self.quote_mint = Some(orderbook.quote.pubkey().to_pubkey().map_err(|error| {
                SdkError::MissingField(format!("invalid quote mint pubkey: {error}"))
            })?);
        }
        Ok(())
    }

    /// Construct or preflight the exact signed ratio using fetched rules.
    fn apply_rules(&mut self, rules: &OrderbookRules, orderbook_id: &str) -> Result<(), SdkError> {
        rules.validate_for_orderbook(orderbook_id)?;
        let side = self
            .side
            .ok_or_else(|| SdkError::MissingField("side (call .bid() or .ask())".into()))?;
        match (self.amount_in, self.amount_out) {
            (Some(amount_in), Some(amount_out)) => {
                validate_raw_amounts(amount_in, amount_out, side, rules)?;
                Ok(())
            }
            (None, None) => {
                let price = self.price_raw.as_deref().ok_or_else(|| {
                    SdkError::MissingField(
                        "either price()+size() or amount_in()+amount_out() is required".into(),
                    )
                })?;
                let size = self.size_raw.as_deref().ok_or_else(|| {
                    SdkError::MissingField(
                        "either price()+size() or amount_in()+amount_out() is required".into(),
                    )
                })?;
                let scaled = scale_price_size(price, size, side, rules)?;
                self.amount_in = Some(scaled.amount_in);
                self.amount_out = Some(scaled.amount_out);
                Ok(())
            }
            _ => Err(SdkError::MissingField(
                "amount_in and amount_out must be supplied together".into(),
            )),
        }
    }
}

// ─── OrderEnvelope trait ────────────────────────────────────────────────────

/// Shared fluent API for building orders.
///
/// Implemented by both `LimitOrderEnvelope` and `TriggerOrderEnvelope`.
///
/// Prefer `client.orders().limit_order().await` or `client.orders().trigger_order().await`
/// which pre-seed the client's deposit source. Direct construction via `::new()` is
/// also available for standalone use.
pub trait OrderEnvelope: Sized {
    fn new() -> Self;
    fn nonce(self, nonce: u64) -> Self;
    fn salt(self, salt: u64) -> Self;
    fn maker(self, maker: Pubkey) -> Self;
    fn market(self, market: Pubkey) -> Self;
    fn base_mint(self, base_mint: Pubkey) -> Self;
    fn quote_mint(self, quote_mint: Pubkey) -> Self;
    fn bid(self) -> Self;
    fn ask(self) -> Self;
    fn side(self, side: OrderSide) -> Self;
    fn amount_in(self, amount: u64) -> Self;
    fn amount_out(self, amount: u64) -> Self;
    fn expiration(self, expiration: i64) -> Self;
    fn price(self, price: &str) -> Self;
    fn size(self, size: &str) -> Self;
    fn deposit_source(self, ds: DepositSource) -> Self;

    /// Build an unsigned `OrderPayload` without consuming the envelope.
    fn payload(&self) -> Result<OrderPayload, SdkError>;

    /// Sign and produce a `SubmitOrderRequest`. Consumes the envelope.
    ///
    /// Fetched rules are mandatory. Human values are constructed exactly and
    /// raw amounts are preflighted against the same admission rules.
    #[cfg(feature = "native-auth")]
    fn sign(
        self,
        keypair: &Keypair,
        orderbook: &OrderBookPair,
        rules: &OrderbookRules,
    ) -> Result<SubmitOrderRequest, SdkError>;

    /// Apply an external wallet-adapter signature and produce a `SubmitOrderRequest`.
    /// Consumes the envelope.
    ///
    /// Performs the same exact preflight as `sign()` before attaching the signature.
    fn finalize(
        self,
        sig_bs58: &str,
        orderbook: &OrderBookPair,
        rules: &OrderbookRules,
    ) -> Result<SubmitOrderRequest, SdkError>;
}

// ─── Shared implementations via macro ───────────────────────────────────────

macro_rules! impl_base_methods {
    ($ty:ident) => {
        fn new() -> Self {
            Self::default()
        }

        fn nonce(mut self, nonce: u64) -> Self {
            self.fields.nonce = Some(nonce);
            self
        }

        fn salt(mut self, salt: u64) -> Self {
            self.fields.salt = Some(salt);
            self
        }

        fn maker(mut self, maker: Pubkey) -> Self {
            self.fields.maker = Some(maker);
            self
        }

        fn market(mut self, market: Pubkey) -> Self {
            self.fields.market = Some(market);
            self
        }

        fn base_mint(mut self, base_mint: Pubkey) -> Self {
            self.fields.base_mint = Some(base_mint);
            self
        }

        fn quote_mint(mut self, quote_mint: Pubkey) -> Self {
            self.fields.quote_mint = Some(quote_mint);
            self
        }

        fn bid(mut self) -> Self {
            self.fields.side = Some(OrderSide::Bid);
            self
        }

        fn ask(mut self) -> Self {
            self.fields.side = Some(OrderSide::Ask);
            self
        }

        fn side(mut self, side: OrderSide) -> Self {
            self.fields.side = Some(side);
            self
        }

        fn amount_in(mut self, amount: u64) -> Self {
            self.fields.amount_in = Some(amount);
            self
        }

        fn amount_out(mut self, amount: u64) -> Self {
            self.fields.amount_out = Some(amount);
            self
        }

        fn expiration(mut self, expiration: i64) -> Self {
            self.fields.expiration = expiration;
            self
        }

        fn price(mut self, price: &str) -> Self {
            self.fields.price_raw = Some(price.to_string());
            self
        }

        fn size(mut self, size: &str) -> Self {
            self.fields.size_raw = Some(size.to_string());
            self
        }

        fn deposit_source(mut self, ds: DepositSource) -> Self {
            self.fields.deposit_source = Some(ds);
            self
        }

        fn payload(&self) -> Result<OrderPayload, SdkError> {
            self.fields.to_payload()
        }
    };
}

// ─── LimitOrderEnvelope ─────────────────────────────────────────────────────

/// Envelope for building and submitting limit orders.
///
/// Prefer `client.orders().limit_order().await` which pre-seeds the client's
/// deposit source. `LimitOrderEnvelope::new()` is also available for standalone use.
///
/// Fields like `market`, `base_mint`, `quote_mint` are auto-populated from the
/// `OrderBookPair` passed to `sign()`/`finalize()` when not set explicitly.
/// When using `submit()`, `nonce` is auto-populated from the client's cached
/// nonce if not explicitly set (falling back to 0). When using `sign()`/`finalize()`
/// directly, `nonce` defaults to 0. `salt` is auto-generated when omitted.
///
/// # Example (via client builder — recommended)
///
/// ```rust,ignore
/// let request = client.orders().limit_order().await
///     .maker(maker_pubkey)
///     .bid()
///     .price("0.55")
///     .size("100")
///     .sign(&keypair, &orderbook, &rules)?;
/// ```
///
/// # Example (standalone with raw amounts)
///
/// ```rust,ignore
/// let request = LimitOrderEnvelope::new()
///     .maker(maker_pubkey)
///     .bid()
///     .amount_in(1_000_000)
///     .amount_out(500_000)
///     .sign(&keypair, &orderbook, &rules)?;
/// ```
#[derive(Debug, Clone, Default)]
pub struct LimitOrderEnvelope {
    fields: OrderFields,
    time_in_force: Option<TimeInForce>,
}

impl OrderEnvelope for LimitOrderEnvelope {
    impl_base_methods!(LimitOrderEnvelope);

    #[cfg(feature = "native-auth")]
    fn sign(
        mut self,
        keypair: &Keypair,
        orderbook: &OrderBookPair,
        rules: &OrderbookRules,
    ) -> Result<SubmitOrderRequest, SdkError> {
        self.fields.auto_fill_from_orderbook(orderbook)?;
        self.fields
            .apply_rules(rules, orderbook.orderbook_id.as_str())?;
        let mut payload = self.fields.to_payload()?;
        payload.sign(keypair, rules)?;
        payload.to_submit_request(
            orderbook.orderbook_id.as_str(),
            self.time_in_force,
            None,
            None,
            self.fields.deposit_source,
        )
    }

    fn finalize(
        mut self,
        sig_bs58: &str,
        orderbook: &OrderBookPair,
        rules: &OrderbookRules,
    ) -> Result<SubmitOrderRequest, SdkError> {
        self.fields.auto_fill_from_orderbook(orderbook)?;
        self.fields
            .apply_rules(rules, orderbook.orderbook_id.as_str())?;
        let mut payload = self.fields.to_payload()?;
        payload.apply_signature(sig_bs58.to_string(), rules)?;
        payload.to_submit_request(
            orderbook.orderbook_id.as_str(),
            self.time_in_force,
            None,
            None,
            self.fields.deposit_source,
        )
    }
}

impl LimitOrderEnvelope {
    /// Set time-in-force policy (GTC, IOC, FOK, ALO).
    pub fn time_in_force(mut self, tif: TimeInForce) -> Self {
        self.time_in_force = Some(tif);
        self
    }
}

// ─── TriggerOrderEnvelope ───────────────────────────────────────────────────

/// Envelope for building and submitting trigger (take-profit / stop-loss) orders.
///
/// Prefer `client.orders().trigger_order().await` which pre-seeds the client's
/// deposit source. `TriggerOrderEnvelope::new()` is also available for standalone use.
///
/// Adds trigger-specific fields on top of the shared order fields.
/// `trigger_price` and `trigger_type` are required before calling `sign()` or `finalize()`.
///
/// Fields like `market`, `base_mint`, `quote_mint` are auto-populated from the
/// `OrderBookPair` passed to `sign()`/`finalize()` when not set explicitly.
/// When using `submit()`, `nonce` is auto-populated from the client's cached
/// nonce if not explicitly set (falling back to 0). When using `sign()`/`finalize()`
/// directly, `nonce` defaults to 0. `salt` is auto-generated when omitted.
///
/// Requires the `trigger_orders` feature.
///
/// # Example (via client builder — recommended)
///
/// ```rust,ignore
/// let request = client.orders().trigger_order().await
///     .maker(maker_pubkey)
///     .ask()
///     .price("0.55")
///     .size("100")
///     .take_profit("0.75")
///     .gtc()
///     .sign(&keypair, &orderbook, &rules)?;
/// ```
///
/// # Example (standalone)
///
/// ```rust,ignore
/// let request = TriggerOrderEnvelope::new()
///     .maker(maker_pubkey)
///     .bid()
///     .amount_in(1_000_000)
///     .amount_out(500_000)
///     .stop_loss("0.30")
///     .sign(&keypair, &orderbook, &rules)?;
/// ```
#[cfg(feature = "trigger_orders")]
#[derive(Debug, Clone, Default)]
pub struct TriggerOrderEnvelope {
    fields: OrderFields,
    time_in_force: Option<TimeInForce>,
    trigger_price: Option<String>,
    trigger_type: Option<TriggerType>,
}

#[cfg(feature = "trigger_orders")]
impl OrderEnvelope for TriggerOrderEnvelope {
    impl_base_methods!(TriggerOrderEnvelope);

    #[cfg(feature = "native-auth")]
    fn sign(
        mut self,
        keypair: &Keypair,
        orderbook: &OrderBookPair,
        rules: &OrderbookRules,
    ) -> Result<SubmitOrderRequest, SdkError> {
        let (trigger_price, trigger_type) = self.validated_trigger(rules)?;

        self.fields.auto_fill_from_orderbook(orderbook)?;
        self.fields
            .apply_rules(rules, orderbook.orderbook_id.as_str())?;
        let mut payload = self.fields.to_payload()?;
        payload.sign(keypair, rules)?;
        payload.to_submit_request(
            orderbook.orderbook_id.as_str(),
            self.time_in_force,
            Some(trigger_price),
            Some(trigger_type),
            self.fields.deposit_source,
        )
    }

    fn finalize(
        mut self,
        sig_bs58: &str,
        orderbook: &OrderBookPair,
        rules: &OrderbookRules,
    ) -> Result<SubmitOrderRequest, SdkError> {
        let (trigger_price, trigger_type) = self.validated_trigger(rules)?;

        self.fields.auto_fill_from_orderbook(orderbook)?;
        self.fields
            .apply_rules(rules, orderbook.orderbook_id.as_str())?;
        let mut payload = self.fields.to_payload()?;
        payload.apply_signature(sig_bs58.to_string(), rules)?;
        payload.to_submit_request(
            orderbook.orderbook_id.as_str(),
            self.time_in_force,
            Some(trigger_price),
            Some(trigger_type),
            self.fields.deposit_source,
        )
    }
}

#[cfg(feature = "trigger_orders")]
impl TriggerOrderEnvelope {
    fn validated_trigger(
        &self,
        rules: &OrderbookRules,
    ) -> Result<(ExactDecimal, TriggerType), SdkError> {
        let trigger_price = self.trigger_price.as_deref().ok_or_else(|| {
            SdkError::MissingField("trigger_price is required for trigger orders".into())
        })?;
        let trigger_type = self.trigger_type.ok_or_else(|| {
            SdkError::MissingField("trigger_type is required for trigger orders".into())
        })?;
        validate_trigger_price(trigger_price, rules.price_decimals)?;
        let trigger_price = trigger_price
            .parse::<ExactDecimal>()
            .map_err(|_| crate::shared::scaling::ScalingError::TriggerPriceOutOfRange)?;
        Ok((trigger_price, trigger_type))
    }

    /// Set time-in-force policy (GTC, IOC, FOK, ALO).
    pub fn time_in_force(mut self, tif: TimeInForce) -> Self {
        self.time_in_force = Some(tif);
        self
    }

    /// Set trigger price for the conditional order.
    pub fn trigger_price(mut self, price: &str) -> Self {
        self.trigger_price = Some(price.to_string());
        self
    }

    /// Set trigger type (TakeProfit or StopLoss).
    pub fn trigger_type(mut self, trigger_type: TriggerType) -> Self {
        self.trigger_type = Some(trigger_type);
        self
    }

    /// Good-til-cancelled (default).
    pub fn gtc(self) -> Self {
        self.time_in_force(TimeInForce::Gtc)
    }

    /// Immediate-or-cancel.
    pub fn ioc(self) -> Self {
        self.time_in_force(TimeInForce::Ioc)
    }

    /// Fill-or-kill.
    pub fn fok(self) -> Self {
        self.time_in_force(TimeInForce::Fok)
    }

    /// Add-liquidity-only (post-only).
    pub fn alo(self) -> Self {
        self.time_in_force(TimeInForce::Alo)
    }

    /// Take-profit shorthand: sets trigger_price and trigger_type in one call.
    pub fn take_profit(self, price: &str) -> Self {
        self.trigger_price(price)
            .trigger_type(TriggerType::TakeProfit)
    }

    /// Stop-loss shorthand: sets trigger_price and trigger_type in one call.
    pub fn stop_loss(self, price: &str) -> Self {
        self.trigger_price(price)
            .trigger_type(TriggerType::StopLoss)
    }
}

// ─── Unified submit (dispatches based on client signing strategy) ────────────

#[cfg(feature = "http")]
impl LimitOrderEnvelope {
    /// Submit this order using the client's signing strategy.
    ///
    /// - **Native**: signs locally with keypair, submits via REST
    /// - **WalletAdapter**: signs via external signer, submits via REST
    /// - **Privy**: sends to backend for signing and submission
    ///
    /// Automatically fills orderbook-derived fields (market, mints, salt) and
    /// scales price/size to raw amounts before signing.
    ///
    /// Returns `Err(SdkError::Validation)` if no signing strategy is set.
    pub async fn submit(
        mut self,
        client: &crate::client::LightconeClient,
        orderbook: &OrderBookPair,
    ) -> Result<crate::domain::order::SubmitOrderResponse, crate::error::SdkError> {
        use crate::shared::signing::SigningStrategy;

        let rules = client
            .orderbooks()
            .decimals(orderbook.orderbook_id.as_str())
            .await?;
        // Pre-fill orderbook-derived fields (market, mints, salt) and validate
        // price/size before the signing strategy runs. This is necessary because
        // the WalletAdapter path calls `payload()` to hash for external signing,
        // and the Privy path reads fields like `get_market()`, both of which
        // happen before `sign()`/`finalize()` where these would otherwise run.
        self.fields.auto_fill_from_orderbook(orderbook)?;
        self.fields
            .apply_rules(&rules, orderbook.orderbook_id.as_str())?;

        // Cache nonce if explicitly provided, or auto-populate from cache
        match self.fields.nonce {
            Some(nonce) => {
                client.set_order_nonce(nonce).await;
            }
            None => {
                self.fields.nonce = Some(client.order_nonce().await.unwrap_or(0));
            }
        }

        let strategy = client.signing_strategy().await.ok_or_else(|| {
            crate::error::SdkError::Validation("signing strategy is not set on the client".into())
        })?;

        match strategy {
            #[cfg(feature = "native-auth")]
            SigningStrategy::Native(keypair) => {
                let request = self.sign(&keypair, orderbook, &rules)?;
                client.orders().submit(&request).await
            }
            SigningStrategy::WalletAdapter(signer) => {
                let hash = self.payload()?.hash_hex();
                let sig_bytes = signer
                    .sign_message(hash.as_bytes())
                    .await
                    .map_err(crate::shared::signing::classify_signer_error)?;
                let sig_bs58 = bs58::encode(&sig_bytes).into_string();
                let request = self.finalize(&sig_bs58, orderbook, &rules)?;
                client.orders().submit(&request).await
            }
        }
    }
}

#[cfg(all(feature = "http", feature = "trigger_orders"))]
impl TriggerOrderEnvelope {
    /// Submit this trigger order using the client's signing strategy.
    ///
    /// - **Native**: signs locally with keypair, submits via REST
    /// - **WalletAdapter**: signs via external signer, submits via REST
    /// - **Privy**: sends to backend for signing and submission
    ///
    /// Automatically fills orderbook-derived fields (market, mints, salt) and
    /// scales price/size to raw amounts before signing.
    ///
    /// Returns `Err(SdkError::Validation)` if no signing strategy is set.
    pub async fn submit(
        mut self,
        client: &crate::client::LightconeClient,
        orderbook: &OrderBookPair,
    ) -> Result<crate::domain::order::TriggerOrderResponse, crate::error::SdkError> {
        use crate::shared::signing::SigningStrategy;

        let rules = client
            .orderbooks()
            .decimals(orderbook.orderbook_id.as_str())
            .await?;
        // Validate every trigger field, including its exact JSON-number
        // representation, before an external wallet is asked to sign.
        self.validated_trigger(&rules)?;
        // Pre-fill orderbook-derived fields (market, mints, salt) and validate
        // price/size before the signing strategy runs. The WalletAdapter path
        // calls `payload()` to hash for external signing before `finalize()`.
        self.fields.auto_fill_from_orderbook(orderbook)?;
        self.fields
            .apply_rules(&rules, orderbook.orderbook_id.as_str())?;

        // Cache nonce if explicitly provided, or auto-populate from cache
        match self.fields.nonce {
            Some(nonce) => {
                client.set_order_nonce(nonce).await;
            }
            None => {
                self.fields.nonce = Some(client.order_nonce().await.unwrap_or(0));
            }
        }

        let strategy = client.signing_strategy().await.ok_or_else(|| {
            crate::error::SdkError::Validation("signing strategy is not set on the client".into())
        })?;

        match strategy {
            #[cfg(feature = "native-auth")]
            SigningStrategy::Native(keypair) => {
                let request = self.sign(&keypair, orderbook, &rules)?;
                client.orders().submit_trigger(&request).await
            }
            SigningStrategy::WalletAdapter(signer) => {
                let hash = self.payload()?.hash_hex();
                let sig_bytes = signer
                    .sign_message(hash.as_bytes())
                    .await
                    .map_err(crate::shared::signing::classify_signer_error)?;
                let sig_bs58 = bs58::encode(&sig_bytes).into_string();
                let request = self.finalize(&sig_bs58, orderbook, &rules)?;
                client.orders().submit_trigger(&request).await
            }
        }
    }
}

// ─── Public accessor for privy helpers ──────────────────────────────────────

impl LimitOrderEnvelope {
    pub fn get_salt(&self) -> Option<u64> {
        self.fields.salt
    }
    pub fn get_maker(&self) -> Option<&Pubkey> {
        self.fields.maker.as_ref()
    }
    pub fn get_market(&self) -> Option<&Pubkey> {
        self.fields.market.as_ref()
    }
    pub fn get_base_mint(&self) -> Option<&Pubkey> {
        self.fields.base_mint.as_ref()
    }
    pub fn get_quote_mint(&self) -> Option<&Pubkey> {
        self.fields.quote_mint.as_ref()
    }
    pub fn get_side(&self) -> Option<OrderSide> {
        self.fields.side
    }
    pub fn get_amount_in(&self) -> Option<u64> {
        self.fields.amount_in
    }
    pub fn get_amount_out(&self) -> Option<u64> {
        self.fields.amount_out
    }
    pub fn get_expiration(&self) -> i64 {
        self.fields.expiration
    }
    pub fn get_nonce(&self) -> Option<u64> {
        self.fields.nonce
    }
    pub fn get_deposit_source(&self) -> Option<DepositSource> {
        self.fields.deposit_source
    }
}

#[cfg(feature = "trigger_orders")]
impl TriggerOrderEnvelope {
    pub fn get_salt(&self) -> Option<u64> {
        self.fields.salt
    }
    pub fn get_maker(&self) -> Option<&Pubkey> {
        self.fields.maker.as_ref()
    }
    pub fn get_market(&self) -> Option<&Pubkey> {
        self.fields.market.as_ref()
    }
    pub fn get_base_mint(&self) -> Option<&Pubkey> {
        self.fields.base_mint.as_ref()
    }
    pub fn get_quote_mint(&self) -> Option<&Pubkey> {
        self.fields.quote_mint.as_ref()
    }
    pub fn get_side(&self) -> Option<OrderSide> {
        self.fields.side
    }
    pub fn get_amount_in(&self) -> Option<u64> {
        self.fields.amount_in
    }
    pub fn get_amount_out(&self) -> Option<u64> {
        self.fields.amount_out
    }
    pub fn get_expiration(&self) -> i64 {
        self.fields.expiration
    }
    pub fn get_nonce(&self) -> Option<u64> {
        self.fields.nonce
    }
    pub fn get_deposit_source(&self) -> Option<DepositSource> {
        self.fields.deposit_source
    }
    pub fn get_time_in_force(&self) -> Option<TimeInForce> {
        self.time_in_force
    }
    pub fn get_trigger_price(&self) -> Option<&str> {
        self.trigger_price.as_deref()
    }
    pub fn get_trigger_type(&self) -> Option<TriggerType> {
        self.trigger_type
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::orderbook::OrderBookPair;
    use crate::shared::ScalingError;

    #[cfg(feature = "native-auth")]
    use solana_signer::Signer;

    #[cfg(all(
        feature = "http",
        feature = "trigger_orders",
        not(target_arch = "wasm32")
    ))]
    use {
        crate::shared::signing::ExternalSigner,
        std::{
            future::Future,
            pin::Pin,
            sync::{
                atomic::{AtomicUsize, Ordering},
                Arc,
            },
        },
        tokio::{
            io::{AsyncReadExt, AsyncWriteExt},
            net::TcpListener,
        },
    };

    fn test_orderbook() -> OrderBookPair {
        OrderBookPair::test_new("test_ob", 6, 6, 0)
    }

    fn test_rules() -> OrderbookRules {
        OrderbookRules {
            orderbook_id: "test_ob".into(),
            base_decimals: 6,
            quote_decimals: 6,
            price_decimals: 6,
            trading_rules: crate::shared::TradingRules {
                base_size_decimals: 6,
                max_price_decimals: 6,
                max_price_significant_figures: 5,
                integer_prices_always_allowed: true,
                price_quantum: "0.000001".into(),
                price_quantum_raw: 1u8.into(),
                base_size_quantum: "0.000001".into(),
                base_size_quantum_raw: 1u8.into(),
            },
        }
    }

    #[test]
    fn test_limit_envelope_payload() {
        let maker = Pubkey::new_unique();
        let market = Pubkey::new_unique();
        let base_mint = Pubkey::new_unique();
        let quote_mint = Pubkey::new_unique();

        let env = LimitOrderEnvelope::new()
            .nonce(1)
            .salt(0)
            .maker(maker)
            .market(market)
            .base_mint(base_mint)
            .quote_mint(quote_mint)
            .bid()
            .amount_in(1_000_000)
            .amount_out(500_000);

        let payload = env.payload().unwrap();
        assert_eq!(payload.nonce, 1);
        assert_eq!(payload.maker, maker);
        assert_eq!(payload.side, OrderSide::Bid);
        assert!(!payload.is_signed());
    }

    #[test]
    #[cfg(feature = "native-auth")]
    fn test_limit_envelope_sign_raw_amounts() {
        let keypair = Keypair::new();
        let maker = keypair.pubkey();
        let ob = test_orderbook();

        let request = LimitOrderEnvelope::new()
            .nonce(1)
            .salt(0)
            .maker(maker)
            .market(Pubkey::new_unique())
            .base_mint(Pubkey::new_unique())
            .quote_mint(Pubkey::new_unique())
            .bid()
            .amount_in(1_000_000)
            .amount_out(500_000)
            .sign(&keypair, &ob, &test_rules())
            .unwrap();

        assert_eq!(request.maker, maker.to_string());
        assert_eq!(request.nonce, 1);
        assert_eq!(request.side, 0); // Bid
        assert_eq!(request.orderbook_id, "test_ob");
        assert_eq!(request.signature.len(), 128);
        assert_eq!(request.time_in_force, None);
        assert_eq!(request.trigger_price, None);
        assert_eq!(request.trigger_type, None);
    }

    #[test]
    #[cfg(feature = "native-auth")]
    fn test_limit_envelope_sign_with_auto_scaling() {
        let keypair = Keypair::new();
        let maker = keypair.pubkey();
        let ob = test_orderbook();

        let request = LimitOrderEnvelope::new()
            .nonce(1)
            .salt(0)
            .maker(maker)
            .market(Pubkey::new_unique())
            .base_mint(Pubkey::new_unique())
            .quote_mint(Pubkey::new_unique())
            .bid()
            .price("0.65")
            .size("100")
            .sign(&keypair, &ob, &test_rules())
            .unwrap();

        // BID: amount_in = quote_lamports = 0.65 * 100 * 10^6 = 65_000_000
        //      amount_out = base_lamports = 100 * 10^6 = 100_000_000
        assert_eq!(request.amount_in, 65_000_000);
        assert_eq!(request.amount_out, 100_000_000);
        assert_eq!(request.signature.len(), 128);
    }

    #[test]
    #[cfg(all(feature = "native-auth", feature = "trigger_orders"))]
    fn test_trigger_envelope_sign() {
        let keypair = Keypair::new();
        let maker = keypair.pubkey();
        let ob = test_orderbook();

        let request = TriggerOrderEnvelope::new()
            .nonce(1)
            .salt(0)
            .maker(maker)
            .market(Pubkey::new_unique())
            .base_mint(Pubkey::new_unique())
            .quote_mint(Pubkey::new_unique())
            .ask()
            .amount_in(500_000)
            .amount_out(1_000_000)
            .take_profit("0.75")
            .gtc()
            .sign(&keypair, &ob, &test_rules())
            .unwrap();

        assert_eq!(request.trigger_price.unwrap().as_str(), "0.75");
        assert_eq!(request.trigger_type, Some(TriggerType::TakeProfit));
        assert_eq!(request.time_in_force, Some(TimeInForce::Gtc));
        assert_eq!(request.side, 1); // Ask
        assert_eq!(request.signature.len(), 128);
    }

    #[test]
    #[cfg(all(feature = "native-auth", feature = "trigger_orders"))]
    fn trigger_price_keeps_exact_json_number() {
        let keypair = Keypair::new();
        let ob = test_orderbook();
        let trigger_price = "9007199254.740993";

        let request = TriggerOrderEnvelope::new()
            .nonce(1)
            .salt(0)
            .maker(keypair.pubkey())
            .market(Pubkey::new_unique())
            .base_mint(Pubkey::new_unique())
            .quote_mint(Pubkey::new_unique())
            .ask()
            .amount_in(500_000)
            .amount_out(1_000_000)
            .take_profit(trigger_price)
            .gtc()
            .sign(&keypair, &ob, &test_rules())
            .unwrap();

        assert_eq!(
            request.trigger_price.as_ref().unwrap().as_str(),
            trigger_price
        );
        assert!(serde_json::to_string(&request)
            .unwrap()
            .contains(r#""trigger_price":9007199254.740993"#));
    }

    #[cfg(all(
        feature = "http",
        feature = "trigger_orders",
        not(target_arch = "wasm32")
    ))]
    struct CountingSigner {
        message_calls: Arc<AtomicUsize>,
    }

    #[cfg(all(
        feature = "http",
        feature = "trigger_orders",
        not(target_arch = "wasm32")
    ))]
    impl ExternalSigner for CountingSigner {
        fn sign_message<'a>(
            &'a self,
            _message: &'a [u8],
        ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, String>> + 'a>> {
            self.message_calls.fetch_add(1, Ordering::SeqCst);
            Box::pin(async { Ok(vec![0; 64]) })
        }

        fn sign_transaction<'a>(
            &'a self,
            _tx_bytes: &'a [u8],
        ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, String>> + 'a>> {
            Box::pin(async { Ok(Vec::new()) })
        }
    }

    #[tokio::test]
    #[cfg(all(
        feature = "http",
        feature = "trigger_orders",
        not(target_arch = "wasm32")
    ))]
    async fn submit_validates_exact_trigger_before_wallet_signing() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        tokio::spawn(async move {
            let (mut socket, _) = listener.accept().await.unwrap();
            let mut request = [0_u8; 4096];
            let bytes_read = socket.read(&mut request).await.unwrap();
            assert!(bytes_read > 0);
            let body = r#"{"status":"success","body":{"orderbook_id":"test_ob","base_decimals":6,"quote_decimals":6,"price_decimals":6,"trading_rules":{"base_size_decimals":6,"max_price_decimals":6,"max_price_significant_figures":5,"integer_prices_always_allowed":true,"price_quantum":"0.000001","price_quantum_raw":"1","base_size_quantum":"0.000001","base_size_quantum_raw":"1"}}}"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            socket.write_all(response.as_bytes()).await.unwrap();
        });

        let message_calls = Arc::new(AtomicUsize::new(0));
        let signer = Arc::new(CountingSigner {
            message_calls: Arc::clone(&message_calls),
        });
        let client = crate::client::LightconeClient::builder()
            .base_url(&format!("http://{address}"))
            .external_signer(signer)
            .build()
            .unwrap();
        let orderbook = test_orderbook();

        // `validate_trigger_price` accepts this exact decimal spelling, but it
        // is not a valid JSON number and cannot be submitted as one.
        assert!(validate_trigger_price("+0.75", 6).is_ok());
        let result = TriggerOrderEnvelope::new()
            .nonce(1)
            .salt(0)
            .maker(Pubkey::new_unique())
            .bid()
            .amount_in(1_000_000)
            .amount_out(500_000)
            .take_profit("+0.75")
            .submit(&client, &orderbook)
            .await;

        assert!(matches!(
            result,
            Err(crate::error::SdkError::Program(SdkError::Scaling(
                ScalingError::TriggerPriceOutOfRange
            )))
        ));
        assert_eq!(message_calls.load(Ordering::SeqCst), 0);
    }

    #[test]
    #[cfg(all(feature = "native-auth", feature = "trigger_orders"))]
    fn test_trigger_envelope_missing_trigger_fields() {
        let keypair = Keypair::new();
        let ob = test_orderbook();

        let result = TriggerOrderEnvelope::new()
            .nonce(1)
            .salt(0)
            .maker(keypair.pubkey())
            .market(Pubkey::new_unique())
            .base_mint(Pubkey::new_unique())
            .quote_mint(Pubkey::new_unique())
            .bid()
            .amount_in(1_000_000)
            .amount_out(500_000)
            .sign(&keypair, &ob, &test_rules());

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("trigger_price"));
    }

    #[test]
    #[cfg(all(feature = "native-auth", feature = "trigger_orders"))]
    fn test_trigger_envelope_stop_loss() {
        use crate::shared::{TimeInForce, TriggerType};

        let keypair = Keypair::new();
        let ob = test_orderbook();

        let request = TriggerOrderEnvelope::new()
            .nonce(1)
            .salt(0)
            .maker(keypair.pubkey())
            .market(Pubkey::new_unique())
            .base_mint(Pubkey::new_unique())
            .quote_mint(Pubkey::new_unique())
            .ask()
            .amount_in(500_000)
            .amount_out(1_000_000)
            .stop_loss("0.30")
            .ioc()
            .sign(&keypair, &ob, &test_rules())
            .unwrap();

        assert_eq!(request.time_in_force, Some(TimeInForce::Ioc));
        assert_eq!(request.trigger_price.unwrap().as_str(), "0.30");
        assert_eq!(request.trigger_type, Some(TriggerType::StopLoss));
    }

    #[test]
    fn test_limit_envelope_zero_amount_in() {
        let result = LimitOrderEnvelope::new()
            .nonce(1)
            .salt(0)
            .maker(Pubkey::new_unique())
            .market(Pubkey::new_unique())
            .base_mint(Pubkey::new_unique())
            .quote_mint(Pubkey::new_unique())
            .bid()
            .amount_in(0)
            .amount_out(500_000)
            .payload();
        assert!(result.is_err());
        assert!(
            result.unwrap_err().to_string().contains("amount_in"),
            "expected error about amount_in"
        );
    }

    #[test]
    #[cfg(feature = "native-auth")]
    fn rejects_rules_for_another_orderbook_before_signing() {
        let keypair = Keypair::new();
        let ob = test_orderbook();
        let mut rules = test_rules();
        rules.orderbook_id = "another_ob".into();

        let result = LimitOrderEnvelope::new()
            .nonce(1)
            .salt(0)
            .maker(keypair.pubkey())
            .market(Pubkey::new_unique())
            .base_mint(Pubkey::new_unique())
            .quote_mint(Pubkey::new_unique())
            .bid()
            .amount_in(1_000_000)
            .amount_out(500_000)
            .sign(&keypair, &ob, &rules);

        assert!(matches!(
            result,
            Err(SdkError::Scaling(ScalingError::OrderbookMismatch {
                ref expected,
                ref actual,
            })) if expected == "test_ob" && actual == "another_ob"
        ));
    }

    #[test]
    fn test_limit_envelope_zero_amount_out() {
        let result = LimitOrderEnvelope::new()
            .nonce(1)
            .salt(0)
            .maker(Pubkey::new_unique())
            .market(Pubkey::new_unique())
            .base_mint(Pubkey::new_unique())
            .quote_mint(Pubkey::new_unique())
            .bid()
            .amount_in(1_000_000)
            .amount_out(0)
            .payload();
        assert!(result.is_err());
        assert!(
            result.unwrap_err().to_string().contains("amount_out"),
            "expected error about amount_out"
        );
    }

    #[test]
    fn test_limit_envelope_nonce_defaults_to_zero() {
        let payload = LimitOrderEnvelope::new()
            .maker(Pubkey::new_unique())
            .market(Pubkey::new_unique())
            .base_mint(Pubkey::new_unique())
            .quote_mint(Pubkey::new_unique())
            .bid()
            .amount_in(1_000_000)
            .amount_out(500_000)
            .payload()
            .unwrap();
        assert_eq!(payload.nonce, 0);
    }

    #[test]
    fn test_limit_envelope_missing_side() {
        let result = LimitOrderEnvelope::new()
            .nonce(1)
            .salt(0)
            .maker(Pubkey::new_unique())
            .market(Pubkey::new_unique())
            .base_mint(Pubkey::new_unique())
            .quote_mint(Pubkey::new_unique())
            .amount_in(1_000_000)
            .amount_out(500_000)
            .payload();
        assert!(result.is_err());
        assert!(
            result.unwrap_err().to_string().contains("side"),
            "expected error about side"
        );
    }

    #[test]
    #[cfg(feature = "native-auth")]
    fn test_limit_envelope_with_deposit_source() {
        let keypair = Keypair::new();
        let maker = keypair.pubkey();
        let ob = test_orderbook();

        let request = LimitOrderEnvelope::new()
            .nonce(1)
            .salt(0)
            .maker(maker)
            .market(Pubkey::new_unique())
            .base_mint(Pubkey::new_unique())
            .quote_mint(Pubkey::new_unique())
            .bid()
            .amount_in(1_000_000)
            .amount_out(500_000)
            .deposit_source(DepositSource::Global)
            .sign(&keypair, &ob, &test_rules())
            .unwrap();

        assert_eq!(request.deposit_source, Some(DepositSource::Global));
    }

    #[test]
    #[cfg(feature = "native-auth")]
    fn test_limit_envelope_deposit_source_none_by_default() {
        let keypair = Keypair::new();
        let ob = test_orderbook();

        let request = LimitOrderEnvelope::new()
            .nonce(1)
            .salt(0)
            .maker(keypair.pubkey())
            .market(Pubkey::new_unique())
            .base_mint(Pubkey::new_unique())
            .quote_mint(Pubkey::new_unique())
            .bid()
            .amount_in(1_000_000)
            .amount_out(500_000)
            .sign(&keypair, &ob, &test_rules())
            .unwrap();

        assert_eq!(request.deposit_source, None);
    }

    #[test]
    fn test_limit_envelope_deposit_source_accessor() {
        let env = LimitOrderEnvelope::new().deposit_source(DepositSource::Market);
        assert_eq!(env.get_deposit_source(), Some(DepositSource::Market));
    }
}
