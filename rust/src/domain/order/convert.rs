//! Conversions: supported WS and REST wire orders into app-owned order state.

use super::wire;
use super::LimitOrder;
use super::UserOpenLimitOrders;
use crate::shared::{OrderBookId, PubkeyStr};
use std::collections::HashMap;

impl From<wire::OrderUpdate> for LimitOrder {
    fn from(update: wire::OrderUpdate) -> Self {
        LimitOrder {
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

/// Preserves snapshot amounts and metadata when constructing an ordinary order.
pub fn limit_snapshot_to_order(
    common: wire::UserSnapshotOrderCommon,
    tx_signature: Option<String>,
) -> LimitOrder {
    LimitOrder {
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

/// Groups supported open orders by market and orderbook, excluding fully filled rows.
pub fn convert_snapshot_orders(orders: Vec<wire::UserSnapshotOrder>) -> UserOpenLimitOrders {
    let mut open_orders: HashMap<PubkeyStr, HashMap<OrderBookId, Vec<LimitOrder>>> = HashMap::new();
    for snapshot in orders {
        let wire::UserSnapshotOrder::Limit {
            common,
            tx_signature,
        } = snapshot;
        if !common.remaining.is_zero() {
            open_orders
                .entry(common.market_pubkey.clone())
                .or_default()
                .entry(common.orderbook_id.clone())
                .or_default()
                .push(limit_snapshot_to_order(common, tx_signature));
        }
    }
    UserOpenLimitOrders {
        orders: open_orders,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::order::OrderStatus;
    use crate::shared::{OrderBookId, OrderUpdateType, PubkeyStr, Side};
    use chrono::Utc;
    use rust_decimal::Decimal;

    fn make_limit_snapshot(
        market: &str,
        hash: &str,
        remaining: Decimal,
    ) -> wire::UserSnapshotOrder {
        wire::UserSnapshotOrder::Limit {
            common: wire::UserSnapshotOrderCommon {
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
            },
            tx_signature: None,
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
        let order: LimitOrder = update.into();
        assert_eq!(order.order_hash, "hash_xyz");
        assert_eq!(order.size, Decimal::new(10, 0));
        assert_eq!(order.filled_size, Decimal::new(2, 0));
        assert_eq!(order.remaining_size, Decimal::new(8, 0));
        assert_eq!(order.tx_signature, Some("sig123".to_string()));
    }

    #[test]
    fn test_limit_snapshot_conversion() {
        let wire::UserSnapshotOrder::Limit {
            common,
            tx_signature,
        } = make_limit_snapshot("mkt222", "snap_hash", Decimal::new(5, 0));
        let order = limit_snapshot_to_order(common, tx_signature);
        assert_eq!(order.order_hash, "snap_hash");
        assert_eq!(order.market_pubkey.as_str(), "mkt222");
    }

    #[test]
    fn snapshot_groups_open_orders_and_excludes_filled_orders() {
        let open = convert_snapshot_orders(vec![
            make_limit_snapshot("mkt1", "o1", Decimal::ONE),
            make_limit_snapshot("mkt1", "o2", Decimal::ZERO),
            make_limit_snapshot("mkt2", "o3", Decimal::ONE),
        ]);
        let orders = open.get(&PubkeyStr::from("mkt1"), &OrderBookId::from("ob1"));
        assert_eq!(orders.map(Vec::len), Some(1));
        assert_eq!(
            orders
                .and_then(|rows| rows.first())
                .map(|order| order.order_hash.as_str()),
            Some("o1")
        );
        assert_eq!(open.orders.len(), 2);
    }
}
