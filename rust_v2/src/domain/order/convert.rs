//! Conversions: WS wire types → Order domain types.

use super::wire;
use super::{Order, OrderStatus, UserOpenOrders};
use crate::shared::PubkeyStr;
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
            status: OrderStatus::Open,
            outcome_index: update.order.outcome_index,
        }
    }
}

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
            status: OrderStatus::Open,
            outcome_index: snap.outcome_index,
        }
    }
}

impl From<Vec<wire::UserSnapshotOrder>> for UserOpenOrders {
    fn from(orders: Vec<wire::UserSnapshotOrder>) -> Self {
        let mut user_orders: HashMap<PubkeyStr, Vec<Order>> = HashMap::new();
        for order in orders {
            if !order.remaining.is_zero() {
                user_orders
                    .entry(order.market_pubkey.clone())
                    .or_insert_with(Vec::new)
                    .push(order.into());
            }
        }
        UserOpenOrders {
            orders: user_orders,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shared::{OrderBookId, PubkeyStr, Side};
    use chrono::Utc;
    use rust_decimal::Decimal;

    #[test]
    fn test_order_update_conversion() {
        let update = wire::OrderUpdate {
            market_pubkey: PubkeyStr::from("mkt111"),
            orderbook_id: OrderBookId::from("ob_abc"),
            timestamp: Utc::now(),
            tx_signature: Some("sig123".to_string()),
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
                balance: wire::UserOrderUpdateBalance {
                    outcomes: vec![],
                },
            },
        };
        let order: Order = update.into();
        assert_eq!(order.order_hash, "hash_xyz");
        assert_eq!(order.size, Decimal::new(10, 0)); // filled + remaining
        assert_eq!(order.filled_size, Decimal::new(2, 0));
        assert_eq!(order.remaining_size, Decimal::new(8, 0));
        assert_eq!(order.tx_signature, Some("sig123".to_string()));
    }

    #[test]
    fn test_user_snapshot_order_conversion() {
        let snap = wire::UserSnapshotOrder {
            market_pubkey: PubkeyStr::from("mkt222"),
            orderbook_id: OrderBookId::from("ob_def"),
            tx_signature: None,
            order_hash: "snap_hash".to_string(),
            side: Side::Ask,
            maker_amount: Decimal::new(100, 0),
            taker_amount: Decimal::new(50, 0),
            remaining: Decimal::new(5, 0),
            filled: Decimal::new(3, 0),
            price: Decimal::new(60, 1),
            created_at: Utc::now(),
            expiration: 0,
            base_mint: PubkeyStr::from("base"),
            quote_mint: PubkeyStr::from("quote"),
            outcome_index: 1,
        };
        let order: Order = snap.into();
        assert_eq!(order.order_hash, "snap_hash");
        assert_eq!(order.size, Decimal::new(8, 0)); // filled + remaining
        assert_eq!(order.market_pubkey.as_str(), "mkt222");
    }

    #[test]
    fn test_user_open_orders_filters_zero_remaining() {
        let orders = vec![
            wire::UserSnapshotOrder {
                market_pubkey: PubkeyStr::from("mkt1"),
                orderbook_id: OrderBookId::from("ob1"),
                tx_signature: None,
                order_hash: "o1".to_string(),
                side: Side::Bid,
                maker_amount: Decimal::ZERO,
                taker_amount: Decimal::ZERO,
                remaining: Decimal::new(1, 0),
                filled: Decimal::ZERO,
                price: Decimal::new(50, 1),
                created_at: Utc::now(),
                expiration: 0,
                base_mint: PubkeyStr::from("b"),
                quote_mint: PubkeyStr::from("q"),
                outcome_index: 0,
            },
            wire::UserSnapshotOrder {
                market_pubkey: PubkeyStr::from("mkt1"),
                orderbook_id: OrderBookId::from("ob2"),
                tx_signature: None,
                order_hash: "o2".to_string(),
                side: Side::Bid,
                maker_amount: Decimal::ZERO,
                taker_amount: Decimal::ZERO,
                remaining: Decimal::ZERO,
                filled: Decimal::new(10, 0),
                price: Decimal::new(51, 1),
                created_at: Utc::now(),
                expiration: 0,
                base_mint: PubkeyStr::from("b"),
                quote_mint: PubkeyStr::from("q"),
                outcome_index: 0,
            },
        ];
        let uoo: UserOpenOrders = orders.into();
        assert_eq!(uoo.orders.len(), 1);
        assert_eq!(uoo.orders.values().next().unwrap().len(), 1);
    }
}
