//! Order state containers — app-owned, SDK-provided update logic.

use crate::shared::PubkeyStr;

use super::wire;
use super::{Order, TriggerOrder};
use std::collections::HashMap;

/// Tracks a user's open orders grouped by market pubkey.
///
/// The app owns instances of this type and calls SDK-provided update methods.
pub struct UserOpenOrders {
    pub orders: HashMap<PubkeyStr, Vec<Order>>,
}

impl UserOpenOrders {
    pub fn new() -> Self {
        Self {
            orders: HashMap::new(),
        }
    }

    pub fn get(&self, market: &PubkeyStr) -> Option<&Vec<Order>> {
        self.orders.get(market)
    }

    /// Insert or update an order from a WS order update.
    pub fn upsert(&mut self, update: &wire::OrderUpdate) {
        let market_orders = self
            .orders
            .entry(update.market_pubkey.clone())
            .or_default();

        market_orders.retain(|o| o.order_hash != update.order.order_hash);
        market_orders.push(update.clone().into());
    }

    /// Remove an order by hash across all markets.
    pub fn remove(&mut self, order_hash: &str) {
        self.orders.values_mut().for_each(|orders| {
            orders.retain(|o| o.order_hash != order_hash);
        });
    }

    /// Clear all orders.
    pub fn clear(&mut self) {
        self.orders.clear();
    }

    /// Check if there are any open orders.
    pub fn is_empty(&self) -> bool {
        self.orders.values().all(|v| v.is_empty())
    }
}

impl Default for UserOpenOrders {
    fn default() -> Self {
        Self::new()
    }
}

// ─── UserTriggerOrders ──────────────────────────────────────────────────────

/// Tracks a user's trigger orders keyed by trigger_order_id.
pub struct UserTriggerOrders {
    pub orders: HashMap<String, TriggerOrder>,
}

impl UserTriggerOrders {
    pub fn new() -> Self {
        Self {
            orders: HashMap::new(),
        }
    }

    pub fn insert(&mut self, order: TriggerOrder) {
        self.orders.insert(order.trigger_order_id.clone(), order);
    }

    pub fn remove(&mut self, trigger_order_id: &str) -> Option<TriggerOrder> {
        self.orders.remove(trigger_order_id)
    }

    pub fn get(&self, trigger_order_id: &str) -> Option<&TriggerOrder> {
        self.orders.get(trigger_order_id)
    }

    pub fn clear(&mut self) {
        self.orders.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.orders.is_empty()
    }

    pub fn len(&self) -> usize {
        self.orders.len()
    }

    pub fn all(&self) -> impl Iterator<Item = &TriggerOrder> {
        self.orders.values()
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
    use crate::shared::{OrderBookId, Side};
    use chrono::Utc;
    use rust_decimal::Decimal;

    fn order_update(market: &str, order_hash: &str, remaining: Decimal) -> wire::OrderUpdate {
        wire::OrderUpdate {
            market_pubkey: PubkeyStr::from(market),
            orderbook_id: OrderBookId::from("ob1"),
            timestamp: Utc::now(),
            tx_signature: None,
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
                balance: Some(wire::UserOrderUpdateBalance { outcomes: vec![] }),
            },
        }
    }

    #[test]
    fn test_upsert_adds_order() {
        let mut uoo = UserOpenOrders::new();
        let update = order_update("mkt1", "hash1", Decimal::new(10, 0));
        uoo.upsert(&update);
        assert!(!uoo.is_empty());
        let orders = uoo.get(&PubkeyStr::from("mkt1")).unwrap();
        assert_eq!(orders.len(), 1);
        assert_eq!(orders[0].order_hash, "hash1");
    }

    #[test]
    fn test_upsert_replaces_same_hash() {
        let mut uoo = UserOpenOrders::new();
        uoo.upsert(&order_update("mkt1", "hash1", Decimal::new(10, 0)));
        uoo.upsert(&order_update("mkt1", "hash1", Decimal::new(5, 0)));
        let orders = uoo.get(&PubkeyStr::from("mkt1")).unwrap();
        assert_eq!(orders.len(), 1);
        assert_eq!(orders[0].remaining_size, Decimal::new(5, 0));
    }

    #[test]
    fn test_remove_by_hash() {
        let mut uoo = UserOpenOrders::new();
        uoo.upsert(&order_update("mkt1", "hash1", Decimal::new(10, 0)));
        uoo.upsert(&order_update("mkt1", "hash2", Decimal::new(5, 0)));
        uoo.remove("hash1");
        let orders = uoo.get(&PubkeyStr::from("mkt1")).unwrap();
        assert_eq!(orders.len(), 1);
        assert_eq!(orders[0].order_hash, "hash2");
    }

    #[test]
    fn test_clear() {
        let mut uoo = UserOpenOrders::new();
        uoo.upsert(&order_update("mkt1", "hash1", Decimal::new(10, 0)));
        uoo.clear();
        assert!(uoo.is_empty());
        assert!(uoo.get(&PubkeyStr::from("mkt1")).is_none());
    }

    // ── UserTriggerOrders tests ─────────────────────────────────────────────

    fn make_trigger_order(id: &str) -> TriggerOrder {
        use crate::shared::TriggerType;
        TriggerOrder {
            trigger_order_id: id.to_string(),
            order_hash: format!("hash_{}", id),
            market_pubkey: "mkt1".to_string(),
            orderbook_id: OrderBookId::from("ob1"),
            trigger_price: "0.55".to_string(),
            trigger_type: TriggerType::TakeProfit,
            side: 0,
            maker_amount: 1000,
            taker_amount: 500,
            tif: 0,
            created_at: 1700000000,
        }
    }

    #[test]
    fn test_trigger_orders_insert_and_get() {
        let mut uto = UserTriggerOrders::new();
        assert!(uto.is_empty());
        assert_eq!(uto.len(), 0);

        uto.insert(make_trigger_order("t1"));
        assert!(!uto.is_empty());
        assert_eq!(uto.len(), 1);

        let order = uto.get("t1").unwrap();
        assert_eq!(order.trigger_order_id, "t1");
    }

    #[test]
    fn test_trigger_orders_remove() {
        let mut uto = UserTriggerOrders::new();
        uto.insert(make_trigger_order("t1"));
        uto.insert(make_trigger_order("t2"));
        assert_eq!(uto.len(), 2);

        let removed = uto.remove("t1");
        assert!(removed.is_some());
        assert_eq!(uto.len(), 1);
        assert!(uto.get("t1").is_none());
        assert!(uto.get("t2").is_some());
    }

    #[test]
    fn test_trigger_orders_clear() {
        let mut uto = UserTriggerOrders::new();
        uto.insert(make_trigger_order("t1"));
        uto.insert(make_trigger_order("t2"));
        uto.clear();
        assert!(uto.is_empty());
    }

    #[test]
    fn test_trigger_orders_all() {
        let mut uto = UserTriggerOrders::new();
        uto.insert(make_trigger_order("t1"));
        uto.insert(make_trigger_order("t2"));
        let all: Vec<_> = uto.all().collect();
        assert_eq!(all.len(), 2);
    }
}
