//! Orderbook state containers — app-owned, SDK-provided update logic.

use crate::domain::orderbook::wire::OrderBook;
use crate::shared::OrderBookId;
use rust_decimal::Decimal;
use std::collections::BTreeMap;

/// Live orderbook state that can apply snapshots and deltas.
///
/// The app owns instances of this type (e.g. inside a Dioxus `Signal`).
/// The SDK provides the update methods.
#[derive(Debug, Clone, Default)]
pub struct OrderbookSnapshot {
    pub orderbook_id: OrderBookId,
    pub seq: u32,
    bids: BTreeMap<Decimal, Decimal>,
    asks: BTreeMap<Decimal, Decimal>,
}

impl OrderbookSnapshot {
    pub fn new(orderbook_id: OrderBookId) -> Self {
        Self {
            orderbook_id,
            seq: 0,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    /// Apply a WS orderbook message (snapshot replaces, delta merges).
    pub fn apply(&mut self, book: &OrderBook) {
        if book.is_snapshot {
            self.bids.clear();
            self.asks.clear();
        }

        self.seq = book.seq;

        for order in &book.bids {
            if order.size.is_zero() {
                self.bids.remove(&order.price);
            } else {
                self.bids.insert(order.price, order.size);
            }
        }

        for order in &book.asks {
            if order.size.is_zero() {
                self.asks.remove(&order.price);
            } else {
                self.asks.insert(order.price, order.size);
            }
        }
    }

    /// Bids sorted by price descending.
    pub fn bids(&self) -> &BTreeMap<Decimal, Decimal> {
        &self.bids
    }

    /// Asks sorted by price ascending.
    pub fn asks(&self) -> &BTreeMap<Decimal, Decimal> {
        &self.asks
    }

    /// Highest bid price.
    pub fn best_bid(&self) -> Option<Decimal> {
        self.bids.keys().next_back().copied()
    }

    /// Lowest ask price.
    pub fn best_ask(&self) -> Option<Decimal> {
        self.asks.keys().next().copied()
    }

    /// Mid price (average of best bid and best ask).
    pub fn mid_price(&self) -> Option<Decimal> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some((bid + ask) / Decimal::from(2)),
            _ => None,
        }
    }

    /// Spread between best ask and best bid.
    pub fn spread(&self) -> Option<Decimal> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Some(ask - bid),
            _ => None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.bids.is_empty() && self.asks.is_empty()
    }

    pub fn clear(&mut self) {
        self.bids.clear();
        self.asks.clear();
        self.seq = 0;
    }
}

impl Default for OrderBookId {
    fn default() -> Self {
        OrderBookId::from("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::orderbook::wire::BookOrder;
    use crate::shared::Side;
    use rust_decimal::Decimal;

    fn order_book(snapshot: bool, seq: u32, bids: Vec<(f64, f64)>, asks: Vec<(f64, f64)>) -> OrderBook {
        OrderBook {
            id: OrderBookId::from("ob_test"),
            is_snapshot: snapshot,
            seq,
            bids: bids
                .into_iter()
                .map(|(price, size)| BookOrder {
                    side: Side::Bid,
                    price: Decimal::try_from(price).unwrap(),
                    size: Decimal::try_from(size).unwrap(),
                })
                .collect(),
            asks: asks
                .into_iter()
                .map(|(price, size)| BookOrder {
                    side: Side::Ask,
                    price: Decimal::try_from(price).unwrap(),
                    size: Decimal::try_from(size).unwrap(),
                })
                .collect(),
        }
    }

    #[test]
    fn test_snapshot_replaces_state() {
        let mut snap = OrderbookSnapshot::new(OrderBookId::from("ob1"));
        snap.apply(&order_book(true, 1, vec![(50.0, 10.0)], vec![(51.0, 5.0)]));
        assert_eq!(snap.bids().len(), 1);
        assert_eq!(snap.asks().len(), 1);
        assert_eq!(snap.best_bid(), Some(Decimal::try_from(50.0).unwrap()));
        assert_eq!(snap.best_ask(), Some(Decimal::try_from(51.0).unwrap()));

        snap.apply(&order_book(true, 2, vec![(49.0, 20.0)], vec![(52.0, 8.0)]));
        assert_eq!(snap.bids().len(), 1);
        assert_eq!(snap.asks().len(), 1);
        assert_eq!(snap.best_bid(), Some(Decimal::try_from(49.0).unwrap()));
        assert_eq!(snap.best_ask(), Some(Decimal::try_from(52.0).unwrap()));
    }

    #[test]
    fn test_delta_merges_with_snapshot() {
        let mut snap = OrderbookSnapshot::new(OrderBookId::from("ob1"));
        snap.apply(&order_book(true, 1, vec![(50.0, 10.0)], vec![(51.0, 5.0)]));
        snap.apply(&order_book(
            false,
            2,
            vec![(49.0, 15.0), (48.0, 3.0)],
            vec![(52.0, 2.0)],
        ));
        assert_eq!(snap.bids().len(), 3);
        assert_eq!(snap.asks().len(), 2);
        assert_eq!(snap.best_bid(), Some(Decimal::try_from(50.0).unwrap()));
        assert_eq!(snap.best_ask(), Some(Decimal::try_from(51.0).unwrap()));
    }

    #[test]
    fn test_zero_size_removes_level() {
        let mut snap = OrderbookSnapshot::new(OrderBookId::from("ob1"));
        snap.apply(&order_book(true, 1, vec![(50.0, 10.0)], vec![(51.0, 5.0)]));
        snap.apply(&order_book(false, 2, vec![(50.0, 0.0)], vec![]));
        assert_eq!(snap.bids().len(), 0);
        assert_eq!(snap.best_bid(), None);
    }

    #[test]
    fn test_mid_price_and_spread() {
        let mut snap = OrderbookSnapshot::new(OrderBookId::from("ob1"));
        snap.apply(&order_book(true, 1, vec![(50.0, 10.0)], vec![(52.0, 5.0)]));
        assert_eq!(snap.mid_price(), Some(Decimal::try_from(51.0).unwrap()));
        assert_eq!(snap.spread(), Some(Decimal::try_from(2.0).unwrap()));
    }

    #[test]
    fn test_clear() {
        let mut snap = OrderbookSnapshot::new(OrderBookId::from("ob1"));
        snap.apply(&order_book(true, 1, vec![(50.0, 10.0)], vec![(51.0, 5.0)]));
        snap.clear();
        assert!(snap.is_empty());
        assert_eq!(snap.seq, 0);
    }
}
