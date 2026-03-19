//! Order envelope types: fluent builders that produce signed SubmitOrderRequests.

use rust_decimal::Decimal;
use solana_pubkey::Pubkey;

#[cfg(feature = "native-auth")]
use solana_keypair::Keypair;

use crate::domain::orderbook::OrderBookPair;
use crate::program::error::SdkError;
use crate::program::orders::OrderPayload;
use crate::program::types::OrderSide;
use crate::shared::scaling::{align_price_to_tick, scale_price_size, ScalingError};
use crate::shared::{DepositSource, SubmitOrderRequest, TimeInForce, TriggerType};

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
    fn to_payload(&self) -> OrderPayload {
        let amount_in = self.amount_in.expect("amount_in is required");
        let amount_out = self.amount_out.expect("amount_out is required");
        assert!(amount_in > 0, "amount_in must be greater than 0");
        assert!(amount_out > 0, "amount_out must be greater than 0");

        OrderPayload {
            nonce: self.nonce.expect("nonce is required"),
            salt: self.salt.expect("salt is required"),
            maker: self.maker.expect("maker is required"),
            market: self.market.expect("market is required"),
            base_mint: self.base_mint.expect("base_mint is required"),
            quote_mint: self.quote_mint.expect("quote_mint is required"),
            side: self.side.expect("side is required (call .bid() or .ask())"),
            amount_in,
            amount_out,
            expiration: self.expiration,
            signature: [0u8; 64],
        }
    }

    /// Auto-scale price/size to raw amounts if the user provided human-readable
    /// strings but not pre-computed amounts. Skips if amounts are already set.
    fn auto_scale(&mut self, orderbook: &OrderBookPair) -> Result<(), ScalingError> {
        if self.amount_in.is_some() || self.amount_out.is_some() {
            return Ok(());
        }

        let price_str = self
            .price_raw
            .as_deref()
            .expect("either price()+size() or amount_in()+amount_out() is required");
        let size_str = self
            .size_raw
            .as_deref()
            .expect("either price()+size() or amount_in()+amount_out() is required");

        let price: Decimal =
            price_str
                .parse()
                .map_err(|e: rust_decimal::Error| ScalingError::InvalidDecimal {
                    input: price_str.to_string(),
                    reason: e.to_string(),
                })?;

        let size: Decimal =
            size_str
                .parse()
                .map_err(|e: rust_decimal::Error| ScalingError::InvalidDecimal {
                    input: size_str.to_string(),
                    reason: e.to_string(),
                })?;

        let side = self
            .side
            .expect("side is required (call .bid() or .ask())");

        let decimals = orderbook.decimals();
        let aligned_price = align_price_to_tick(price, &decimals);
        let scaled = scale_price_size(aligned_price, size, side, &decimals)?;
        self.amount_in = Some(scaled.amount_in);
        self.amount_out = Some(scaled.amount_out);
        Ok(())
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
    fn payload(&self) -> OrderPayload;

    /// Sign and produce a `SubmitOrderRequest`. Consumes the envelope.
    ///
    /// If `price()` and `size()` were set, scaling is applied automatically
    /// using the orderbook's decimals. If `amount_in()` and `amount_out()`
    /// were set directly, those raw values are used as-is.
    #[cfg(feature = "native-auth")]
    fn sign(
        self,
        keypair: &Keypair,
        orderbook: &OrderBookPair,
    ) -> Result<SubmitOrderRequest, SdkError>;

    /// Apply an external wallet-adapter signature and produce a `SubmitOrderRequest`.
    /// Consumes the envelope.
    ///
    /// Same auto-scaling behavior as `sign()`.
    fn finalize(
        self,
        sig_bs58: &str,
        orderbook: &OrderBookPair,
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

        fn payload(&self) -> OrderPayload {
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
/// # Example (via client builder — recommended)
///
/// ```rust,ignore
/// let request = client.orders().limit_order().await
///     .maker(maker_pubkey)
///     .market(market_pubkey)
///     .base_mint(yes_token)
///     .quote_mint(usdc)
///     .bid()
///     .nonce(5)
///     .price("0.55")
///     .size("100")
///     .sign(&keypair, &orderbook)?;
/// ```
///
/// # Example (standalone)
///
/// ```rust,ignore
/// let request = LimitOrderEnvelope::new()
///     .maker(maker_pubkey)
///     .market(market_pubkey)
///     .base_mint(yes_token)
///     .quote_mint(usdc)
///     .bid()
///     .nonce(5)
///     .amount_in(1_000_000)
///     .amount_out(500_000)
///     .sign(&keypair, &orderbook)?;
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
    ) -> Result<SubmitOrderRequest, SdkError> {
        self.fields.auto_scale(orderbook)?;
        let mut payload = self.fields.to_payload();
        payload.sign(keypair);
        payload.to_submit_request(
            orderbook.orderbook_id.as_str(),
            self.time_in_force, None, None,
            self.fields.deposit_source,
        )
    }

    fn finalize(
        mut self,
        sig_bs58: &str,
        orderbook: &OrderBookPair,
    ) -> Result<SubmitOrderRequest, SdkError> {
        self.fields.auto_scale(orderbook)?;
        let mut payload = self.fields.to_payload();
        payload.apply_signature(sig_bs58.to_string())?;
        payload.to_submit_request(
            orderbook.orderbook_id.as_str(),
            self.time_in_force, None, None,
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
/// # Example (via client builder — recommended)
///
/// ```rust,ignore
/// let request = client.orders().trigger_order().await
///     .maker(maker_pubkey)
///     .market(market_pubkey)
///     .base_mint(yes_token)
///     .quote_mint(usdc)
///     .ask()
///     .nonce(5)
///     .price("0.55")
///     .size("100")
///     .take_profit(0.75)
///     .gtc()
///     .sign(&keypair, &orderbook)?;
/// ```
///
/// # Example (standalone)
///
/// ```rust,ignore
/// let request = TriggerOrderEnvelope::new()
///     .maker(maker_pubkey)
///     // ... same chain as above
///     .sign(&keypair, &orderbook)?;
/// ```
#[derive(Debug, Clone, Default)]
pub struct TriggerOrderEnvelope {
    fields: OrderFields,
    time_in_force: Option<TimeInForce>,
    trigger_price: Option<f64>,
    trigger_type: Option<TriggerType>,
}

impl OrderEnvelope for TriggerOrderEnvelope {
    impl_base_methods!(TriggerOrderEnvelope);

    #[cfg(feature = "native-auth")]
    fn sign(
        mut self,
        keypair: &Keypair,
        orderbook: &OrderBookPair,
    ) -> Result<SubmitOrderRequest, SdkError> {
        let trigger_price = self.trigger_price.ok_or_else(|| {
            SdkError::MissingField("trigger_price is required for trigger orders".into())
        })?;
        let trigger_type = self.trigger_type.ok_or_else(|| {
            SdkError::MissingField("trigger_type is required for trigger orders".into())
        })?;

        self.fields.auto_scale(orderbook)?;
        let mut payload = self.fields.to_payload();
        payload.sign(keypair);
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
    ) -> Result<SubmitOrderRequest, SdkError> {
        let trigger_price = self.trigger_price.ok_or_else(|| {
            SdkError::MissingField("trigger_price is required for trigger orders".into())
        })?;
        let trigger_type = self.trigger_type.ok_or_else(|| {
            SdkError::MissingField("trigger_type is required for trigger orders".into())
        })?;

        self.fields.auto_scale(orderbook)?;
        let mut payload = self.fields.to_payload();
        payload.apply_signature(sig_bs58.to_string())?;
        payload.to_submit_request(
            orderbook.orderbook_id.as_str(),
            self.time_in_force,
            Some(trigger_price),
            Some(trigger_type),
            self.fields.deposit_source,
        )
    }
}

impl TriggerOrderEnvelope {
    /// Set time-in-force policy (GTC, IOC, FOK, ALO).
    pub fn time_in_force(mut self, tif: TimeInForce) -> Self {
        self.time_in_force = Some(tif);
        self
    }

    /// Set trigger price for the conditional order.
    pub fn trigger_price(mut self, price: f64) -> Self {
        self.trigger_price = Some(price);
        self
    }

    /// Set trigger type (TakeProfit or StopLoss).
    pub fn trigger_type(mut self, trigger_type: TriggerType) -> Self {
        self.trigger_type = Some(trigger_type);
        self
    }

    /// Good-til-cancelled (default).
    pub fn gtc(self) -> Self { self.time_in_force(TimeInForce::Gtc) }

    /// Immediate-or-cancel.
    pub fn ioc(self) -> Self { self.time_in_force(TimeInForce::Ioc) }

    /// Fill-or-kill.
    pub fn fok(self) -> Self { self.time_in_force(TimeInForce::Fok) }

    /// Add-liquidity-only (post-only).
    pub fn alo(self) -> Self { self.time_in_force(TimeInForce::Alo) }

    /// Take-profit shorthand: sets trigger_price and trigger_type in one call.
    pub fn take_profit(self, price: f64) -> Self {
        self.trigger_price(price).trigger_type(TriggerType::TakeProfit)
    }

    /// Stop-loss shorthand: sets trigger_price and trigger_type in one call.
    pub fn stop_loss(self, price: f64) -> Self {
        self.trigger_price(price).trigger_type(TriggerType::StopLoss)
    }
}

// ─── Public accessor for privy helpers ──────────────────────────────────────

impl LimitOrderEnvelope {
    pub fn get_salt(&self) -> Option<u64> { self.fields.salt }
    pub fn get_maker(&self) -> Option<&Pubkey> { self.fields.maker.as_ref() }
    pub fn get_market(&self) -> Option<&Pubkey> { self.fields.market.as_ref() }
    pub fn get_base_mint(&self) -> Option<&Pubkey> { self.fields.base_mint.as_ref() }
    pub fn get_quote_mint(&self) -> Option<&Pubkey> { self.fields.quote_mint.as_ref() }
    pub fn get_side(&self) -> Option<OrderSide> { self.fields.side }
    pub fn get_amount_in(&self) -> Option<u64> { self.fields.amount_in }
    pub fn get_amount_out(&self) -> Option<u64> { self.fields.amount_out }
    pub fn get_expiration(&self) -> i64 { self.fields.expiration }
    pub fn get_nonce(&self) -> Option<u64> { self.fields.nonce }
    pub fn get_deposit_source(&self) -> Option<DepositSource> { self.fields.deposit_source }
}

impl TriggerOrderEnvelope {
    pub fn get_salt(&self) -> Option<u64> { self.fields.salt }
    pub fn get_maker(&self) -> Option<&Pubkey> { self.fields.maker.as_ref() }
    pub fn get_market(&self) -> Option<&Pubkey> { self.fields.market.as_ref() }
    pub fn get_base_mint(&self) -> Option<&Pubkey> { self.fields.base_mint.as_ref() }
    pub fn get_quote_mint(&self) -> Option<&Pubkey> { self.fields.quote_mint.as_ref() }
    pub fn get_side(&self) -> Option<OrderSide> { self.fields.side }
    pub fn get_amount_in(&self) -> Option<u64> { self.fields.amount_in }
    pub fn get_amount_out(&self) -> Option<u64> { self.fields.amount_out }
    pub fn get_expiration(&self) -> i64 { self.fields.expiration }
    pub fn get_nonce(&self) -> Option<u64> { self.fields.nonce }
    pub fn get_deposit_source(&self) -> Option<DepositSource> { self.fields.deposit_source }
    pub fn get_time_in_force(&self) -> Option<TimeInForce> { self.time_in_force }
    pub fn get_trigger_price(&self) -> Option<f64> { self.trigger_price }
    pub fn get_trigger_type(&self) -> Option<TriggerType> { self.trigger_type }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::orderbook::OrderBookPair;

    #[cfg(feature = "native-auth")]
    use solana_signer::Signer;

    fn test_orderbook() -> OrderBookPair {
        OrderBookPair::test_new("test_ob", 6, 6, 0)
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

        let payload = env.payload();
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
            .sign(&keypair, &ob)
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
            .sign(&keypair, &ob)
            .unwrap();

        // BID: amount_in = quote_lamports = 0.65 * 100 * 10^6 = 65_000_000
        //      amount_out = base_lamports = 100 * 10^6 = 100_000_000
        assert_eq!(request.amount_in, 65_000_000);
        assert_eq!(request.amount_out, 100_000_000);
        assert_eq!(request.signature.len(), 128);
    }

    #[test]
    #[cfg(feature = "native-auth")]
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
            .take_profit(0.75)
            .gtc()
            .sign(&keypair, &ob)
            .unwrap();

        assert_eq!(request.trigger_price, Some(0.75));
        assert_eq!(request.trigger_type, Some(TriggerType::TakeProfit));
        assert_eq!(request.time_in_force, Some(TimeInForce::Gtc));
        assert_eq!(request.side, 1); // Ask
        assert_eq!(request.signature.len(), 128);
    }

    #[test]
    #[cfg(feature = "native-auth")]
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
            .sign(&keypair, &ob);

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("trigger_price"));
    }

    #[test]
    #[cfg(feature = "native-auth")]
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
            .stop_loss(0.30)
            .ioc()
            .sign(&keypair, &ob)
            .unwrap();

        assert_eq!(request.time_in_force, Some(TimeInForce::Ioc));
        assert_eq!(request.trigger_price, Some(0.30));
        assert_eq!(request.trigger_type, Some(TriggerType::StopLoss));
    }

    #[test]
    #[should_panic(expected = "amount_in must be greater than 0")]
    fn test_limit_envelope_zero_amount_in() {
        LimitOrderEnvelope::new()
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
    }

    #[test]
    #[should_panic(expected = "amount_out must be greater than 0")]
    fn test_limit_envelope_zero_amount_out() {
        LimitOrderEnvelope::new()
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
    }

    #[test]
    #[should_panic(expected = "nonce is required")]
    fn test_limit_envelope_missing_nonce() {
        LimitOrderEnvelope::new()
            .maker(Pubkey::new_unique())
            .market(Pubkey::new_unique())
            .base_mint(Pubkey::new_unique())
            .quote_mint(Pubkey::new_unique())
            .bid()
            .amount_in(1_000_000)
            .amount_out(500_000)
            .payload();
    }

    #[test]
    #[should_panic(expected = "side is required")]
    fn test_limit_envelope_missing_side() {
        LimitOrderEnvelope::new()
            .nonce(1)
            .salt(0)
            .maker(Pubkey::new_unique())
            .market(Pubkey::new_unique())
            .base_mint(Pubkey::new_unique())
            .quote_mint(Pubkey::new_unique())
            .amount_in(1_000_000)
            .amount_out(500_000)
            .payload();
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
            .sign(&keypair, &ob)
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
            .sign(&keypair, &ob)
            .unwrap();

        assert_eq!(request.deposit_source, None);
    }

    #[test]
    fn test_limit_envelope_deposit_source_accessor() {
        let env = LimitOrderEnvelope::new()
            .deposit_source(DepositSource::Market);
        assert_eq!(env.get_deposit_source(), Some(DepositSource::Market));
    }
}
