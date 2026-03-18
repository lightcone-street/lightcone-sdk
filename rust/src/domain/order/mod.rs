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
    CancelAllBody, CancelAllResponse, CancelAllSuccess, CancelBody, CancelResponse, CancelSuccess,
    CancelTriggerBody, CancelTriggerResponse, CancelTriggerSuccess, FillInfo, PlaceResponse,
    SubmitOrderResponse, TriggerOrderResponse, TriggerSubmitResponse, UserOrdersResponse,
};
pub use convert::split_snapshot_orders;
pub use state::{UserOpenOrders, UserTriggerOrders};
pub use wire::{
    ConditionalBalance, GlobalDepositBalance, NotificationUpdate, OrderEvent, TriggerOrderUpdate,
    UserSnapshotBalance, UserSnapshotOrder, UserSnapshotOrderCommon,
};

// ─── OrderType ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OrderType {
    Limit,
    Market,
    Deposit,
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

// ─── Order ───────────────────────────────────────────────────────────────────

/// A validated, domain-level order.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Order {
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

// ─── TriggerOrder ───────────────────────────────────────────────────────────

/// A validated, domain-level trigger order (take-profit / stop-loss).
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

impl TriggerOrder {
    /// Derive the human-readable limit price from raw lamport amounts.
    ///
    /// For Ask: amount_in = base_lamports, amount_out = quote_lamports
    ///   price = (amount_out / 10^quote_decimals) / (amount_in / 10^base_decimals)
    ///         = amount_out * 10^base_decimals / (amount_in * 10^quote_decimals)
    ///
    /// For Bid: amount_in = quote_lamports, amount_out = base_lamports
    ///   price = (amount_in / 10^quote_decimals) / (amount_out / 10^base_decimals)
    ///         = amount_in * 10^base_decimals / (amount_out * 10^quote_decimals)
    pub fn limit_price(&self, base_decimals: u8, quote_decimals: u8) -> Option<Decimal> {
        let base_mult = Decimal::from(10u64.pow(base_decimals as u32));
        let quote_mult = Decimal::from(10u64.pow(quote_decimals as u32));

        match self.side {
            Side::Ask if self.amount_in > Decimal::ZERO => {
                Some(self.amount_out * base_mult / (self.amount_in * quote_mult))
            }
            Side::Bid if self.amount_out > Decimal::ZERO => {
                Some(self.amount_in * base_mult / (self.amount_out * quote_mult))
            }
            _ => None,
        }
    }
}
