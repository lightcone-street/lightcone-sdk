//! Wire types for order and user WS messages.

use crate::shared::{serde_util, OrderBookId, PubkeyStr, Side};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::Deserialize;

// ─── WS balance wire types ──────────────────────────────────────────────────

/// Balance for a single conditional token (from WS user updates).
#[derive(Deserialize, Debug, Clone)]
pub struct ConditionalBalance {
    pub outcome_index: i16,
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
#[derive(Deserialize, Debug, Clone)]
pub struct UserSnapshotBalance {
    pub market_pubkey: PubkeyStr,
    pub orderbook_id: OrderBookId,
    pub outcomes: Vec<ConditionalBalance>,
}

// ─── WS order wire types ────────────────────────────────────────────────────

/// WS order update event.
#[derive(Deserialize, Debug, Clone)]
pub struct OrderUpdate {
    pub market_pubkey: PubkeyStr,
    pub orderbook_id: OrderBookId,
    pub timestamp: DateTime<Utc>,
    pub tx_signature: Option<String>,
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
    pub balance: UserOrderUpdateBalance,
}

/// WS order snapshot (initial state on connect).
#[derive(Deserialize, Debug, Clone)]
pub struct UserSnapshotOrder {
    pub market_pubkey: PubkeyStr,
    pub orderbook_id: OrderBookId,
    pub tx_signature: Option<String>,
    pub order_hash: String,
    pub side: Side,
    pub maker_amount: Decimal,
    pub taker_amount: Decimal,
    pub remaining: Decimal,
    pub filled: Decimal,
    pub price: Decimal,
    #[serde(with = "serde_util::timestamp_ms")]
    pub created_at: DateTime<Utc>,
    pub expiration: u64,
    pub base_mint: PubkeyStr,
    pub quote_mint: PubkeyStr,
    pub outcome_index: i16,
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

/// WS user update — tagged enum.
#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "event_type")]
pub enum UserUpdate {
    #[serde(rename = "snapshot")]
    Snapshot(UserSnapshot),
    #[serde(rename = "order")]
    OrderUpdate(OrderUpdate),
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
