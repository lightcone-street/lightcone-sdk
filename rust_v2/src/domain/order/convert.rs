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
