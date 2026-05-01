//! Order state containers — app-owned, SDK-provided update logic.

use crate::shared::{OrderBookId, PubkeyStr};

use super::wire;
use super::{LimitOrder, TriggerOrder};
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

// ─── UserTriggerOrders ──────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub struct UserTriggerOrders {
    pub orders: HashMap<PubkeyStr, HashMap<OrderBookId, Vec<TriggerOrder>>>,
}

impl UserTriggerOrders {
    pub fn new() -> Self {
        Self {
            orders: HashMap::new(),
        }
    }

    pub fn get(
        &self,
        market: &PubkeyStr,
        orderbook_id: &OrderBookId,
    ) -> Option<&Vec<TriggerOrder>> {
        self.orders.get(market)?.get(orderbook_id)
    }

    pub fn get_by_market(
        &self,
        market: &PubkeyStr,
    ) -> Option<&HashMap<OrderBookId, Vec<TriggerOrder>>> {
        self.orders.get(market)
    }

    pub fn get_by_id(&self, trigger_order_id: &str) -> Option<&TriggerOrder> {
        self.orders
            .values()
            .flat_map(|by_orderbook| by_orderbook.values())
            .flat_map(|orders| orders.iter())
            .find(|order| order.trigger_order_id == trigger_order_id)
    }

    pub fn insert(&mut self, order: TriggerOrder) {
        self.orders
            .entry(order.market_pubkey.clone())
            .or_default()
            .entry(order.orderbook_id.clone())
            .or_default()
            .push(order);
    }

    pub fn remove(&mut self, trigger_order_id: &str) -> Option<TriggerOrder> {
        for by_orderbook in self.orders.values_mut() {
            for orders in by_orderbook.values_mut() {
                if let Some(index) = orders
                    .iter()
                    .position(|order| order.trigger_order_id == trigger_order_id)
                {
                    return Some(orders.swap_remove(index));
                }
            }
        }
        None
    }

    pub fn clear(&mut self) {
        self.orders.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.orders
            .values()
            .all(|by_orderbook| by_orderbook.values().all(|orders| orders.is_empty()))
    }

    pub fn len(&self) -> usize {
        self.orders
            .values()
            .flat_map(|by_orderbook| by_orderbook.values())
            .map(|orders| orders.len())
            .sum()
    }

    pub fn all(&self) -> impl Iterator<Item = &TriggerOrder> {
        self.orders
            .values()
            .flat_map(|by_orderbook| by_orderbook.values())
            .flat_map(|orders| orders.iter())
    }
}

impl Default for UserTriggerOrders {
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

    // ── UserTriggerOrders tests ─────────────────────────────────────────────

    fn make_trigger_order(trigger_id: &str, market: &str, orderbook: &str) -> TriggerOrder {
        use crate::shared::{TimeInForce, TriggerType};
        TriggerOrder {
            trigger_order_id: trigger_id.to_string(),
            order_hash: format!("hash_{}", trigger_id),
            market_pubkey: PubkeyStr::from(market),
            orderbook_id: OrderBookId::from(orderbook),
            trigger_price: Decimal::new(55, 2),
            trigger_type: TriggerType::TakeProfit,
            side: Side::Bid,
            amount_in: Decimal::new(1000, 0),
            amount_out: Decimal::new(500, 0),
            time_in_force: TimeInForce::Gtc,
            created_at: chrono::DateTime::from_timestamp_millis(1700000000000).unwrap(),
        }
    }

    #[test]
    fn test_trigger_orders_insert_and_get() {
        let mut container = UserTriggerOrders::new();
        assert!(container.is_empty());
        assert_eq!(container.len(), 0);

        container.insert(make_trigger_order("t1", "mkt1", "ob1"));
        assert!(!container.is_empty());
        assert_eq!(container.len(), 1);

        let orders = container
            .get(&PubkeyStr::from("mkt1"), &OrderBookId::from("ob1"))
            .unwrap();
        assert_eq!(orders[0].trigger_order_id, "t1");
    }

    #[test]
    fn test_trigger_orders_get_by_id() {
        let mut container = UserTriggerOrders::new();
        container.insert(make_trigger_order("t1", "mkt1", "ob1"));
        container.insert(make_trigger_order("t2", "mkt1", "ob2"));

        let order = container.get_by_id("t2").unwrap();
        assert_eq!(order.trigger_order_id, "t2");
        assert!(container.get_by_id("t99").is_none());
    }

    #[test]
    fn test_trigger_orders_groups_by_market_and_orderbook() {
        let mut container = UserTriggerOrders::new();
        container.insert(make_trigger_order("t1", "mkt1", "ob1"));
        container.insert(make_trigger_order("t2", "mkt1", "ob1"));
        container.insert(make_trigger_order("t3", "mkt1", "ob2"));

        assert_eq!(container.len(), 3);
        assert_eq!(
            container
                .get(&PubkeyStr::from("mkt1"), &OrderBookId::from("ob1"))
                .unwrap()
                .len(),
            2
        );
        assert_eq!(
            container
                .get(&PubkeyStr::from("mkt1"), &OrderBookId::from("ob2"))
                .unwrap()
                .len(),
            1
        );
    }

    #[test]
    fn test_trigger_orders_get_by_market() {
        let mut container = UserTriggerOrders::new();
        container.insert(make_trigger_order("t1", "mkt1", "ob1"));
        container.insert(make_trigger_order("t2", "mkt1", "ob2"));
        container.insert(make_trigger_order("t3", "mkt2", "ob3"));
        let by_orderbook = container.get_by_market(&PubkeyStr::from("mkt1")).unwrap();
        assert_eq!(by_orderbook.len(), 2);
        assert!(container
            .get_by_market(&PubkeyStr::from("mkt_nonexistent"))
            .is_none());
    }

    #[test]
    fn test_trigger_orders_remove() {
        let mut container = UserTriggerOrders::new();
        container.insert(make_trigger_order("t1", "mkt1", "ob1"));
        container.insert(make_trigger_order("t2", "mkt1", "ob1"));
        assert_eq!(container.len(), 2);

        let removed = container.remove("t1");
        assert!(removed.is_some());
        assert_eq!(container.len(), 1);
        assert!(container.get_by_id("t1").is_none());
        assert!(container.get_by_id("t2").is_some());
    }

    #[test]
    fn test_trigger_orders_clear() {
        let mut container = UserTriggerOrders::new();
        container.insert(make_trigger_order("t1", "mkt1", "ob1"));
        container.insert(make_trigger_order("t2", "mkt1", "ob2"));
        container.clear();
        assert!(container.is_empty());
    }

    #[test]
    fn test_trigger_orders_all() {
        let mut container = UserTriggerOrders::new();
        container.insert(make_trigger_order("t1", "mkt1", "ob1"));
        container.insert(make_trigger_order("t2", "mkt2", "ob2"));
        let all: Vec<_> = container.all().collect();
        assert_eq!(all.len(), 2);
    }
}
