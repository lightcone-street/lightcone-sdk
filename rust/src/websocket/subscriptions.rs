//! Subscription management for WebSocket channels.
//!
//! Tracks active subscriptions and supports re-subscribing after reconnect.

use std::collections::{HashMap, HashSet};

use crate::websocket::types::SubscribeParams;

/// Represents a subscription to a specific channel
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Subscription {
    /// Book update subscription for orderbook IDs
    BookUpdate { orderbook_ids: Vec<String> },
    /// Trades subscription for orderbook IDs
    Trades { orderbook_ids: Vec<String> },
    /// User subscription for a wallet address
    User { wallet_address: String },
    /// Price history subscription
    PriceHistory {
        orderbook_id: String,
        resolution: String,
        include_ohlcv: bool,
    },
    /// Market events subscription
    Market { market_pubkey: String },
    /// Ticker (best bid/ask) subscription for orderbook IDs
    Ticker { orderbook_ids: Vec<String> },
}

impl Subscription {
    /// Convert to SubscribeParams for sending
    pub fn to_params(&self) -> SubscribeParams {
        match self {
            Self::BookUpdate { orderbook_ids } => SubscribeParams::book_update(orderbook_ids.clone()),
            Self::Trades { orderbook_ids } => SubscribeParams::trades(orderbook_ids.clone()),
            Self::User { wallet_address } => SubscribeParams::user(wallet_address.clone()),
            Self::PriceHistory {
                orderbook_id,
                resolution,
                include_ohlcv,
            } => SubscribeParams::price_history(
                orderbook_id.clone(),
                resolution.clone(),
                *include_ohlcv,
            ),
            Self::Market { market_pubkey } => SubscribeParams::market(market_pubkey.clone()),
            Self::Ticker { orderbook_ids } => SubscribeParams::ticker(orderbook_ids.clone()),
        }
    }

    /// Get the subscription type as a string
    pub fn subscription_type(&self) -> &'static str {
        match self {
            Self::BookUpdate { .. } => "book_update",
            Self::Trades { .. } => "trades",
            Self::User { .. } => "user",
            Self::PriceHistory { .. } => "price_history",
            Self::Market { .. } => "market",
            Self::Ticker { .. } => "ticker",
        }
    }
}

/// Manages active subscriptions
#[derive(Debug, Default)]
pub struct SubscriptionManager {
    /// Book update subscriptions (orderbook_id -> subscription)
    book_updates: HashSet<String>,
    /// Trades subscriptions (orderbook_id -> subscription)
    trades: HashSet<String>,
    /// User subscriptions (wallet address -> subscription)
    users: HashSet<String>,
    /// Price history subscriptions (orderbook_id:resolution -> params)
    price_history: HashMap<String, (String, String, bool)>, // key -> (orderbook_id, resolution, include_ohlcv)
    /// Market subscriptions (market_pubkey -> subscription)
    markets: HashSet<String>,
    /// Ticker subscriptions (orderbook_id -> subscription)
    ticker: HashSet<String>,
}

impl SubscriptionManager {
    /// Create a new subscription manager
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a book update subscription
    pub fn add_book_update(&mut self, orderbook_ids: Vec<String>) {
        for id in orderbook_ids {
            self.book_updates.insert(id);
        }
    }

    /// Remove a book update subscription
    pub fn remove_book_update(&mut self, orderbook_ids: &[String]) {
        for id in orderbook_ids {
            self.book_updates.remove(id);
        }
    }

    /// Check if subscribed to book updates for an orderbook
    pub fn is_subscribed_book_update(&self, orderbook_id: &str) -> bool {
        self.book_updates.contains(orderbook_id)
    }

    /// Add a trades subscription
    pub fn add_trades(&mut self, orderbook_ids: Vec<String>) {
        for id in orderbook_ids {
            self.trades.insert(id);
        }
    }

    /// Remove a trades subscription
    pub fn remove_trades(&mut self, orderbook_ids: &[String]) {
        for id in orderbook_ids {
            self.trades.remove(id);
        }
    }

    /// Check if subscribed to trades for an orderbook
    pub fn is_subscribed_trades(&self, orderbook_id: &str) -> bool {
        self.trades.contains(orderbook_id)
    }

    /// Add a user subscription
    pub fn add_user(&mut self, wallet_address: String) {
        self.users.insert(wallet_address);
    }

    /// Remove a user subscription
    pub fn remove_user(&mut self, wallet_address: &str) {
        self.users.remove(wallet_address);
    }

    /// Check if subscribed to a user
    pub fn is_subscribed_user(&self, wallet_address: &str) -> bool {
        self.users.contains(wallet_address)
    }

    /// Add a price history subscription
    pub fn add_price_history(&mut self, orderbook_id: String, resolution: String, include_ohlcv: bool) {
        let key = format!("{}:{}", orderbook_id, resolution);
        self.price_history
            .insert(key, (orderbook_id, resolution, include_ohlcv));
    }

    /// Remove a price history subscription
    pub fn remove_price_history(&mut self, orderbook_id: &str, resolution: &str) {
        let key = format!("{}:{}", orderbook_id, resolution);
        self.price_history.remove(&key);
    }

    /// Check if subscribed to price history for an orderbook/resolution
    pub fn is_subscribed_price_history(&self, orderbook_id: &str, resolution: &str) -> bool {
        let key = format!("{}:{}", orderbook_id, resolution);
        self.price_history.contains_key(&key)
    }

    /// Add a market subscription
    pub fn add_market(&mut self, market_pubkey: String) {
        self.markets.insert(market_pubkey);
    }

    /// Remove a market subscription
    pub fn remove_market(&mut self, market_pubkey: &str) {
        self.markets.remove(market_pubkey);
    }

    /// Check if subscribed to market events
    pub fn is_subscribed_market(&self, market_pubkey: &str) -> bool {
        self.markets.contains(market_pubkey) || self.markets.contains("all")
    }

    /// Add a ticker subscription
    pub fn add_ticker(&mut self, orderbook_ids: Vec<String>) {
        for id in orderbook_ids {
            self.ticker.insert(id);
        }
    }

    /// Remove a ticker subscription
    pub fn remove_ticker(&mut self, orderbook_ids: &[String]) {
        for id in orderbook_ids {
            self.ticker.remove(id);
        }
    }

    /// Check if subscribed to ticker for an orderbook
    pub fn is_subscribed_ticker(&self, orderbook_id: &str) -> bool {
        self.ticker.contains(orderbook_id)
    }

    /// Get all subscribed orderbook IDs (for ticker)
    pub fn ticker_orderbooks(&self) -> Vec<String> {
        self.ticker.iter().cloned().collect()
    }

    /// Get all subscriptions for re-subscribing after reconnect
    pub fn get_all_subscriptions(&self) -> Vec<Subscription> {
        let mut subs = Vec::new();

        // Group book updates
        if !self.book_updates.is_empty() {
            subs.push(Subscription::BookUpdate {
                orderbook_ids: self.book_updates.iter().cloned().collect(),
            });
        }

        // Group trades
        if !self.trades.is_empty() {
            subs.push(Subscription::Trades {
                orderbook_ids: self.trades.iter().cloned().collect(),
            });
        }

        // Users
        for wallet_address in &self.users {
            subs.push(Subscription::User { wallet_address: wallet_address.clone() });
        }

        // Price history
        for (orderbook_id, resolution, include_ohlcv) in self.price_history.values() {
            subs.push(Subscription::PriceHistory {
                orderbook_id: orderbook_id.clone(),
                resolution: resolution.clone(),
                include_ohlcv: *include_ohlcv,
            });
        }

        // Markets
        for market_pubkey in &self.markets {
            subs.push(Subscription::Market {
                market_pubkey: market_pubkey.clone(),
            });
        }

        // Ticker
        if !self.ticker.is_empty() {
            subs.push(Subscription::Ticker {
                orderbook_ids: self.ticker.iter().cloned().collect(),
            });
        }

        subs
    }

    /// Clear all subscriptions
    pub fn clear(&mut self) {
        self.book_updates.clear();
        self.trades.clear();
        self.users.clear();
        self.price_history.clear();
        self.markets.clear();
        self.ticker.clear();
    }

    /// Check if there are any active subscriptions
    pub fn has_subscriptions(&self) -> bool {
        !self.book_updates.is_empty()
            || !self.trades.is_empty()
            || !self.users.is_empty()
            || !self.price_history.is_empty()
            || !self.markets.is_empty()
            || !self.ticker.is_empty()
    }

    /// Get count of active subscriptions
    pub fn subscription_count(&self) -> usize {
        self.book_updates.len()
            + self.trades.len()
            + self.users.len()
            + self.price_history.len()
            + self.markets.len()
            + self.ticker.len()
    }

    /// Get all subscribed orderbook IDs (for book updates)
    pub fn book_update_orderbooks(&self) -> Vec<String> {
        self.book_updates.iter().cloned().collect()
    }

    /// Get all subscribed orderbook IDs (for trades)
    pub fn trade_orderbooks(&self) -> Vec<String> {
        self.trades.iter().cloned().collect()
    }

    /// Get all subscribed users
    pub fn subscribed_users(&self) -> Vec<String> {
        self.users.iter().cloned().collect()
    }

    /// Get all subscribed markets
    pub fn subscribed_markets(&self) -> Vec<String> {
        self.markets.iter().cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_book_update_subscriptions() {
        let mut manager = SubscriptionManager::new();

        manager.add_book_update(vec!["ob1".to_string(), "ob2".to_string()]);
        assert!(manager.is_subscribed_book_update("ob1"));
        assert!(manager.is_subscribed_book_update("ob2"));
        assert!(!manager.is_subscribed_book_update("ob3"));

        manager.remove_book_update(&["ob1".to_string()]);
        assert!(!manager.is_subscribed_book_update("ob1"));
        assert!(manager.is_subscribed_book_update("ob2"));
    }

    #[test]
    fn test_user_subscriptions() {
        let mut manager = SubscriptionManager::new();

        manager.add_user("wallet1".to_string());
        assert!(manager.is_subscribed_user("wallet1"));
        assert!(!manager.is_subscribed_user("wallet2"));

        manager.remove_user("wallet1");
        assert!(!manager.is_subscribed_user("wallet1"));
    }

    #[test]
    fn test_price_history_subscriptions() {
        let mut manager = SubscriptionManager::new();

        manager.add_price_history("ob1".to_string(), "1m".to_string(), true);
        assert!(manager.is_subscribed_price_history("ob1", "1m"));
        assert!(!manager.is_subscribed_price_history("ob1", "5m"));

        manager.remove_price_history("ob1", "1m");
        assert!(!manager.is_subscribed_price_history("ob1", "1m"));
    }

    #[test]
    fn test_market_subscriptions() {
        let mut manager = SubscriptionManager::new();

        manager.add_market("market1".to_string());
        assert!(manager.is_subscribed_market("market1"));

        // Test "all" markets
        manager.add_market("all".to_string());
        assert!(manager.is_subscribed_market("any_market"));
    }

    #[test]
    fn test_ticker_subscriptions() {
        let mut manager = SubscriptionManager::new();

        manager.add_ticker(vec!["ob1".to_string(), "ob2".to_string()]);
        assert!(manager.is_subscribed_ticker("ob1"));
        assert!(manager.is_subscribed_ticker("ob2"));
        assert!(!manager.is_subscribed_ticker("ob3"));

        manager.remove_ticker(&["ob1".to_string()]);
        assert!(!manager.is_subscribed_ticker("ob1"));
        assert!(manager.is_subscribed_ticker("ob2"));
    }

    #[test]
    fn test_get_all_subscriptions() {
        let mut manager = SubscriptionManager::new();

        manager.add_book_update(vec!["ob1".to_string()]);
        manager.add_user("wallet1".to_string());
        manager.add_price_history("ob1".to_string(), "1m".to_string(), true);
        manager.add_ticker(vec!["ob1".to_string()]);

        let subs = manager.get_all_subscriptions();
        assert_eq!(subs.len(), 4);
    }

    #[test]
    fn test_subscription_count() {
        let mut manager = SubscriptionManager::new();

        assert_eq!(manager.subscription_count(), 0);
        assert!(!manager.has_subscriptions());

        manager.add_book_update(vec!["ob1".to_string(), "ob2".to_string()]);
        manager.add_user("wallet1".to_string());
        manager.add_ticker(vec!["ob1".to_string()]);

        assert_eq!(manager.subscription_count(), 4);
        assert!(manager.has_subscriptions());
    }

    #[test]
    fn test_clear() {
        let mut manager = SubscriptionManager::new();

        manager.add_book_update(vec!["ob1".to_string()]);
        manager.add_user("wallet1".to_string());
        manager.add_ticker(vec!["ob1".to_string()]);

        manager.clear();

        assert!(!manager.has_subscriptions());
        assert_eq!(manager.subscription_count(), 0);
    }

    #[test]
    fn test_subscription_to_params() {
        let sub = Subscription::BookUpdate {
            orderbook_ids: vec!["ob1".to_string()],
        };

        let params = sub.to_params();
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("book_update"));
        assert!(json.contains("ob1"));
    }

    #[test]
    fn test_ticker_subscription_to_params() {
        let sub = Subscription::Ticker {
            orderbook_ids: vec!["ob1".to_string()],
        };

        let params = sub.to_params();
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("ticker"));
        assert!(json.contains("ob1"));
    }

    #[test]
    fn test_user_subscription_to_params() {
        let sub = Subscription::User {
            wallet_address: "wallet123".to_string(),
        };

        let params = sub.to_params();
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("wallet_address"));
        assert!(json.contains("wallet123"));
    }
}
