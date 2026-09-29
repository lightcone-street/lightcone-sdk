#![doc = include_str!("README.md")]

pub mod client;
mod convert;
#[cfg(test)]
mod response_tests;
pub mod state;
pub mod wire;

use crate::domain::position::wire::FundingSource;
#[cfg(feature = "trigger_orders")]
use crate::shared::TriggerType;
use crate::shared::{OrderBookId, PubkeyStr, Side, TimeInForce};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

pub use client::{
    CancelAllBody, CancelAllSuccess, CancelBody, CancelQuantities, CancelStatus, CancelSuccess,
    ClosureAck, FillInfo, SubmitOrderResponse, SubmitOrderStatus, UserOrdersResponse,
};
#[cfg(feature = "trigger_orders")]
pub use client::{CancelTriggerBody, CancelTriggerSuccess, TriggerOrderResponse};
pub use convert::convert_snapshot_orders;
#[cfg(feature = "trigger_orders")]
pub use state::UserTriggerOrders;
pub use state::{ApplyOutcome, UserOpenLimitOrders};
pub use wire::{
    ClosureScope, ClosureUpdate, CommitInfo, InitialCohort, InitialCohortState, NotificationUpdate,
    OrderFillEvent, OrderState, OrderUpdate, RecordedOrderState, RecoveryCompleted, Role,
    UserOrder, UserOrderFill, UserOrderFillsResponse, UserSnapshot, UserSnapshotOrder, UserUpdate,
};

// ─── Order (trait) ──────────────────────────────────────────────────────────

pub trait Order {
    fn id(&self) -> &str;
    fn order_hash(&self) -> &str;
    fn market_pubkey(&self) -> &PubkeyStr;
    fn orderbook_id(&self) -> &OrderBookId;
    fn side(&self) -> Side;
    fn created_at(&self) -> DateTime<Utc>;
}

// ─── OrderType ───────────────────────────────────────────────────────────────

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrderType {
    Limit,
    #[default]
    Market,
    Split,
    Merge,
    Withdraw,
    #[cfg(feature = "trigger_orders")]
    StopLimit,
    #[cfg(feature = "trigger_orders")]
    TakeProfitLimit,
}

impl OrderType {
    pub fn label(&self) -> &'static str {
        match self {
            OrderType::Limit => "Limit",
            OrderType::Market => "Market",
            OrderType::Split => "Split",
            OrderType::Merge => "Merge",
            OrderType::Withdraw => "Withdraw",
            #[cfg(feature = "trigger_orders")]
            OrderType::StopLimit => "Stop Limit",
            #[cfg(feature = "trigger_orders")]
            OrderType::TakeProfitLimit => "Take Profit Limit",
        }
    }
}

impl std::fmt::Display for OrderType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            OrderType::Limit => write!(f, "limit"),
            OrderType::Market => write!(f, "market"),
            OrderType::Split => write!(f, "split"),
            OrderType::Merge => write!(f, "merge"),
            OrderType::Withdraw => write!(f, "withdraw"),
            #[cfg(feature = "trigger_orders")]
            OrderType::StopLimit => write!(f, "stop_limit"),
            #[cfg(feature = "trigger_orders")]
            OrderType::TakeProfitLimit => write!(f, "take_profit_limit"),
        }
    }
}

impl std::str::FromStr for OrderType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "limit" => Ok(OrderType::Limit),
            "market" => Ok(OrderType::Market),
            "split" => Ok(OrderType::Split),
            "merge" => Ok(OrderType::Merge),
            "withdraw" => Ok(OrderType::Withdraw),
            #[cfg(feature = "trigger_orders")]
            "stop_limit" => Ok(OrderType::StopLimit),
            #[cfg(feature = "trigger_orders")]
            "take_profit_limit" => Ok(OrderType::TakeProfitLimit),
            _ => Err(format!("invalid order type: {s}")),
        }
    }
}

// ─── OrderStatus ─────────────────────────────────────────────────────────────

/// Committed lifecycle status of an order.
///
/// Serialized lowercase, as in `GET /api/users/order-fills`. Derived with the
/// backend's precedence: a closure reason wins, then a complete confirmed
/// fill, then pending (unconfirmed) fills, otherwise open.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum OrderStatus {
    /// Resting, or awaiting its first match.
    #[default]
    Open,
    /// Some matched base awaits on-chain confirmation.
    Pending,
    /// The whole original base is confirmed filled.
    Filled,
    /// Stopped resting (cancelled, expired, closure cutoff, or remainder of
    /// an IOC/FOK order); see the closed reason.
    Closed,
}

impl OrderStatus {
    /// Apply the backend's status precedence to committed quantities.
    pub fn derive(
        closed_reason: Option<&str>,
        original_base: Decimal,
        confirmed_base: Decimal,
        pending_base: Decimal,
    ) -> Self {
        if closed_reason.is_some() {
            Self::Closed
        } else if confirmed_base == original_base {
            Self::Filled
        } else if pending_base > Decimal::ZERO {
            Self::Pending
        } else {
            Self::Open
        }
    }
}

// ─── LimitOrder ─────────────────────────────────────────────────────────────

/// A limit order's committed state. Size fields are in base-token units and
/// satisfy `size = filled_size + pending_size + remaining_size + cancelled_size`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LimitOrder {
    pub market_pubkey: PubkeyStr,
    pub orderbook_id: OrderBookId,
    pub base_mint: PubkeyStr,
    pub quote_mint: PubkeyStr,
    pub order_hash: String,
    pub side: Side,
    /// Limit price, quote units per base unit.
    pub price: Decimal,
    /// Original order size.
    pub size: Decimal,
    /// Base filled by confirmed (on-chain authenticated) executions.
    pub filled_size: Decimal,
    /// Base matched but awaiting on-chain confirmation.
    pub pending_size: Decimal,
    /// Base still resting on the book.
    pub remaining_size: Decimal,
    /// Base cancelled (explicitly, by expiry, IOC/FOK remainder, or closure).
    pub cancelled_size: Decimal,
    pub time_in_force: TimeInForce,
    pub funding_source: FundingSource,
    pub closed_reason: Option<String>,
    pub status: OrderStatus,
    /// Engine acceptance sequence; closures cut off by it.
    pub accepted_seq: u64,
    /// Committed revision of this state; newer revisions supersede older ones.
    pub committed_revision: u64,
    pub created_at: DateTime<Utc>,
    /// Unix seconds; `0` means no expiration.
    pub expiration: i64,
}

impl LimitOrder {
    /// True while the order rests on the book or has fills awaiting
    /// confirmation (which may still return to the book if they fail).
    pub fn is_live(&self) -> bool {
        self.remaining_size > Decimal::ZERO || self.pending_size > Decimal::ZERO
    }
}

impl Order for LimitOrder {
    fn id(&self) -> &str {
        &self.order_hash
    }
    fn order_hash(&self) -> &str {
        &self.order_hash
    }
    fn market_pubkey(&self) -> &PubkeyStr {
        &self.market_pubkey
    }
    fn orderbook_id(&self) -> &OrderBookId {
        &self.orderbook_id
    }
    fn side(&self) -> Side {
        self.side
    }
    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}

// ─── TriggerOrder ───────────────────────────────────────────────────────────

#[cfg(feature = "trigger_orders")]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TriggerOrder {
    pub trigger_order_id: String,
    pub order_hash: String,
    pub market_pubkey: PubkeyStr,
    pub orderbook_id: OrderBookId,
    pub trigger_price: Decimal,
    pub trigger_type: TriggerType,
    pub side: Side,
    pub amount_in: Decimal,
    pub amount_out: Decimal,
    pub time_in_force: TimeInForce,
    pub created_at: DateTime<Utc>,
}

#[cfg(feature = "trigger_orders")]
impl Order for TriggerOrder {
    fn id(&self) -> &str {
        &self.trigger_order_id
    }
    fn order_hash(&self) -> &str {
        &self.order_hash
    }
    fn market_pubkey(&self) -> &PubkeyStr {
        &self.market_pubkey
    }
    fn orderbook_id(&self) -> &OrderBookId {
        &self.orderbook_id
    }
    fn side(&self) -> Side {
        self.side.clone()
    }
    fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}

#[cfg(feature = "trigger_orders")]
impl TriggerOrder {
    pub fn limit_price(&self) -> Option<Decimal> {
        match self.side {
            Side::Ask if self.amount_in > Decimal::ZERO => Some(self.amount_out / self.amount_in),
            Side::Bid if self.amount_out > Decimal::ZERO => Some(self.amount_in / self.amount_out),
            _ => None,
        }
    }
}

// ─── AnyOrder ───────────────────────────────────────────────────────────────

#[derive(Clone, PartialEq)]
pub enum AnyOrder {
    Limit(LimitOrder),
    #[cfg(feature = "trigger_orders")]
    Trigger(TriggerOrder),
}

impl From<LimitOrder> for AnyOrder {
    fn from(order: LimitOrder) -> Self {
        Self::Limit(order)
    }
}

#[cfg(feature = "trigger_orders")]
impl From<TriggerOrder> for AnyOrder {
    fn from(order: TriggerOrder) -> Self {
        Self::Trigger(order)
    }
}

impl Order for AnyOrder {
    fn id(&self) -> &str {
        match self {
            Self::Limit(order) => order.id(),
            #[cfg(feature = "trigger_orders")]
            Self::Trigger(order) => order.id(),
        }
    }
    fn order_hash(&self) -> &str {
        match self {
            Self::Limit(order) => order.order_hash(),
            #[cfg(feature = "trigger_orders")]
            Self::Trigger(order) => order.order_hash(),
        }
    }
    fn market_pubkey(&self) -> &PubkeyStr {
        match self {
            Self::Limit(order) => order.market_pubkey(),
            #[cfg(feature = "trigger_orders")]
            Self::Trigger(order) => order.market_pubkey(),
        }
    }
    fn orderbook_id(&self) -> &OrderBookId {
        match self {
            Self::Limit(order) => order.orderbook_id(),
            #[cfg(feature = "trigger_orders")]
            Self::Trigger(order) => order.orderbook_id(),
        }
    }
    fn side(&self) -> Side {
        match self {
            Self::Limit(order) => order.side(),
            #[cfg(feature = "trigger_orders")]
            Self::Trigger(order) => order.side(),
        }
    }
    fn created_at(&self) -> DateTime<Utc> {
        match self {
            Self::Limit(order) => order.created_at(),
            #[cfg(feature = "trigger_orders")]
            Self::Trigger(order) => order.created_at(),
        }
    }
}

impl AnyOrder {
    #[cfg(feature = "trigger_orders")]
    pub fn vec_from(
        limit_orders: Vec<LimitOrder>,
        trigger_orders: Vec<TriggerOrder>,
    ) -> Vec<AnyOrder> {
        let mut entries: Vec<AnyOrder> = Vec::new();
        for order in limit_orders.into_iter() {
            entries.push(AnyOrder::Limit(order));
        }
        for trigger_order in trigger_orders.into_iter() {
            entries.push(AnyOrder::Trigger(trigger_order));
        }
        entries.sort_by(|a, b| Order::created_at(a).cmp(&Order::created_at(b)));
        entries
    }

    #[cfg(not(feature = "trigger_orders"))]
    pub fn vec_from(limit_orders: Vec<LimitOrder>) -> Vec<AnyOrder> {
        let mut entries: Vec<AnyOrder> = limit_orders.into_iter().map(AnyOrder::Limit).collect();
        entries.sort_by(|a, b| Order::created_at(a).cmp(&Order::created_at(b)));
        entries
    }
}
