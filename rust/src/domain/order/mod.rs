#![doc = include_str!("README.md")]

pub mod client;
mod convert;
pub mod state;
pub mod wire;

use crate::shared::{OrderBookId, PubkeyStr, Side};
#[cfg(feature = "trigger_orders")]
use crate::shared::{TimeInForce, TriggerType};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

pub use client::{
    CancelAllBody, CancelAllSuccess, CancelBody, CancelSuccess, FillInfo, SubmitOrderResponse,
    SubmitOrderStatus, UserOrdersResponse,
};
#[cfg(feature = "trigger_orders")]
pub use client::{CancelTriggerBody, CancelTriggerSuccess, TriggerOrderResponse};
pub use convert::convert_snapshot_orders;
pub use state::UserOpenLimitOrders;
#[cfg(feature = "trigger_orders")]
pub use state::UserTriggerOrders;
pub use wire::{
    ConditionalBalance, FillStatus, GlobalDepositBalance, GlobalDepositUpdate, NonceUpdate,
    NotificationUpdate, OrderEvent, OrderFillEvent, Role, TriggerOrderUpdate, UserBalanceUpdate,
    UserDepositAssetBalance, UserMarketBalance, UserOrderFill, UserOrderFillsResponse,
    UserOutcomeBalance, UserSnapshotOrder, UserSnapshotOrderCommon, UserUpdate,
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

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum OrderStatus {
    #[default]
    Open,
    Matching,
    Cancelled,
    Filled,
    Pending,
    Expired,
}

// ─── LimitOrder ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LimitOrder {
    pub market_pubkey: PubkeyStr,
    pub orderbook_id: OrderBookId,
    pub tx_signature: Option<String>,
    pub base_mint: PubkeyStr,
    pub quote_mint: PubkeyStr,
    pub order_hash: String,
    pub side: Side,
    pub size: Decimal,
    pub price: Decimal,
    pub filled_size: Decimal,
    pub remaining_size: Decimal,
    pub created_at: DateTime<Utc>,
    pub status: OrderStatus,
    pub outcome_index: i16,
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
        self.side.clone()
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
