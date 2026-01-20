//! User state management.
//!
//! Maintains local state for user orders and balances.

use std::collections::HashMap;

use crate::websocket::types::{Balance, BalanceEntry, Order, UserEventData};

/// User state tracking orders and balances
#[derive(Debug, Clone, Default)]
pub struct UserState {
    /// User public key
    pub user: String,
    /// Open orders by order hash
    pub orders: HashMap<String, Order>,
    /// Balances by market_pubkey:deposit_mint key
    pub balances: HashMap<String, BalanceEntry>,
    /// Whether initial snapshot has been received
    has_snapshot: bool,
    /// Last update timestamp
    last_timestamp: Option<String>,
}

impl UserState {
    /// Create a new empty user state
    pub fn new(user: String) -> Self {
        Self {
            user,
            orders: HashMap::new(),
            balances: HashMap::new(),
            has_snapshot: false,
            last_timestamp: None,
        }
    }

    /// Apply a snapshot (full user state)
    pub fn apply_snapshot(&mut self, data: &UserEventData) {
        // Clear existing state
        self.orders.clear();
        self.balances.clear();

        // Apply orders
        for order in &data.orders {
            self.orders.insert(order.order_hash.clone(), order.clone());
        }

        // Apply balances
        for (key, balance) in &data.balances {
            self.balances.insert(key.clone(), balance.clone());
        }

        self.has_snapshot = true;
        self.last_timestamp = data.timestamp.clone();
    }

    /// Apply an order update
    pub fn apply_order_update(&mut self, data: &UserEventData) {
        if let Some(update) = &data.order {
            let order_hash = &update.order_hash;

            // If remaining is 0, the order is fully filled or cancelled - remove it
            if update.remaining == 0 {
                self.orders.remove(order_hash);
            } else if let Some(existing) = self.orders.get_mut(order_hash) {
                // Update existing order
                existing.remaining = update.remaining;
                existing.filled = update.filled;
            } else {
                // New order - we need to construct it from the update
                // This shouldn't normally happen as orders should come from snapshot first
                // but we handle it for robustness
                if let (Some(market_pubkey), Some(orderbook_id)) =
                    (&data.market_pubkey, &data.orderbook_id)
                {
                    let order = Order {
                        order_hash: order_hash.clone(),
                        market_pubkey: market_pubkey.clone(),
                        orderbook_id: orderbook_id.clone(),
                        side: update.side,
                        maker_amount: update.remaining + update.filled, // Approximate
                        taker_amount: 0,                                // Unknown
                        remaining: update.remaining,
                        filled: update.filled,
                        price: update.price,
                        created_at: update.created_at,
                        expiration: 0,
                    };
                    self.orders.insert(order_hash.clone(), order);
                }
            }

            // Apply balance updates if present
            if let Some(balance) = &update.balance {
                self.apply_balance_from_order(data, balance);
            }
        }

        self.last_timestamp = data.timestamp.clone();
    }

    /// Apply a balance update
    pub fn apply_balance_update(&mut self, data: &UserEventData) {
        if let (Some(market_pubkey), Some(deposit_mint), Some(balance)) =
            (&data.market_pubkey, &data.deposit_mint, &data.balance)
        {
            let key = format!("{}:{}", market_pubkey, deposit_mint);
            let entry = BalanceEntry {
                market_pubkey: market_pubkey.clone(),
                deposit_mint: deposit_mint.clone(),
                outcomes: balance.outcomes.clone(),
            };
            self.balances.insert(key, entry);
        }

        self.last_timestamp = data.timestamp.clone();
    }

    /// Apply balance from order update
    fn apply_balance_from_order(&mut self, data: &UserEventData, balance: &Balance) {
        if let (Some(market_pubkey), Some(deposit_mint)) =
            (&data.market_pubkey, &data.deposit_mint)
        {
            let key = format!("{}:{}", market_pubkey, deposit_mint);
            let entry = BalanceEntry {
                market_pubkey: market_pubkey.clone(),
                deposit_mint: deposit_mint.clone(),
                outcomes: balance.outcomes.clone(),
            };
            self.balances.insert(key, entry);
        } else if let Some(market_pubkey) = &data.market_pubkey {
            // If no deposit_mint, update existing entry with matching market
            for (key, entry) in self.balances.iter_mut() {
                if key.starts_with(market_pubkey) {
                    entry.outcomes = balance.outcomes.clone();
                    break;
                }
            }
        }
    }

    /// Apply any user event
    pub fn apply_event(&mut self, data: &UserEventData) {
        match data.event_type.as_str() {
            "snapshot" => self.apply_snapshot(data),
            "order_update" => self.apply_order_update(data),
            "balance_update" => self.apply_balance_update(data),
            _ => {
                tracing::warn!("Unknown user event type: {}", data.event_type);
            }
        }
    }

    /// Get an order by hash
    pub fn get_order(&self, order_hash: &str) -> Option<&Order> {
        self.orders.get(order_hash)
    }

    /// Get all open orders
    pub fn open_orders(&self) -> Vec<&Order> {
        self.orders.values().collect()
    }

    /// Get orders for a specific market
    pub fn orders_for_market(&self, market_pubkey: &str) -> Vec<&Order> {
        self.orders
            .values()
            .filter(|o| o.market_pubkey == market_pubkey)
            .collect()
    }

    /// Get orders for a specific orderbook
    pub fn orders_for_orderbook(&self, orderbook_id: &str) -> Vec<&Order> {
        self.orders
            .values()
            .filter(|o| o.orderbook_id == orderbook_id)
            .collect()
    }

    /// Get balance for a market/deposit_mint pair
    pub fn get_balance(&self, market_pubkey: &str, deposit_mint: &str) -> Option<&BalanceEntry> {
        let key = format!("{}:{}", market_pubkey, deposit_mint);
        self.balances.get(&key)
    }

    /// Get all balances
    pub fn all_balances(&self) -> Vec<&BalanceEntry> {
        self.balances.values().collect()
    }

    /// Get total idle balance for a specific outcome
    pub fn idle_balance_for_outcome(
        &self,
        market_pubkey: &str,
        deposit_mint: &str,
        outcome_index: i32,
    ) -> Option<i64> {
        self.get_balance(market_pubkey, deposit_mint)
            .and_then(|b| b.outcomes.iter().find(|o| o.outcome_index == outcome_index))
            .map(|o| o.idle)
    }

    /// Get total on-book balance for a specific outcome
    pub fn on_book_balance_for_outcome(
        &self,
        market_pubkey: &str,
        deposit_mint: &str,
        outcome_index: i32,
    ) -> Option<i64> {
        self.get_balance(market_pubkey, deposit_mint)
            .and_then(|b| b.outcomes.iter().find(|o| o.outcome_index == outcome_index))
            .map(|o| o.on_book)
    }

    /// Number of open orders
    pub fn order_count(&self) -> usize {
        self.orders.len()
    }

    /// Whether the user state has received its initial snapshot
    pub fn has_snapshot(&self) -> bool {
        self.has_snapshot
    }

    /// Last update timestamp
    pub fn last_timestamp(&self) -> Option<&str> {
        self.last_timestamp.as_deref()
    }

    /// Clear the user state (for disconnect/resync)
    pub fn clear(&mut self) {
        self.orders.clear();
        self.balances.clear();
        self.has_snapshot = false;
        self.last_timestamp = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_snapshot() -> UserEventData {
        UserEventData {
            event_type: "snapshot".to_string(),
            orders: vec![Order {
                order_hash: "hash1".to_string(),
                market_pubkey: "market1".to_string(),
                orderbook_id: "ob1".to_string(),
                side: 0,
                maker_amount: 1000,
                taker_amount: 500,
                remaining: 800,
                filled: 200,
                price: 500000,
                created_at: 1704067200000,
                expiration: 0,
            }],
            balances: {
                let mut map = HashMap::new();
                map.insert(
                    "market1:mint1".to_string(),
                    BalanceEntry {
                        market_pubkey: "market1".to_string(),
                        deposit_mint: "mint1".to_string(),
                        outcomes: vec![OutcomeBalance {
                            outcome_index: 0,
                            mint: "outcome_mint".to_string(),
                            idle: 5000,
                            on_book: 1000,
                        }],
                    },
                );
                map
            },
            order: None,
            balance: None,
            market_pubkey: None,
            orderbook_id: None,
            deposit_mint: None,
            timestamp: Some("2024-01-01T00:00:00.000Z".to_string()),
        }
    }

    #[test]
    fn test_apply_snapshot() {
        let mut state = UserState::new("user1".to_string());
        let snapshot = create_snapshot();

        state.apply_snapshot(&snapshot);

        assert!(state.has_snapshot());
        assert_eq!(state.order_count(), 1);
        assert!(state.get_order("hash1").is_some());
        assert!(state.get_balance("market1", "mint1").is_some());
    }

    #[test]
    fn test_order_update() {
        let mut state = UserState::new("user1".to_string());
        state.apply_snapshot(&create_snapshot());

        let update = UserEventData {
            event_type: "order_update".to_string(),
            orders: vec![],
            balances: HashMap::new(),
            order: Some(OrderUpdate {
                order_hash: "hash1".to_string(),
                price: 500000,
                fill_amount: 100,
                remaining: 700,
                filled: 300,
                side: 0,
                is_maker: true,
                created_at: 1704067200000,
                balance: None,
            }),
            balance: None,
            market_pubkey: Some("market1".to_string()),
            orderbook_id: Some("ob1".to_string()),
            deposit_mint: None,
            timestamp: Some("2024-01-01T00:00:01.000Z".to_string()),
        };

        state.apply_order_update(&update);

        let order = state.get_order("hash1").unwrap();
        assert_eq!(order.remaining, 700);
        assert_eq!(order.filled, 300);
    }

    #[test]
    fn test_order_removal_on_full_fill() {
        let mut state = UserState::new("user1".to_string());
        state.apply_snapshot(&create_snapshot());

        let update = UserEventData {
            event_type: "order_update".to_string(),
            orders: vec![],
            balances: HashMap::new(),
            order: Some(OrderUpdate {
                order_hash: "hash1".to_string(),
                price: 500000,
                fill_amount: 800,
                remaining: 0, // Fully filled
                filled: 1000,
                side: 0,
                is_maker: true,
                created_at: 1704067200000,
                balance: None,
            }),
            balance: None,
            market_pubkey: Some("market1".to_string()),
            orderbook_id: Some("ob1".to_string()),
            deposit_mint: None,
            timestamp: Some("2024-01-01T00:00:01.000Z".to_string()),
        };

        state.apply_order_update(&update);

        assert!(state.get_order("hash1").is_none());
        assert_eq!(state.order_count(), 0);
    }

    #[test]
    fn test_balance_update() {
        let mut state = UserState::new("user1".to_string());
        state.apply_snapshot(&create_snapshot());

        let update = UserEventData {
            event_type: "balance_update".to_string(),
            orders: vec![],
            balances: HashMap::new(),
            order: None,
            balance: Some(Balance {
                outcomes: vec![OutcomeBalance {
                    outcome_index: 0,
                    mint: "outcome_mint".to_string(),
                    idle: 6000,
                    on_book: 500,
                }],
            }),
            market_pubkey: Some("market1".to_string()),
            orderbook_id: None,
            deposit_mint: Some("mint1".to_string()),
            timestamp: Some("2024-01-01T00:00:01.000Z".to_string()),
        };

        state.apply_balance_update(&update);

        let balance = state.get_balance("market1", "mint1").unwrap();
        assert_eq!(balance.outcomes[0].idle, 6000);
        assert_eq!(balance.outcomes[0].on_book, 500);
    }
}
