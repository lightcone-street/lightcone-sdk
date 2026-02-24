//! Fluent builder for creating and signing orders.

use rust_decimal::Decimal;
use solana_pubkey::Pubkey;

#[cfg(feature = "native-auth")]
use solana_keypair::Keypair;

use crate::program::orders::SignedOrder;
use crate::program::types::OrderSide;
use crate::shared::scaling::{scale_price_size, OrderbookDecimals, ScalingError};
use crate::shared::{TimeInForce, TriggerType};
#[cfg(feature = "native-auth")]
use crate::program::error::SdkError;
#[cfg(feature = "native-auth")]
use crate::shared::SubmitOrderRequest;

/// Builder for creating orders with a fluent API.
///
/// Provides a convenient way to construct, sign, and convert orders
/// for API submission in a single chain of method calls.
///
/// # Example
///
/// ```rust,ignore
/// use lightcone_sdk::prelude::*;
///
/// let request = OrderBuilder::new()
///     .maker(maker_pubkey)
///     .market(market_pubkey)
///     .base_mint(yes_token)
///     .quote_mint(usdc)
///     .bid()
///     .nonce(5)
///     .amount_in(1_000_000)
///     .amount_out(500_000)
///     .build_and_sign(&keypair)
///     .to_submit_request("orderbook_id");
/// ```
#[derive(Debug, Clone, Default)]
pub struct OrderBuilder {
    nonce: Option<u32>,
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
    tif: Option<TimeInForce>,
    trigger_price: Option<f64>,
    trigger_type: Option<TriggerType>,
}

impl OrderBuilder {
    /// Create a new order builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the nonce (required).
    ///
    /// The nonce must be >= the user's on-chain nonce for the order to be valid.
    pub fn nonce(mut self, nonce: u32) -> Self {
        self.nonce = Some(nonce);
        self
    }

    /// Set the maker pubkey (required).
    ///
    /// This is the public key of the order creator.
    pub fn maker(mut self, maker: Pubkey) -> Self {
        self.maker = Some(maker);
        self
    }

    /// Set the market pubkey (required).
    pub fn market(mut self, market: Pubkey) -> Self {
        self.market = Some(market);
        self
    }

    /// Set the base mint / token being bought or sold (required).
    pub fn base_mint(mut self, base_mint: Pubkey) -> Self {
        self.base_mint = Some(base_mint);
        self
    }

    /// Set the quote mint / payment token (required).
    pub fn quote_mint(mut self, quote_mint: Pubkey) -> Self {
        self.quote_mint = Some(quote_mint);
        self
    }

    /// Set as a bid order (buy base with quote).
    pub fn bid(mut self) -> Self {
        self.side = Some(OrderSide::Bid);
        self
    }

    /// Set as an ask order (sell base for quote).
    pub fn ask(mut self) -> Self {
        self.side = Some(OrderSide::Ask);
        self
    }

    /// Set the side directly.
    pub fn side(mut self, side: OrderSide) -> Self {
        self.side = Some(side);
        self
    }

    /// Set the amount the maker gives.
    pub fn amount_in(mut self, amount: u64) -> Self {
        self.amount_in = Some(amount);
        self
    }

    /// Set the amount the maker wants to receive.
    pub fn amount_out(mut self, amount: u64) -> Self {
        self.amount_out = Some(amount);
        self
    }

    /// Set expiration timestamp (0 = no expiration).
    pub fn expiration(mut self, expiration: i64) -> Self {
        self.expiration = expiration;
        self
    }

    /// Set time-in-force policy (GTC, IOC, FOK, ALO).
    /// Omit for backend default (GTC).
    pub fn tif(mut self, tif: TimeInForce) -> Self {
        self.tif = Some(tif);
        self
    }

    /// Set trigger price for conditional orders.
    /// Must be paired with `trigger_type()`.
    pub fn trigger_price(mut self, price: f64) -> Self {
        self.trigger_price = Some(price);
        self
    }

    /// Set trigger type (TakeProfit or StopLoss).
    /// Must be paired with `trigger_price()`.
    pub fn trigger_type(mut self, trigger_type: TriggerType) -> Self {
        self.trigger_type = Some(trigger_type);
        self
    }

    /// Good-til-cancelled (default). Shorthand for `.tif(TimeInForce::Gtc)`.
    pub fn gtc(self) -> Self { self.tif(TimeInForce::Gtc) }

    /// Immediate-or-cancel. Shorthand for `.tif(TimeInForce::Ioc)`.
    pub fn ioc(self) -> Self { self.tif(TimeInForce::Ioc) }

    /// Fill-or-kill. Shorthand for `.tif(TimeInForce::Fok)`.
    pub fn fok(self) -> Self { self.tif(TimeInForce::Fok) }

    /// Add-liquidity-only (post-only). Shorthand for `.tif(TimeInForce::Alo)`.
    pub fn alo(self) -> Self { self.tif(TimeInForce::Alo) }

    /// Take-profit trigger. Shorthand for `.trigger_price(price).trigger_type(TriggerType::TakeProfit)`.
    pub fn take_profit(self, price: f64) -> Self {
        self.trigger_price(price).trigger_type(TriggerType::TakeProfit)
    }

    /// Stop-loss trigger. Shorthand for `.trigger_price(price).trigger_type(TriggerType::StopLoss)`.
    pub fn stop_loss(self, price: f64) -> Self {
        self.trigger_price(price).trigger_type(TriggerType::StopLoss)
    }

    /// Build an unsigned SignedOrder.
    ///
    /// The returned order has an all-zero signature and must be signed
    /// before submission.
    ///
    /// # Panics
    ///
    /// Panics if required fields are missing.
    pub fn build(self) -> SignedOrder {
        let amount_in = self.amount_in.expect("amount_in is required");
        let amount_out = self.amount_out.expect("amount_out is required");
        assert!(amount_in > 0, "amount_in must be greater than 0");
        assert!(amount_out > 0, "amount_out must be greater than 0");

        SignedOrder {
            nonce: self.nonce.expect("nonce is required"),
            maker: self.maker.expect("maker is required"),
            market: self.market.expect("market is required"),
            base_mint: self.base_mint.expect("base_mint is required"),
            quote_mint: self.quote_mint.expect("quote_mint is required"),
            side: self.side.expect("side is required (call .bid() or .ask())"),
            amount_in: amount_in,
            amount_out: amount_out,
            expiration: self.expiration,
            signature: [0u8; 64],
        }
    }

    /// Build and sign the order with the given keypair.
    ///
    /// Returns a signed SignedOrder ready for API submission.
    ///
    /// # Panics
    ///
    /// Panics if required fields are missing.
    #[cfg(feature = "native-auth")]
    pub fn build_and_sign(self, keypair: &Keypair) -> SignedOrder {
        let mut order = self.build();
        order.sign(keypair);
        order
    }

    /// Build, sign, and convert directly to a SubmitOrderRequest.
    ///
    /// # Arguments
    ///
    /// * `keypair` - Keypair to sign the order with
    /// * `orderbook_id` - Target orderbook ID
    ///
    /// # Errors
    ///
    /// Returns `SdkError` if required fields are missing or signing fails.
    #[cfg(feature = "native-auth")]
    pub fn to_submit_request(
        self,
        keypair: &Keypair,
        orderbook_id: impl Into<String>,
    ) -> Result<SubmitOrderRequest, SdkError> {
        let tif = self.tif;
        let trigger_price = self.trigger_price;
        let trigger_type = self.trigger_type;
        self.build_and_sign(keypair)
            .to_submit_request_with_options(orderbook_id, tif, trigger_price, trigger_type)
    }

    // =========================================================================
    // Auto-scaling: price/size -> amount_in/amount_out
    // =========================================================================

    /// Set price as a human-readable string (e.g., "0.65" quote per base).
    pub fn price(mut self, price: &str) -> Self {
        self.price_raw = Some(price.to_string());
        self
    }

    /// Set size as a human-readable string (e.g., "100" base tokens).
    pub fn size(mut self, size: &str) -> Self {
        self.size_raw = Some(size.to_string());
        self
    }

    /// Convert price/size strings into amount_in/amount_out using orderbook decimals.
    ///
    /// Call this after `.price()`, `.size()`, and `.bid()`/`.ask()`, then use
    /// any existing build method (`build()`, `build_and_sign()`, `to_submit_request()`).
    pub fn apply_scaling(mut self, decimals: &OrderbookDecimals) -> Result<Self, ScalingError> {
        let price_str = self
            .price_raw
            .as_deref()
            .expect("price() is required for apply_scaling");
        let size_str = self
            .size_raw
            .as_deref()
            .expect("size() is required for apply_scaling");

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
            .expect("side is required (call .bid() or .ask()) for apply_scaling");

        let scaled = scale_price_size(price, size, side, decimals)?;
        self.amount_in = Some(scaled.amount_in);
        self.amount_out = Some(scaled.amount_out);
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "native-auth")]
    use solana_signer::Signer;

    #[test]
    #[cfg(feature = "native-auth")]
    fn test_order_builder_basic() {
        let keypair = Keypair::new();
        let maker = keypair.pubkey();
        let market = Pubkey::new_unique();
        let base_mint = Pubkey::new_unique();
        let quote_mint = Pubkey::new_unique();

        let order = OrderBuilder::new()
            .nonce(1)
            .maker(maker)
            .market(market)
            .base_mint(base_mint)
            .quote_mint(quote_mint)
            .bid()
            .amount_in(1_000_000)
            .amount_out(500_000)
            .build_and_sign(&keypair);

        assert_eq!(order.nonce, 1);
        assert_eq!(order.maker, maker);
        assert_eq!(order.market, market);
        assert_eq!(order.base_mint, base_mint);
        assert_eq!(order.quote_mint, quote_mint);
        assert_eq!(order.side, OrderSide::Bid);
        assert_eq!(order.amount_in, 1_000_000);
        assert_eq!(order.amount_out, 500_000);
        assert!(order.is_signed());
    }

    #[test]
    #[cfg(feature = "native-auth")]
    fn test_order_builder_to_submit_request() {
        let keypair = Keypair::new();
        let maker = keypair.pubkey();
        let market = Pubkey::new_unique();
        let base_mint = Pubkey::new_unique();
        let quote_mint = Pubkey::new_unique();

        let request = OrderBuilder::new()
            .nonce(1)
            .maker(maker)
            .market(market)
            .base_mint(base_mint)
            .quote_mint(quote_mint)
            .ask()
            .amount_in(500_000)
            .amount_out(1_000_000)
            .to_submit_request(&keypair, "test_orderbook")
            .unwrap();

        assert_eq!(request.maker, maker.to_string());
        assert_eq!(request.nonce, 1);
        assert_eq!(request.market_pubkey, market.to_string());
        assert_eq!(request.base_token, base_mint.to_string());
        assert_eq!(request.quote_token, quote_mint.to_string());
        assert_eq!(request.side, 1); // Ask
        assert_eq!(request.amount_in, 500_000);
        assert_eq!(request.amount_out, 1_000_000);
        assert_eq!(request.orderbook_id, "test_orderbook");
        assert_eq!(request.signature.len(), 128); // 64 bytes = 128 hex chars
    }

    #[test]
    fn test_order_builder_unsigned() {
        #[cfg(feature = "native-auth")]
        let keypair = Keypair::new();
        #[cfg(feature = "native-auth")]
        let maker = keypair.pubkey();
        #[cfg(not(feature = "native-auth"))]
        let maker = Pubkey::new_unique();

        let order = OrderBuilder::new()
            .nonce(1)
            .maker(maker)
            .market(Pubkey::new_unique())
            .base_mint(Pubkey::new_unique())
            .quote_mint(Pubkey::new_unique())
            .bid()
            .amount_in(1_000_000)
            .amount_out(500_000)
            .build();

        assert!(!order.is_signed());
    }

    #[test]
    #[cfg(feature = "native-auth")]
    #[should_panic(expected = "nonce is required")]
    fn test_order_builder_missing_nonce() {
        let keypair = Keypair::new();
        OrderBuilder::new()
            .maker(keypair.pubkey())
            .market(Pubkey::new_unique())
            .base_mint(Pubkey::new_unique())
            .quote_mint(Pubkey::new_unique())
            .bid()
            .amount_in(1_000_000)
            .amount_out(500_000)
            .build_and_sign(&keypair);
    }

    #[test]
    #[should_panic(expected = "amount_in must be greater than 0")]
    fn test_order_builder_zero_amount_in() {
        OrderBuilder::new()
            .nonce(1)
            .maker(Pubkey::new_unique())
            .market(Pubkey::new_unique())
            .base_mint(Pubkey::new_unique())
            .quote_mint(Pubkey::new_unique())
            .bid()
            .amount_in(0)
            .amount_out(500_000)
            .build();
    }

    #[test]
    #[should_panic(expected = "amount_out must be greater than 0")]
    fn test_order_builder_zero_amount_out() {
        OrderBuilder::new()
            .nonce(1)
            .maker(Pubkey::new_unique())
            .market(Pubkey::new_unique())
            .base_mint(Pubkey::new_unique())
            .quote_mint(Pubkey::new_unique())
            .bid()
            .amount_in(1_000_000)
            .amount_out(0)
            .build();
    }

    #[test]
    #[cfg(feature = "native-auth")]
    #[should_panic(expected = "side is required")]
    fn test_order_builder_missing_side() {
        let keypair = Keypair::new();
        OrderBuilder::new()
            .nonce(1)
            .maker(keypair.pubkey())
            .market(Pubkey::new_unique())
            .base_mint(Pubkey::new_unique())
            .quote_mint(Pubkey::new_unique())
            .amount_in(1_000_000)
            .amount_out(500_000)
            .build_and_sign(&keypair);
    }

    #[test]
    #[cfg(feature = "native-auth")]
    fn test_order_builder_with_tif() {
        use crate::shared::TimeInForce;

        let keypair = Keypair::new();
        let request = OrderBuilder::new()
            .nonce(1)
            .maker(keypair.pubkey())
            .market(Pubkey::new_unique())
            .base_mint(Pubkey::new_unique())
            .quote_mint(Pubkey::new_unique())
            .bid()
            .amount_in(1_000_000)
            .amount_out(500_000)
            .ioc()
            .to_submit_request(&keypair, "ob")
            .unwrap();

        assert_eq!(request.tif, Some(TimeInForce::Ioc));
        assert_eq!(request.trigger_price, None);
        assert_eq!(request.trigger_type, None);
    }

    #[test]
    #[cfg(feature = "native-auth")]
    fn test_order_builder_with_trigger() {
        use crate::shared::TriggerType;

        let keypair = Keypair::new();
        let request = OrderBuilder::new()
            .nonce(1)
            .maker(keypair.pubkey())
            .market(Pubkey::new_unique())
            .base_mint(Pubkey::new_unique())
            .quote_mint(Pubkey::new_unique())
            .ask()
            .amount_in(500_000)
            .amount_out(1_000_000)
            .take_profit(0.75)
            .to_submit_request(&keypair, "ob")
            .unwrap();

        assert_eq!(request.trigger_price, Some(0.75));
        assert_eq!(request.trigger_type, Some(TriggerType::TakeProfit));
    }

    #[test]
    #[cfg(feature = "native-auth")]
    fn test_order_builder_with_tif_and_trigger() {
        use crate::shared::{TimeInForce, TriggerType};

        let keypair = Keypair::new();
        let request = OrderBuilder::new()
            .nonce(1)
            .maker(keypair.pubkey())
            .market(Pubkey::new_unique())
            .base_mint(Pubkey::new_unique())
            .quote_mint(Pubkey::new_unique())
            .ask()
            .amount_in(500_000)
            .amount_out(1_000_000)
            .gtc()
            .stop_loss(0.30)
            .to_submit_request(&keypair, "ob")
            .unwrap();

        assert_eq!(request.tif, Some(TimeInForce::Gtc));
        assert_eq!(request.trigger_price, Some(0.30));
        assert_eq!(request.trigger_type, Some(TriggerType::StopLoss));
    }

    #[test]
    #[cfg(feature = "native-auth")]
    fn test_order_builder_trigger_validation() {
        let keypair = Keypair::new();

        // trigger_price without trigger_type should fail
        let result = OrderBuilder::new()
            .nonce(1)
            .maker(keypair.pubkey())
            .market(Pubkey::new_unique())
            .base_mint(Pubkey::new_unique())
            .quote_mint(Pubkey::new_unique())
            .bid()
            .amount_in(1_000_000)
            .amount_out(500_000)
            .trigger_price(0.75)
            .to_submit_request(&keypair, "ob");

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("trigger_type"));
    }

    #[test]
    #[cfg(feature = "native-auth")]
    fn test_order_builder_defaults_no_tif_trigger() {
        let keypair = Keypair::new();
        let request = OrderBuilder::new()
            .nonce(1)
            .maker(keypair.pubkey())
            .market(Pubkey::new_unique())
            .base_mint(Pubkey::new_unique())
            .quote_mint(Pubkey::new_unique())
            .bid()
            .amount_in(1_000_000)
            .amount_out(500_000)
            .to_submit_request(&keypair, "ob")
            .unwrap();

        assert_eq!(request.tif, None);
        assert_eq!(request.trigger_price, None);
        assert_eq!(request.trigger_type, None);
    }
}
