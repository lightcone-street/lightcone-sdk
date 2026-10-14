//! Wire types for order and user WS messages.

use super::OrderStatus;
use crate::shared::{serde_util, OrderBookId, OrderUpdateType, PubkeyStr, Side};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// ─── WS balance wire types ──────────────────────────────────────────────────

/// Balance for a single conditional token (from WS user updates).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ConditionalBalance {
    pub outcome_index: i16,
    pub conditional_token: PubkeyStr,
    pub idle: Decimal,
    pub on_book: Decimal,
}

/// WS user market balance update.
#[derive(Deserialize, Debug, Clone)]
pub struct UserBalanceUpdate {
    pub market_pubkey: PubkeyStr,
    pub market_balance: UserMarketBalance,
    pub timestamp: DateTime<Utc>,
}

/// User balances for a single market.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct UserMarketBalance {
    pub market_pubkey: PubkeyStr,
    pub deposit_assets: Vec<UserDepositAssetBalance>,
}

/// User balance for one conditional token/outcome.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct UserOutcomeBalance {
    pub outcome_index: i16,
    pub conditional_token: PubkeyStr,
    pub balance: Decimal,
    pub balance_idle: Decimal,
    pub balance_on_book: Decimal,
}

impl UserOutcomeBalance {
    pub fn is_zero(&self) -> bool {
        !(self.balance_idle > Decimal::ZERO || self.balance_on_book > Decimal::ZERO)
    }
}

/// User balances for a single deposit asset within a market.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct UserDepositAssetBalance {
    pub deposit_asset: PubkeyStr,
    pub outcomes: Vec<UserOutcomeBalance>,
}

/// Global deposit balance for a single mint (used in snapshots).
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct GlobalDepositBalance {
    pub mint: PubkeyStr,
    pub balance: Decimal,
}

/// WS global deposit update event.
#[derive(Deserialize, Debug, Clone)]
pub struct GlobalDepositUpdate {
    pub mint: PubkeyStr,
    pub balance: Decimal,
    pub timestamp: DateTime<Utc>,
}

/// WS nonce update event.
#[derive(Deserialize, Debug, Clone)]
pub struct NonceUpdate {
    pub user_pubkey: PubkeyStr,
    pub new_nonce: u64,
    pub timestamp: DateTime<Utc>,
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

/// Validated fields of a supported limit-order snapshot.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
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

/// Limit order in REST lists and WebSocket account snapshots.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "order_type", rename_all = "lowercase")]
pub enum UserSnapshotOrder {
    Limit {
        #[serde(flatten)]
        common: UserSnapshotOrderCommon,
        #[serde(default)]
        tx_signature: Option<String>,
    },
}

impl UserSnapshotOrder {
    pub fn common(&self) -> &UserSnapshotOrderCommon {
        match self {
            UserSnapshotOrder::Limit { common, .. } => common,
        }
    }
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
    pub market_balances: Vec<UserMarketBalance>,
    #[serde(default)]
    pub global_deposits: Vec<GlobalDepositBalance>,
    #[serde(default)]
    pub notifications: Vec<crate::domain::notification::Notification>,
    #[serde(default)]
    pub nonce: u64,
}

/// Live limit-order update, decoded with the ordinary order payload validation.
#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "order_type", rename_all = "lowercase")]
pub enum OrderEvent {
    Limit(OrderUpdate),
}

/// WS user update — tagged enum.
#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "event_type")]
pub enum UserUpdate {
    #[serde(rename = "snapshot")]
    Snapshot(UserSnapshot),
    #[serde(rename = "order")]
    Order(OrderEvent),
    #[serde(rename = "market_balance_update")]
    BalanceUpdate(UserBalanceUpdate),
    #[serde(rename = "global_deposit_update")]
    GlobalDepositUpdate(GlobalDepositUpdate),
    #[serde(rename = "nonce")]
    NonceUpdate(NonceUpdate),
    #[serde(rename = "notification")]
    Notification(NotificationUpdate),
}

/// WS notification push event.
#[derive(Deserialize, Debug, Clone)]
pub struct NotificationUpdate {
    pub notification: crate::domain::notification::Notification,
}

/// WS auth update.
#[derive(Deserialize, Debug, Clone)]
pub struct Authenticated {
    pub wallet: PubkeyStr,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "status")]
pub enum AuthUpdate {
    #[serde(rename = "authenticated")]
    Authenticated(Authenticated),
    #[serde(rename = "anonymous")]
    Anonymous {
        #[serde(default)]
        reason: Option<String>,
    },
}

// ─── User order fills (REST) ───────────────────────────────────────────────

/// Response from `GET /api/users/order-fills`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserOrderFillsResponse {
    pub orders: Vec<UserOrderFill>,
    pub next_cursor: Option<String>,
    pub has_more: bool,
}

/// An order the user participated in, with nested fill events.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserOrderFill {
    pub order_hash: String,
    pub market_pubkey: PubkeyStr,
    pub orderbook_id: OrderBookId,
    pub side: Side,
    pub role: Role,
    pub price: Decimal,
    pub size: Decimal,
    pub filled_size: Decimal,
    pub remaining_size: Decimal,
    pub base_mint: PubkeyStr,
    pub quote_mint: PubkeyStr,
    pub outcome_index: i16,
    pub status: FillStatus,
    #[serde(with = "serde_util::timestamp_ms")]
    pub created_at: DateTime<Utc>,
    pub fills: Vec<OrderFillEvent>,
}

/// A single fill event within an order.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrderFillEvent {
    pub fill_amount: Decimal,
    pub tx_signature: String,
    #[serde(with = "serde_util::timestamp_ms")]
    pub filled_at: DateTime<Utc>,
}

/// Whether the user was the maker or taker on an order.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Maker,
    Taker,
}

/// Status of a filled order, derived from DB state after the fact.
///
/// Distinct from `OrderStatus` which is the engine's real-time state.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FillStatus {
    Filled,
    Cancelled,
    PartiallyFilled,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::order::{UserOpenLimitOrders, UserOrdersResponse};
    use crate::ws::{Kind, MessageIn};
    use serde_json::{json, Value};

    fn supported_order() -> Value {
        json!({
            "order_type": "limit", "order_hash": "order-1", "market_pubkey": "market-1",
            "orderbook_id": "book-1", "side": "bid", "amount_in": "1.25",
            "amount_out": "2.5", "remaining": "2.5", "filled": "0", "price": "0.5",
            "created_at": 1700000000000_u64, "base_mint": "base", "quote_mint": "quote",
            "outcome_index": 0, "status": "OPEN", "tx_signature": "signature"
        })
    }

    fn account_payload(orders: Vec<Value>) -> Value {
        json!({
            "event_type": "snapshot", "user_pubkey": "wallet", "orders": orders,
            "market_balances": [{"market_pubkey": "market-1", "deposit_assets": [{
                "deposit_asset": "quote", "outcomes": [{"outcome_index": 0,
                "conditional_token": "base", "balance": "1234.56789",
                "balance_idle": "1200.5", "balance_on_book": "34.06789"}]
            }]}],
            "global_deposits": [{"mint": "quote", "balance": "9.25"}],
            "notifications": [{"id": "notice", "notification_type": "global",
                "title": "Notice", "message": "Fixture", "created_at": "2026-01-01T00:00:00Z"}],
            "nonce": 17, "next_cursor": "next-page", "has_more": true
        })
    }

    #[test]
    fn limit_rest_and_ws_snapshots_preserve_account_data() -> Result<(), serde_json::Error> {
        let payload = account_payload(vec![supported_order()]);
        let expected_order: UserSnapshotOrder = serde_json::from_value(supported_order())?;
        let rest: UserOrdersResponse = serde_json::from_value(payload.clone())?;
        assert_eq!(rest.orders, vec![expected_order.clone()]);
        assert_eq!(rest.next_cursor.as_deref(), Some("next-page"));
        assert!(rest.has_more);
        assert_eq!(
            serde_json::to_value(&rest.market_balances)?,
            payload["market_balances"]
        );

        let message: MessageIn = serde_json::from_value(json!({
            "type": "user", "version": 0.1, "data": payload
        }))?;
        let Kind::User(UserUpdate::Snapshot(snapshot)) = message.kind else {
            return Err(<serde_json::Error as serde::de::Error>::custom(
                "expected snapshot",
            ));
        };
        assert_eq!(snapshot.orders, vec![expected_order]);
        assert_eq!(snapshot.market_balances, rest.market_balances);
        assert_eq!(snapshot.nonce, 17);
        assert_eq!(
            serde_json::to_value(snapshot.global_deposits)?,
            payload["global_deposits"]
        );
        assert_eq!(
            serde_json::to_value(snapshot.notifications)?,
            payload["notifications"]
        );
        Ok(())
    }

    #[test]
    fn empty_order_list_keeps_balances_and_pagination() -> Result<(), serde_json::Error> {
        let rest: UserOrdersResponse = serde_json::from_value(account_payload(vec![]))?;
        assert!(rest.orders.is_empty());
        assert_eq!(rest.market_balances.len(), 1);
        assert_eq!(rest.next_cursor.as_deref(), Some("next-page"));
        assert!(rest.has_more);
        Ok(())
    }

    #[test]
    fn malformed_limit_orders_fail_rest_and_snapshot_decoding() {
        for field in ["amount_in", "amount_out", "order_hash"] {
            let mut invalid = supported_order();
            invalid[field] = Value::Null;
            let payload = account_payload(vec![supported_order(), invalid]);
            assert!(serde_json::from_value::<UserOrdersResponse>(payload.clone()).is_err());
            assert!(serde_json::from_value::<MessageIn>(json!({
                "type": "user", "version": 0.1, "data": payload
            }))
            .is_err());
        }
    }

    #[test]
    fn live_limit_order_updates_state() -> Result<(), serde_json::Error> {
        let data = json!({
            "event_type": "order", "order_type": "limit", "market_pubkey": "market-1",
            "orderbook_id": "book-1", "timestamp": "2026-01-01T00:00:00Z", "type": "PLACEMENT",
            "order": {"order_hash": "order-1", "price": "0.5", "is_maker": true,
                "remaining": "2.5", "filled": "0", "fill_amount": "0", "side": "bid",
                "created_at": 1700000000000_u64, "base_mint": "base", "quote_mint": "quote",
                "outcome_index": 0}
        });
        let mut state = UserOpenLimitOrders::new();
        let message: MessageIn = serde_json::from_value(json!({
            "type": "user", "version": 0.1, "data": data
        }))?;
        let Kind::User(UserUpdate::Order(OrderEvent::Limit(update))) = message.kind else {
            return Err(<serde_json::Error as serde::de::Error>::custom(
                "expected order event",
            ));
        };
        state.upsert(&update);
        assert_eq!(
            state
                .get(&"market-1".into(), &"book-1".into())
                .map(Vec::len),
            Some(1)
        );
        Ok(())
    }

    #[test]
    fn malformed_live_limit_order_fails_decoding() {
        for order in [Value::Null, json!({})] {
            let data = json!({
                "event_type": "order", "order_type": "limit", "market_pubkey": "market-1",
                "orderbook_id": "book-1", "timestamp": "2026-01-01T00:00:00Z", "order": order
            });
            assert!(serde_json::from_value::<MessageIn>(json!({
                "type": "user", "version": 0.1, "data": data
            }))
            .is_err());
        }
    }
}
