#![doc = include_str!("README.md")]

pub mod client;
mod convert;
pub mod state;
pub mod wire;

use crate::shared::{OrderBookId, PubkeyStr, Side};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

pub use client::{
    CancelAllBody, CancelAllSuccess, CancelBody, CancelSuccess, FillInfo, SubmitOrderResponse,
    SubmitOrderStatus, UserOrdersResponse,
};
pub use convert::convert_snapshot_orders;
pub use state::UserOpenLimitOrders;
pub use wire::{
    ConditionalBalance, FillStatus, GlobalDepositBalance, GlobalDepositUpdate, NonceUpdate,
    NotificationUpdate, OrderEvent, OrderFillEvent, Role, UserBalanceUpdate,
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

/// Supported trade-form selections. Fund transfers are separate wallet operations.
#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum OrderType {
    Limit,
    #[default]
    Market,
    Split,
    Merge,
}

impl OrderType {
    pub fn label(&self) -> &'static str {
        match self {
            OrderType::Limit => "Limit",
            OrderType::Market => "Market",
            OrderType::Split => "Split",
            OrderType::Merge => "Merge",
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

// ─── AnyOrder ───────────────────────────────────────────────────────────────

/// Supported entries rendered by order tables.
#[derive(Clone, PartialEq)]
pub enum AnyOrder {
    Limit(LimitOrder),
}

impl From<LimitOrder> for AnyOrder {
    fn from(order: LimitOrder) -> Self {
        Self::Limit(order)
    }
}

impl Order for AnyOrder {
    fn id(&self) -> &str {
        match self {
            Self::Limit(order) => order.id(),
        }
    }
    fn order_hash(&self) -> &str {
        match self {
            Self::Limit(order) => order.order_hash(),
        }
    }
    fn market_pubkey(&self) -> &PubkeyStr {
        match self {
            Self::Limit(order) => order.market_pubkey(),
        }
    }
    fn orderbook_id(&self) -> &OrderBookId {
        match self {
            Self::Limit(order) => order.orderbook_id(),
        }
    }
    fn side(&self) -> Side {
        match self {
            Self::Limit(order) => order.side(),
        }
    }
    fn created_at(&self) -> DateTime<Utc> {
        match self {
            Self::Limit(order) => order.created_at(),
        }
    }
}

impl AnyOrder {
    /// Converts supported orders to table entries sorted by creation time.
    pub fn vec_from(limit_orders: Vec<LimitOrder>) -> Vec<AnyOrder> {
        let mut entries: Vec<AnyOrder> = limit_orders.into_iter().map(AnyOrder::Limit).collect();
        entries.sort_by(|a, b| Order::created_at(a).cmp(&Order::created_at(b)));
        entries
    }
}
