//! User state management.
//!
//! Maintains local state for user orders and balances.

use std::collections::HashMap;
use std::str::FromStr;

use rust_decimal::Decimal;

use crate::websocket::types::{
    BalanceEntry, UserOrderSnapshot,
    UserEventData, UserSnapshotData, UserOrderEvent, UserBalanceEvent, UserNonceEvent,
};

/// Check if a string represents zero using precise decimal comparison.
fn is_zero(s: &str) -> bool {
    Decimal::from_str(s)
        .map(|v| v.is_zero())
        .unwrap_or(false)
}

/// User state tracking orders and balances
#[derive(Debug, Clone, Default)]
pub struct UserState {
    /// User public key / wallet address
    pub user: String,
    /// Open orders by order hash
    pub orders: HashMap<String, UserOrderSnapshot>,
    /// Balances keyed by orderbook_id
    pub balances: HashMap<String, BalanceEntry>,
    /// Latest nonce from the server
    pub nonce: u64,
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
            nonce: 0,
            has_snapshot: false,
            last_timestamp: None,
        }
    }

    /// Apply any user event (dispatches by variant)
    pub fn apply_event(&mut self, data: &UserEventData) {
        match data {
            UserEventData::Snapshot(snapshot) => self.apply_snapshot(snapshot),
            UserEventData::Order(order_event) => self.apply_order_event(order_event),
            UserEventData::BalanceUpdate(balance_event) => self.apply_balance_update(balance_event),
            UserEventData::Nonce(nonce_event) => self.apply_nonce(nonce_event),
        }
    }

    /// Apply a snapshot (full user state)
    pub fn apply_snapshot(&mut self, data: &UserSnapshotData) {
        self.orders.clear();
        self.balances.clear();

        for order in &data.orders {
            self.orders.insert(order.order_hash.clone(), order.clone());
        }

        for (key, balance) in &data.balances {
            self.balances.insert(key.clone(), balance.clone());
        }

        self.nonce = data.nonce;
        self.has_snapshot = true;
    }

    /// Apply a real-time order event (placement, update, cancellation)
    pub fn apply_order_event(&mut self, event: &UserOrderEvent) {
        let fill = &event.order;
        let order_hash = &fill.order_hash;

        // If remaining is 0, the order is fully filled or cancelled - remove it
        if is_zero(&fill.remaining) {
            self.orders.remove(order_hash);
        } else if let Some(existing) = self.orders.get_mut(order_hash) {
            // Update existing order
            existing.remaining = fill.remaining.clone();
            existing.filled = fill.filled.clone();
            if !fill.status.is_empty() {
                existing.status = fill.status.clone();
            }
        } else {
            // New order (PLACEMENT) - construct from fill info
            let order = UserOrderSnapshot {
                order_hash: order_hash.clone(),
                market_pubkey: event.market_pubkey.clone(),
                orderbook_id: event.orderbook_id.clone(),
                side: fill.side.clone(),
                amount_in: fill.remaining.clone(),
                amount_out: "0".to_string(),
                remaining: fill.remaining.clone(),
                filled: fill.filled.clone(),
                price: fill.price.clone(),
                created_at: fill.created_at,
                expiration: 0,
                base_mint: fill.base_mint.clone(),
                quote_mint: fill.quote_mint.clone(),
                outcome_index: fill.outcome_index,
                status: if fill.status.is_empty() { "OPEN".to_string() } else { fill.status.clone() },
            };
            self.orders.insert(order_hash.clone(), order);
        }

        // Apply balance from fill if present
        if let Some(balance) = &fill.balance {
            let entry = BalanceEntry {
                market_pubkey: event.market_pubkey.clone(),
                orderbook_id: event.orderbook_id.clone(),
                outcomes: balance.outcomes.clone(),
            };
            self.balances.insert(event.orderbook_id.clone(), entry);
        }

        self.last_timestamp = Some(event.timestamp.clone());
    }

    /// Apply a real-time balance update
    pub fn apply_balance_update(&mut self, event: &UserBalanceEvent) {
        let entry = BalanceEntry {
            market_pubkey: event.market_pubkey.clone(),
            orderbook_id: event.orderbook_id.clone(),
            outcomes: event.balance.outcomes.clone(),
        };
        self.balances.insert(event.orderbook_id.clone(), entry);
        self.last_timestamp = Some(event.timestamp.clone());
    }

    /// Apply a nonce update
    fn apply_nonce(&mut self, event: &UserNonceEvent) {
        self.nonce = event.new_nonce;
        self.last_timestamp = Some(event.timestamp.clone());
    }

    /// Get an order by hash
    pub fn get_order(&self, order_hash: &str) -> Option<&UserOrderSnapshot> {
        self.orders.get(order_hash)
    }

    /// Get all open orders
    pub fn open_orders(&self) -> Vec<&UserOrderSnapshot> {
        self.orders.values().collect()
    }

    /// Get orders for a specific market
    pub fn orders_for_market(&self, market_pubkey: &str) -> Vec<&UserOrderSnapshot> {
        self.orders
            .values()
            .filter(|o| o.market_pubkey == market_pubkey)
            .collect()
    }

    /// Get orders for a specific orderbook
    pub fn orders_for_orderbook(&self, orderbook_id: &str) -> Vec<&UserOrderSnapshot> {
        self.orders
            .values()
            .filter(|o| o.orderbook_id == orderbook_id)
            .collect()
    }

    /// Get balance for an orderbook
    pub fn get_balance(&self, orderbook_id: &str) -> Option<&BalanceEntry> {
        self.balances.get(orderbook_id)
    }

    /// Get all balances
    pub fn all_balances(&self) -> Vec<&BalanceEntry> {
        self.balances.values().collect()
    }

    /// Get total idle balance for a specific outcome
    pub fn idle_balance_for_outcome(
        &self,
        orderbook_id: &str,
        outcome_index: i32,
    ) -> Option<String> {
        self.get_balance(orderbook_id)
            .and_then(|b| b.outcomes.iter().find(|o| o.outcome_index == outcome_index))
            .map(|o| o.idle.clone())
    }

    /// Get total on-book balance for a specific outcome
    pub fn on_book_balance_for_outcome(
        &self,
        orderbook_id: &str,
        outcome_index: i32,
    ) -> Option<String> {
        self.get_balance(orderbook_id)
            .and_then(|b| b.outcomes.iter().find(|o| o.outcome_index == outcome_index))
            .map(|o| o.on_book.clone())
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
        self.nonce = 0;
        self.has_snapshot = false;
        self.last_timestamp = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::websocket::types::{OutcomeBalance, Balance, UserFillInfo};

    fn create_snapshot() -> UserEventData {
        UserEventData::Snapshot(UserSnapshotData {
            orders: vec![UserOrderSnapshot {
                order_hash: "hash1".to_string(),
                market_pubkey: "market1".to_string(),
                orderbook_id: "ob1".to_string(),
                side: "bid".to_string(),
                amount_in: "0.001000".to_string(),
                amount_out: "0.000500".to_string(),
                remaining: "0.000800".to_string(),
                filled: "0.000200".to_string(),
                price: "0.500000".to_string(),
                created_at: 1704067200000,
                expiration: 0,
                base_mint: "base_mint1".to_string(),
                quote_mint: "quote_mint1".to_string(),
                outcome_index: 0,
                status: "OPEN".to_string(),
            }],
            balances: {
                let mut map = HashMap::new();
                map.insert(
                    "ob1".to_string(),
                    BalanceEntry {
                        market_pubkey: "market1".to_string(),
                        orderbook_id: "ob1".to_string(),
                        outcomes: vec![OutcomeBalance {
                            outcome_index: 0,
                            mint: "outcome_mint".to_string(),
                            idle: "0.005000".to_string(),
                            on_book: "0.001000".to_string(),
                        }],
                    },
                );
                map
            },
            nonce: 42,
        })
    }

    #[test]
    fn test_apply_snapshot() {
        let mut state = UserState::new("user1".to_string());
        state.apply_event(&create_snapshot());

        assert!(state.has_snapshot());
        assert_eq!(state.order_count(), 1);
        assert!(state.get_order("hash1").is_some());
        assert!(state.get_balance("ob1").is_some());
        assert_eq!(state.nonce, 42);
    }

    #[test]
    fn test_order_update() {
        let mut state = UserState::new("user1".to_string());
        state.apply_event(&create_snapshot());

        let update = UserEventData::Order(UserOrderEvent {
            update_type: "UPDATE".to_string(),
            market_pubkey: "market1".to_string(),
            orderbook_id: "ob1".to_string(),
            timestamp: "2024-01-01T00:00:01.000Z".to_string(),
            order: UserFillInfo {
                order_hash: "hash1".to_string(),
                price: "0.500000".to_string(),
                fill_amount: "0.000100".to_string(),
                remaining: "0.000700".to_string(),
                filled: "0.000300".to_string(),
                side: "bid".to_string(),
                is_maker: true,
                created_at: 1704067200000,
                balance: None,
                base_mint: "base_mint1".to_string(),
                quote_mint: "quote_mint1".to_string(),
                outcome_index: 0,
                status: "OPEN".to_string(),
            },
        });

        state.apply_event(&update);

        let order = state.get_order("hash1").unwrap();
        assert_eq!(order.remaining, "0.000700");
        assert_eq!(order.filled, "0.000300");
    }

    #[test]
    fn test_order_removal_on_full_fill() {
        let mut state = UserState::new("user1".to_string());
        state.apply_event(&create_snapshot());

        let update = UserEventData::Order(UserOrderEvent {
            update_type: "UPDATE".to_string(),
            market_pubkey: "market1".to_string(),
            orderbook_id: "ob1".to_string(),
            timestamp: "2024-01-01T00:00:01.000Z".to_string(),
            order: UserFillInfo {
                order_hash: "hash1".to_string(),
                price: "0.500000".to_string(),
                fill_amount: "0.000800".to_string(),
                remaining: "0".to_string(), // Fully filled
                filled: "0.001000".to_string(),
                side: "bid".to_string(),
                is_maker: true,
                created_at: 1704067200000,
                balance: None,
                base_mint: "base_mint1".to_string(),
                quote_mint: "quote_mint1".to_string(),
                outcome_index: 0,
                status: "FILLED".to_string(),
            },
        });

        state.apply_event(&update);

        assert!(state.get_order("hash1").is_none());
        assert_eq!(state.order_count(), 0);
    }

    #[test]
    fn test_new_order_placement() {
        let mut state = UserState::new("user1".to_string());
        state.apply_event(&create_snapshot());

        let placement = UserEventData::Order(UserOrderEvent {
            update_type: "PLACEMENT".to_string(),
            market_pubkey: "market1".to_string(),
            orderbook_id: "ob1".to_string(),
            timestamp: "2024-01-01T00:00:01.000Z".to_string(),
            order: UserFillInfo {
                order_hash: "hash2".to_string(),
                price: "0.600000".to_string(),
                fill_amount: "0".to_string(),
                remaining: "1.000000".to_string(),
                filled: "0".to_string(),
                side: "ask".to_string(),
                is_maker: true,
                created_at: 1704067200000,
                balance: None,
                base_mint: "base_mint1".to_string(),
                quote_mint: "quote_mint1".to_string(),
                outcome_index: 0,
                status: "OPEN".to_string(),
            },
        });

        state.apply_event(&placement);

        assert_eq!(state.order_count(), 2);
        let order = state.get_order("hash2").unwrap();
        assert_eq!(order.price, "0.600000");
        assert_eq!(order.side, "ask");
        assert_eq!(order.base_mint, "base_mint1");
    }

    #[test]
    fn test_balance_update() {
        let mut state = UserState::new("user1".to_string());
        state.apply_event(&create_snapshot());

        let update = UserEventData::BalanceUpdate(UserBalanceEvent {
            market_pubkey: "market1".to_string(),
            orderbook_id: "ob1".to_string(),
            balance: Balance {
                outcomes: vec![OutcomeBalance {
                    outcome_index: 0,
                    mint: "outcome_mint".to_string(),
                    idle: "0.006000".to_string(),
                    on_book: "0.000500".to_string(),
                }],
            },
            timestamp: "2024-01-01T00:00:01.000Z".to_string(),
        });

        state.apply_event(&update);

        let balance = state.get_balance("ob1").unwrap();
        assert_eq!(balance.outcomes[0].idle, "0.006000");
        assert_eq!(balance.outcomes[0].on_book, "0.000500");
    }

    #[test]
    fn test_nonce_update() {
        let mut state = UserState::new("user1".to_string());
        state.apply_event(&create_snapshot());
        assert_eq!(state.nonce, 42);

        let nonce_event = UserEventData::Nonce(UserNonceEvent {
            user_pubkey: "user1".to_string(),
            new_nonce: 99,
            timestamp: "2024-01-01T00:00:01.000Z".to_string(),
        });

        state.apply_event(&nonce_event);
        assert_eq!(state.nonce, 99);
    }

    #[test]
    fn test_balance_lookup_by_orderbook() {
        let mut state = UserState::new("user1".to_string());
        state.apply_event(&create_snapshot());

        assert_eq!(
            state.idle_balance_for_outcome("ob1", 0),
            Some("0.005000".to_string())
        );
        assert_eq!(
            state.on_book_balance_for_outcome("ob1", 0),
            Some("0.001000".to_string())
        );
        assert_eq!(state.idle_balance_for_outcome("ob_nonexistent", 0), None);
    }
}
