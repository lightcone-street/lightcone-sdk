//! Order state containers — app-owned, SDK-provided update logic.

use crate::shared::{OrderBookId, PubkeyStr};

use super::wire;
use super::LimitOrder;
use std::collections::HashMap;

// ─── UserOpenLimitOrders ────────────────────────────────────────────────────

pub struct UserOpenLimitOrders {
    pub orders: HashMap<PubkeyStr, HashMap<OrderBookId, Vec<LimitOrder>>>,
}

impl UserOpenLimitOrders {
    pub fn new() -> Self {
        Self {
            orders: HashMap::new(),
        }
    }

    pub fn get(&self, market: &PubkeyStr, orderbook_id: &OrderBookId) -> Option<&Vec<LimitOrder>> {
        self.orders.get(market)?.get(orderbook_id)
    }

    pub fn get_by_market(
        &self,
        market: &PubkeyStr,
    ) -> Option<&HashMap<OrderBookId, Vec<LimitOrder>>> {
        self.orders.get(market)
    }

    pub fn upsert(&mut self, update: &wire::OrderUpdate) {
        let orderbook_orders = self
            .orders
            .entry(update.market_pubkey.clone())
            .or_default()
            .entry(update.orderbook_id.clone())
            .or_default();

        orderbook_orders.retain(|order| order.order_hash != update.order.order_hash);
        orderbook_orders.push(update.clone().into());
    }

    pub fn remove(&mut self, order_hash: &str) {
        for by_orderbook in self.orders.values_mut() {
            for orders in by_orderbook.values_mut() {
                orders.retain(|order| order.order_hash != order_hash);
            }
        }
    }

    pub fn clear(&mut self) {
        self.orders.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.orders
            .values()
            .all(|by_orderbook| by_orderbook.values().all(|orders| orders.is_empty()))
    }
}

impl Default for UserOpenLimitOrders {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::order::OrderStatus;
    use crate::shared::{OrderBookId, OrderUpdateType, Side};
    use chrono::Utc;
    use rust_decimal::Decimal;

    fn order_update(
        market: &str,
        order_hash: &str,
        orderbook_id: &str,
        remaining: Decimal,
    ) -> wire::OrderUpdate {
        wire::OrderUpdate {
            market_pubkey: PubkeyStr::from(market),
            orderbook_id: OrderBookId::from(orderbook_id),
            timestamp: Utc::now(),
            tx_signature: None,
            update_type: OrderUpdateType::Update,
            order: wire::WsOrder {
                order_hash: order_hash.to_string(),
                price: Decimal::new(50, 1),
                is_maker: true,
                remaining,
                filled: Decimal::ZERO,
                fill_amount: Decimal::ZERO,
                side: Side::Bid,
                created_at: Utc::now(),
                base_mint: PubkeyStr::from("base"),
                quote_mint: PubkeyStr::from("quote"),
                outcome_index: 0,
                status: OrderStatus::Open,
                balance: Some(wire::UserOrderUpdateBalance { outcomes: vec![] }),
            },
        }
    }

    #[test]
    fn test_upsert_adds_order() {
        let mut container = UserOpenLimitOrders::new();
        let update = order_update("mkt1", "hash1", "ob1", Decimal::new(10, 0));
        container.upsert(&update);
        assert!(!container.is_empty());
        let orders = container
            .get(&PubkeyStr::from("mkt1"), &OrderBookId::from("ob1"))
            .unwrap();
        assert_eq!(orders.len(), 1);
        assert_eq!(orders[0].order_hash, "hash1");
    }

    #[test]
    fn test_upsert_replaces_same_hash() {
        let mut container = UserOpenLimitOrders::new();
        container.upsert(&order_update("mkt1", "hash1", "ob1", Decimal::new(10, 0)));
        container.upsert(&order_update("mkt1", "hash1", "ob1", Decimal::new(5, 0)));
        let orders = container
            .get(&PubkeyStr::from("mkt1"), &OrderBookId::from("ob1"))
            .unwrap();
        assert_eq!(orders.len(), 1);
        assert_eq!(orders[0].remaining_size, Decimal::new(5, 0));
    }

    #[test]
    fn test_remove_by_hash() {
        let mut container = UserOpenLimitOrders::new();
        container.upsert(&order_update("mkt1", "hash1", "ob1", Decimal::new(10, 0)));
        container.upsert(&order_update("mkt1", "hash2", "ob1", Decimal::new(5, 0)));
        container.remove("hash1");
        let orders = container
            .get(&PubkeyStr::from("mkt1"), &OrderBookId::from("ob1"))
            .unwrap();
        assert_eq!(orders.len(), 1);
        assert_eq!(orders[0].order_hash, "hash2");
    }

    #[test]
    fn test_get_by_market() {
        let mut container = UserOpenLimitOrders::new();
        container.upsert(&order_update("mkt1", "hash1", "ob1", Decimal::new(10, 0)));
        container.upsert(&order_update("mkt1", "hash2", "ob2", Decimal::new(5, 0)));
        container.upsert(&order_update("mkt2", "hash3", "ob3", Decimal::new(1, 0)));
        let by_orderbook = container.get_by_market(&PubkeyStr::from("mkt1")).unwrap();
        assert_eq!(by_orderbook.len(), 2);
        assert_eq!(
            by_orderbook.get(&OrderBookId::from("ob1")).unwrap().len(),
            1
        );
        assert_eq!(
            by_orderbook.get(&OrderBookId::from("ob2")).unwrap().len(),
            1
        );
        assert!(container
            .get_by_market(&PubkeyStr::from("mkt_nonexistent"))
            .is_none());
    }

    #[test]
    fn test_clear() {
        let mut container = UserOpenLimitOrders::new();
        container.upsert(&order_update("mkt1", "hash1", "ob1", Decimal::new(10, 0)));
        container.clear();
        assert!(container.is_empty());
        assert!(container
            .get(&PubkeyStr::from("mkt1"), &OrderBookId::from("ob1"))
            .is_none());
    }
}
