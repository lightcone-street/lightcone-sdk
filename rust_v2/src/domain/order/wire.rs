//! Wire types for order and user WS messages.

use super::OrderStatus;
use crate::shared::{serde_util, OrderBookId, OrderUpdateType, PubkeyStr, Side, TimeInForce, TriggerStatus, TriggerResultStatus, TriggerType, TriggerUpdateType};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// ─── WS balance wire types ──────────────────────────────────────────────────

/// Balance for a single conditional token (from WS user updates).
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConditionalBalance {
    pub outcome_index: i16,
    #[serde(alias = "conditional_token")]
    pub mint: PubkeyStr,
    pub idle: Decimal,
    pub on_book: Decimal,
}

/// WS user balance update.
#[derive(Deserialize, Debug, Clone)]
pub struct UserBalanceUpdate {
    pub market_pubkey: PubkeyStr,
    pub orderbook_id: OrderBookId,
    pub balance: BalanceUpdateOutcomes,
    pub timestamp: DateTime<Utc>,
}

/// WS balance update payload.
#[derive(Deserialize, Debug, Clone)]
pub struct BalanceUpdateOutcomes {
    pub outcomes: Vec<ConditionalBalance>,
}

/// WS user snapshot balance.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserSnapshotBalance {
    pub market_pubkey: PubkeyStr,
    pub orderbook_id: OrderBookId,
    pub outcomes: Vec<ConditionalBalance>,
}

// ─── WS order wire types ────────────────────────────────────────────────────

/// WS order update event (limit orders).
#[derive(Deserialize, Debug, Clone)]
pub struct OrderUpdate {
    pub market_pubkey: PubkeyStr,
    pub orderbook_id: OrderBookId,
    pub timestamp: DateTime<Utc>,
    pub tx_signature: Option<String>,
    /// "PLACEMENT", "UPDATE", or "CANCELLATION".
    #[serde(rename = "type", default)]
    pub update_type: OrderUpdateType,
    pub order: WsOrder,
}

/// Individual order within a WS update.
#[derive(Deserialize, Debug, Clone)]
pub struct WsOrder {
    pub order_hash: String,
    pub price: Decimal,
    pub is_maker: bool,
    pub remaining: Decimal,
    pub filled: Decimal,
    pub fill_amount: Decimal,
    pub side: Side,
    #[serde(with = "serde_util::timestamp_ms")]
    pub created_at: DateTime<Utc>,
    pub base_mint: PubkeyStr,
    pub quote_mint: PubkeyStr,
    pub outcome_index: i16,
    #[serde(default)]
    pub status: OrderStatus,
    /// Balance is absent on cancellation events.
    #[serde(default)]
    pub balance: Option<UserOrderUpdateBalance>,
}

/// Fields shared by both limit and trigger order snapshots.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UserSnapshotOrderCommon {
    pub order_hash: String,
    pub market_pubkey: PubkeyStr,
    pub orderbook_id: OrderBookId,
    pub side: Side,
    #[serde(alias = "maker_amount")]
    pub amount_in: Decimal,
    #[serde(alias = "taker_amount")]
    pub amount_out: Decimal,
    #[serde(default)]
    pub remaining: Decimal,
    #[serde(default)]
    pub filled: Decimal,
    #[serde(default)]
    pub price: Decimal,
    #[serde(with = "serde_util::timestamp_ms")]
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub expiration: u64,
    pub base_mint: PubkeyStr,
    pub quote_mint: PubkeyStr,
    pub outcome_index: i16,
    #[serde(default)]
    pub status: OrderStatus,
}

/// Order snapshot — tagged enum discriminated by `order_type`.
///
/// Used in REST `GET /api/users/orders` and WS user snapshots.
/// The backend returns limit and trigger orders in the same array,
/// distinguished by the `order_type` field.
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "order_type", rename_all = "lowercase")]
pub enum UserSnapshotOrder {
    Limit {
        #[serde(flatten)]
        common: UserSnapshotOrderCommon,
        #[serde(default)]
        tx_signature: Option<String>,
    },
    Trigger {
        #[serde(flatten)]
        common: UserSnapshotOrderCommon,
        trigger_order_id: String,
        trigger_price: Decimal,
        trigger_type: TriggerType,
        #[serde(default, with = "serde_util::tif_numeric_opt", skip_serializing_if = "Option::is_none")]
        time_in_force: Option<TimeInForce>,
    },
}

/// Balance information attached to an order update.
#[derive(Deserialize, Debug, Clone)]
pub struct UserOrderUpdateBalance {
    pub outcomes: Vec<ConditionalBalance>,
}

/// WS user snapshot.
#[derive(Deserialize, Debug, Clone)]
pub struct UserSnapshot {
    pub orders: Vec<UserSnapshotOrder>,
    pub balances: std::collections::HashMap<OrderBookId, UserSnapshotBalance>,
}

// ─── Trigger order wire types ───────────────────────────────────────────────

/// Trigger order WS update event on `user_events` channel.
#[derive(Debug, Clone, Deserialize)]
pub struct TriggerOrderUpdate {
    pub trigger_order_id: String,
    #[serde(default)]
    pub user_pubkey: PubkeyStr,
    pub market_pubkey: PubkeyStr,
    pub orderbook_id: OrderBookId,
    pub trigger_price: Decimal,
    pub trigger_above: bool,
    pub status: TriggerStatus,
    /// Uppercase version of status: "TRIGGERED", "FAILED", or "EXPIRED".
    #[serde(rename = "type", default)]
    pub update_type: TriggerUpdateType,
    pub order_hash: String,
    pub side: Side,
    #[serde(default, with = "serde_util::empty_string_as_none")]
    pub result_status: Option<TriggerResultStatus>,
    #[serde(default)]
    pub result_filled: Decimal,
    #[serde(default)]
    pub result_remaining: Decimal,
    pub timestamp: DateTime<Utc>,
}

/// WS order event — two-level dispatch on `order_type`.
///
/// Both limit and trigger order updates arrive as `event_type: "order"`,
/// discriminated by `order_type: "limit"` or `"trigger"`.
#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "order_type")]
pub enum OrderEvent {
    #[serde(rename = "limit")]
    Limit(OrderUpdate),
    #[serde(rename = "trigger")]
    Trigger(TriggerOrderUpdate),
}

/// WS user update — tagged enum.
#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "event_type")]
pub enum UserUpdate {
    #[serde(rename = "snapshot")]
    Snapshot(UserSnapshot),
    #[serde(rename = "order")]
    Order(OrderEvent),
    #[serde(rename = "balance_update")]
    BalanceUpdate(UserBalanceUpdate),
}

/// WS auth update.
#[derive(Deserialize, Debug, Clone)]
pub struct Authenticated {
    pub wallet: PubkeyStr,
}

#[derive(Deserialize, Debug, Clone)]
pub struct AuthFailed {
    pub code: String,
    pub message: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "status")]
pub enum AuthUpdate {
    #[serde(rename = "authenticated")]
    Authenticated(Authenticated),
    #[serde(rename = "anonymous")]
    Anonymous,
    #[serde(rename = "failed")]
    Failed(AuthFailed),
}
