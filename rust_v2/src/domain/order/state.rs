//! Order state containers — app-owned, SDK-provided update logic.

use super::wire;
use super::{Order, PubkeyStr};
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
