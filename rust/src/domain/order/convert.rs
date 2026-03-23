//! Conversions: WS wire types → Order domain types.

use super::wire;
use super::{Order, TriggerOrder, UserOpenOrders, UserTriggerOrders};
use crate::shared::{OrderBookId, PubkeyStr, TimeInForce};
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

/// Convert a limit snapshot (from the tagged enum) to a domain Order.
pub fn limit_snapshot_to_order(
    common: wire::UserSnapshotOrderCommon,
    tx_signature: Option<String>,
) -> Order {
    Order {
        market_pubkey: common.market_pubkey,
        orderbook_id: common.orderbook_id,
        order_hash: common.order_hash,
        base_mint: common.base_mint,
        quote_mint: common.quote_mint,
        side: common.side,
        size: common.filled + common.remaining,
        price: common.price,
        filled_size: common.filled,
        remaining_size: common.remaining,
        created_at: common.created_at,
        tx_signature,
        status: common.status,
        outcome_index: common.outcome_index,
    }
}

/// Convert a trigger snapshot (from the tagged enum) to a domain TriggerOrder.
pub fn trigger_snapshot_to_order(
    common: wire::UserSnapshotOrderCommon,
    trigger_order_id: String,
    trigger_price: rust_decimal::Decimal,
    trigger_type: crate::shared::TriggerType,
    time_in_force: Option<TimeInForce>,
) -> TriggerOrder {
    TriggerOrder {
        trigger_order_id,
        order_hash: common.order_hash,
        market_pubkey: common.market_pubkey,
        orderbook_id: common.orderbook_id,
        trigger_price,
        trigger_type,
        side: common.side,
        amount_in: common.amount_in,
        amount_out: common.amount_out,
        time_in_force: time_in_force.unwrap_or_default(),
        created_at: common.created_at,
    }
}

/// Build UserOpenOrders + UserTriggerOrders from a unified snapshot orders array.
pub fn split_snapshot_orders(
    orders: Vec<wire::UserSnapshotOrder>,
) -> (UserOpenOrders, UserTriggerOrders) {
    let mut open_orders: HashMap<PubkeyStr, Vec<Order>> = HashMap::new();
    let mut trigger_orders: HashMap<OrderBookId, Vec<TriggerOrder>> = HashMap::new();

    for snap in orders {
        match snap {
            wire::UserSnapshotOrder::Limit {
                common,
                tx_signature,
            } => {
                if !common.remaining.is_zero() {
                    let market = common.market_pubkey.clone();
                    open_orders
                        .entry(market)
                        .or_default()
                        .push(limit_snapshot_to_order(common, tx_signature));
                }
            }
            wire::UserSnapshotOrder::Trigger {
                common,
                trigger_order_id,
                trigger_price,
                trigger_type,
                time_in_force,
            } => {
                let ob_id = common.orderbook_id.clone();
                trigger_orders
                    .entry(ob_id)
                    .or_default()
                    .push(trigger_snapshot_to_order(
                        common,
                        trigger_order_id,
                        trigger_price,
                        trigger_type,
                        time_in_force,
                    ));
            }
        }
    }

    (
        UserOpenOrders {
            orders: open_orders,
        },
        UserTriggerOrders {
            orders: trigger_orders,
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::order::OrderStatus;
    use crate::shared::{OrderBookId, OrderUpdateType, PubkeyStr, Side, TriggerType};
    use chrono::Utc;
    use rust_decimal::Decimal;

    fn make_common(market: &str, hash: &str, remaining: Decimal) -> wire::UserSnapshotOrderCommon {
        wire::UserSnapshotOrderCommon {
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
        }
    }

    fn make_limit_snapshot(
        market: &str,
        hash: &str,
        remaining: Decimal,
    ) -> wire::UserSnapshotOrder {
        wire::UserSnapshotOrder::Limit {
            common: make_common(market, hash, remaining),
            tx_signature: None,
        }
    }

    fn make_trigger_snapshot(id: &str, ob_id: &str) -> wire::UserSnapshotOrder {
        wire::UserSnapshotOrder::Trigger {
            common: wire::UserSnapshotOrderCommon {
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
            },
            trigger_order_id: id.to_string(),
            trigger_price: Decimal::new(55, 2),
            trigger_type: TriggerType::TakeProfit,
            time_in_force: None,
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
                balance: Some(wire::UserOrderUpdateBalance { outcomes: vec![] }),
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
    fn test_limit_snapshot_conversion() {
        let snap = make_limit_snapshot("mkt222", "snap_hash", Decimal::new(5, 0));
        if let wire::UserSnapshotOrder::Limit {
            common,
            tx_signature,
        } = snap
        {
            let order = limit_snapshot_to_order(common, tx_signature);
            assert_eq!(order.order_hash, "snap_hash");
            assert_eq!(order.market_pubkey.as_str(), "mkt222");
        } else {
            panic!("expected Limit variant");
        }
    }

    #[test]
    fn test_trigger_snapshot_conversion() {
        let snap = make_trigger_snapshot("trig-123", "ob_test");
        if let wire::UserSnapshotOrder::Trigger {
            common,
            trigger_order_id,
            trigger_price,
            trigger_type,
            time_in_force,
        } = snap
        {
            let order = trigger_snapshot_to_order(
                common,
                trigger_order_id,
                trigger_price,
                trigger_type,
                time_in_force,
            );
            assert_eq!(order.trigger_order_id, "trig-123");
            assert_eq!(order.trigger_type, TriggerType::TakeProfit);
            assert_eq!(order.orderbook_id.as_str(), "ob_test");
            assert_eq!(order.amount_in, Decimal::new(1000, 0));
            assert_eq!(order.time_in_force, TimeInForce::Gtc);
        } else {
            panic!("expected Trigger variant");
        }
    }

    #[test]
    fn test_split_snapshot_orders() {
        let orders = vec![
            make_limit_snapshot("mkt1", "o1", Decimal::new(1, 0)),
            make_limit_snapshot("mkt1", "o2", Decimal::ZERO),
            make_trigger_snapshot("t1", "ob_test"),
            make_trigger_snapshot("t2", "ob_test"),
        ];

        let (open, triggers) = split_snapshot_orders(orders);
        assert_eq!(open.orders.len(), 1);
        assert_eq!(open.orders.values().next().unwrap().len(), 1);
        assert_eq!(triggers.len(), 2);
        assert_eq!(
            triggers.get(&OrderBookId::from("ob_test")).unwrap().len(),
            2
        );
    }
}
