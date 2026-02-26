//! Conversions: WS wire types → Order domain types.

use super::wire;
use super::{Order, OrderStatus, TriggerOrder, UserOpenOrders, UserTriggerOrders};
use crate::shared::{OrderBookId, PubkeyStr, SnapshotOrderType, TimeInForce};
use std::collections::HashMap;

impl From<wire::OrderUpdate> for Order {
    fn from(update: wire::OrderUpdate) -> Self {
        Order {
            market_pubkey: update.market_pubkey,
            orderbook_id: update.orderbook_id,
            base_mint: update.order.base_mint,
            quote_mint: update.order.quote_mint,
            order_hash: update.order.order_hash,
            side: update.order.side,
            size: update.order.filled + update.order.remaining,
            price: update.order.price,
            filled_size: update.order.filled,
            remaining_size: update.order.remaining,
            created_at: update.order.created_at,
            tx_signature: update.tx_signature,
            status: update.order.status,
            outcome_index: update.order.outcome_index,
        }
    }
}

/// Convert a limit snapshot order to a domain Order.
impl From<wire::UserSnapshotOrder> for Order {
    fn from(snap: wire::UserSnapshotOrder) -> Self {
        Order {
            market_pubkey: snap.market_pubkey,
            orderbook_id: snap.orderbook_id,
            order_hash: snap.order_hash,
            base_mint: snap.base_mint,
            quote_mint: snap.quote_mint,
            side: snap.side,
            size: snap.filled + snap.remaining,
            price: snap.price,
            filled_size: snap.filled,
            remaining_size: snap.remaining,
            created_at: snap.created_at,
            tx_signature: snap.tx_signature,
            status: snap.status,
            outcome_index: snap.outcome_index,
        }
    }
}

/// Convert a trigger snapshot order (from unified UserSnapshotOrder) to a domain TriggerOrder.
///
/// Requires the snapshot to have `order_type == Trigger` and trigger-specific fields present.
impl TriggerOrder {
    pub fn from_snapshot(snap: &wire::UserSnapshotOrder) -> Option<Self> {
        let trigger_order_id = snap.trigger_order_id.clone()?;
        let trigger_price = snap.trigger_price?;
        let trigger_type = snap.trigger_type?;
        let tif_numeric = snap.tif.unwrap_or(0);
        let time_in_force = match tif_numeric {
            0 => TimeInForce::Gtc,
            1 => TimeInForce::Ioc,
            2 => TimeInForce::Fok,
            3 => TimeInForce::Alo,
            _ => TimeInForce::Gtc,
        };
        Some(TriggerOrder {
            trigger_order_id,
            order_hash: snap.order_hash.clone(),
            market_pubkey: snap.market_pubkey.clone(),
            orderbook_id: snap.orderbook_id.clone(),
            trigger_price,
            trigger_type,
            side: snap.side,
            amount_in: snap.amount_in,
            amount_out: snap.amount_out,
            time_in_force,
            created_at: snap.created_at,
        })
    }
}

/// Build UserOpenOrders + UserTriggerOrders from a unified snapshot orders array.
pub fn split_snapshot_orders(
    orders: Vec<wire::UserSnapshotOrder>,
) -> (UserOpenOrders, UserTriggerOrders) {
    let mut open_orders: HashMap<PubkeyStr, Vec<Order>> = HashMap::new();
    let mut trigger_orders: HashMap<OrderBookId, Vec<TriggerOrder>> = HashMap::new();

    for snap in orders {
        match snap.order_type {
            SnapshotOrderType::Trigger => {
                if let Some(trigger) = TriggerOrder::from_snapshot(&snap) {
                    trigger_orders
                        .entry(trigger.orderbook_id.clone())
                        .or_default()
                        .push(trigger);
                }
            }
            SnapshotOrderType::Limit => {
                if !snap.remaining.is_zero() {
                    open_orders
                        .entry(snap.market_pubkey.clone())
                        .or_default()
                        .push(snap.into());
                }
            }
        }
    }

    (
        UserOpenOrders { orders: open_orders },
        UserTriggerOrders { orders: trigger_orders },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::{OrderBookId, OrderUpdateType, PubkeyStr, Side, SnapshotOrderType, TriggerType};
    use chrono::Utc;
    use rust_decimal::Decimal;

    fn make_limit_snapshot(market: &str, hash: &str, remaining: Decimal) -> wire::UserSnapshotOrder {
        wire::UserSnapshotOrder {
            order_hash: hash.to_string(),
            market_pubkey: PubkeyStr::from(market),
            orderbook_id: OrderBookId::from("ob1"),
            side: Side::Bid,
            amount_in: Decimal::ZERO,
            amount_out: Decimal::ZERO,
            remaining,
            filled: Decimal::ZERO,
            price: Decimal::new(50, 1),
            created_at: Utc::now(),
            expiration: 0,
            base_mint: PubkeyStr::from("b"),
            quote_mint: PubkeyStr::from("q"),
            outcome_index: 0,
            status: OrderStatus::Open,
            order_type: SnapshotOrderType::Limit,
            tx_signature: None,
            trigger_order_id: None,
            trigger_price: None,
            trigger_type: None,
            tif: None,
        }
    }

    fn make_trigger_snapshot_order(id: &str, ob_id: &str) -> wire::UserSnapshotOrder {
        wire::UserSnapshotOrder {
            order_hash: format!("hash-{id}"),
            market_pubkey: PubkeyStr::from("mkt-xyz"),
            orderbook_id: OrderBookId::from(ob_id),
            side: Side::Bid,
            amount_in: Decimal::new(1000, 0),
            amount_out: Decimal::new(500, 0),
            remaining: Decimal::ZERO,
            filled: Decimal::ZERO,
            price: Decimal::ZERO,
            created_at: Utc::now(),
            expiration: 0,
            base_mint: PubkeyStr::from("b"),
            quote_mint: PubkeyStr::from("q"),
            outcome_index: 0,
            status: OrderStatus::Pending,
            order_type: SnapshotOrderType::Trigger,
            tx_signature: None,
            trigger_order_id: Some(id.to_string()),
            trigger_price: Some(Decimal::new(55, 2)),
            trigger_type: Some(TriggerType::TakeProfit),
            tif: Some(0),
        }
    }

    #[test]
    fn test_order_update_conversion() {
        let update = wire::OrderUpdate {
            market_pubkey: PubkeyStr::from("mkt111"),
            orderbook_id: OrderBookId::from("ob_abc"),
            timestamp: Utc::now(),
            tx_signature: Some("sig123".to_string()),
            update_type: OrderUpdateType::Update,
            order: wire::WsOrder {
                order_hash: "hash_xyz".to_string(),
                price: Decimal::new(55, 1),
                is_maker: true,
                remaining: Decimal::new(8, 0),
                filled: Decimal::new(2, 0),
                fill_amount: Decimal::new(2, 0),
                side: Side::Bid,
                created_at: Utc::now(),
                base_mint: PubkeyStr::from("base_mint"),
                quote_mint: PubkeyStr::from("quote_mint"),
                outcome_index: 0,
                status: OrderStatus::Open,
                balance: Some(wire::UserOrderUpdateBalance {
                    outcomes: vec![],
                }),
            },
        };
        let order: Order = update.into();
        assert_eq!(order.order_hash, "hash_xyz");
        assert_eq!(order.size, Decimal::new(10, 0));
        assert_eq!(order.filled_size, Decimal::new(2, 0));
        assert_eq!(order.remaining_size, Decimal::new(8, 0));
        assert_eq!(order.tx_signature, Some("sig123".to_string()));
    }

    #[test]
    fn test_user_snapshot_order_conversion() {
        let snap = make_limit_snapshot("mkt222", "snap_hash", Decimal::new(5, 0));
        let order: Order = snap.into();
        assert_eq!(order.order_hash, "snap_hash");
        assert_eq!(order.market_pubkey.as_str(), "mkt222");
    }

    #[test]
    fn test_trigger_order_from_snapshot() {
        let snap = make_trigger_snapshot_order("trig-123", "ob_test");
        let order = TriggerOrder::from_snapshot(&snap).unwrap();
        assert_eq!(order.trigger_order_id, "trig-123");
        assert_eq!(order.trigger_type, TriggerType::TakeProfit);
        assert_eq!(order.orderbook_id.as_str(), "ob_test");
        assert_eq!(order.amount_in, Decimal::new(1000, 0));
    }

    #[test]
    fn test_split_snapshot_orders() {
        let orders = vec![
            make_limit_snapshot("mkt1", "o1", Decimal::new(1, 0)),
            make_limit_snapshot("mkt1", "o2", Decimal::ZERO), // filled, should be excluded
            make_trigger_snapshot_order("t1", "ob_test"),
            make_trigger_snapshot_order("t2", "ob_test"),
        ];

        let (open, triggers) = split_snapshot_orders(orders);
        assert_eq!(open.orders.len(), 1); // 1 market
        assert_eq!(open.orders.values().next().unwrap().len(), 1); // o1 only (o2 filtered)
        assert_eq!(triggers.len(), 2); // t1 + t2
        assert_eq!(triggers.get(&OrderBookId::from("ob_test")).unwrap().len(), 2);
    }
}
