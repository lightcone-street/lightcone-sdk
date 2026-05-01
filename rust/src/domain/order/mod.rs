#![doc = include_str!("README.md")]

pub mod client;
mod convert;
pub mod state;
pub mod wire;

use crate::shared::{OrderBookId, PubkeyStr, Side, TimeInForce, TriggerType};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

pub use client::{
    CancelAllBody, CancelAllSuccess, CancelBody, CancelSuccess, CancelTriggerBody,
    CancelTriggerSuccess, FillInfo, SubmitOrderResponse, TriggerOrderResponse, UserOrdersResponse,
};
pub use convert::split_snapshot_orders;
pub use state::{UserOpenLimitOrders, UserTriggerOrders};
pub use wire::{
    ConditionalBalance, FillStatus, GlobalDepositBalance, GlobalDepositUpdate, NonceUpdate,
    NotificationUpdate, OrderEvent, OrderFillEvent, Role, TriggerOrderUpdate, UserOrderFill,
    UserOrderFillsResponse, UserSnapshotBalance, UserSnapshotOrder, UserSnapshotOrderCommon,
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderType {
    Limit,
    Market,
    Deposit,
    Merge,
    Withdraw,
    StopLimit,
    TakeProfitLimit,
}

impl std::fmt::Display for OrderType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            OrderType::Limit => write!(f, "Limit"),
            OrderType::Market => write!(f, "Market"),
            OrderType::Deposit => write!(f, "Deposit"),
            OrderType::Merge => write!(f, "Merge"),
            OrderType::Withdraw => write!(f, "Withdraw"),
            OrderType::StopLimit => write!(f, "Stop Limit"),
            OrderType::TakeProfitLimit => write!(f, "Take Profit Limit"),
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
    Trigger(TriggerOrder),
}

impl From<LimitOrder> for AnyOrder {
    fn from(order: LimitOrder) -> Self {
        Self::Limit(order)
    }
}

impl From<TriggerOrder> for AnyOrder {
    fn from(order: TriggerOrder) -> Self {
        Self::Trigger(order)
    }
}

impl Order for AnyOrder {
    fn id(&self) -> &str {
        match self {
            Self::Limit(order) => order.id(),
            Self::Trigger(order) => order.id(),
        }
    }
    fn order_hash(&self) -> &str {
        match self {
            Self::Limit(order) => order.order_hash(),
            Self::Trigger(order) => order.order_hash(),
        }
    }
    fn market_pubkey(&self) -> &PubkeyStr {
        match self {
            Self::Limit(order) => order.market_pubkey(),
            Self::Trigger(order) => order.market_pubkey(),
        }
    }
    fn orderbook_id(&self) -> &OrderBookId {
        match self {
            Self::Limit(order) => order.orderbook_id(),
            Self::Trigger(order) => order.orderbook_id(),
        }
    }
    fn side(&self) -> Side {
        match self {
            Self::Limit(order) => order.side(),
            Self::Trigger(order) => order.side(),
        }
    }
    fn created_at(&self) -> DateTime<Utc> {
        match self {
            Self::Limit(order) => order.created_at(),
            Self::Trigger(order) => order.created_at(),
        }
    }
}

impl AnyOrder {
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
}
