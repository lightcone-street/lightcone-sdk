//! Orderbook state containers — app-owned, SDK-provided update logic.
//!
//! The `book_update` stream is snapshot-only: every data frame carries the
//! full top-20 levels per side and replaces the previous book wholesale
//! (last-write-wins). Consumers holding multiple aggregation views of one
//! orderbook on the same connection key their [`OrderbookState`] instances by
//! `(orderbook_id, aggregation)` using
//! [`OrderBook::aggregation`](crate::domain::orderbook::wire::OrderBook::aggregation).

use crate::domain::orderbook::wire::OrderBook;
use crate::shared::OrderBookId;
use rust_decimal::Decimal;
use std::collections::BTreeMap;

/// Result of applying a WS orderbook frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplyResult {
    Applied,
    RefreshRequired(RefreshReason),
}

/// A dropped frame that means consumers should refresh the subscription.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RefreshReason {
    /// The backend explicitly requested a resync: unsubscribe and
    /// re-subscribe with the same parameters (including aggregation) to
    /// receive a fresh snapshot.
    ServerResync,
}

/// Live orderbook state replaced wholesale by snapshot frames.
///
/// The app owns instances of this type (e.g. inside a Dioxus `Signal`).
/// The SDK provides the update methods.
#[derive(Debug, Clone, Default)]
pub struct OrderbookState {
    pub orderbook_id: OrderBookId,
    /// Projection version of the last applied frame. Strictly increasing but
    /// non-contiguous server-side (conflation skips versions), and the
    /// initial snapshot after every (re)subscribe is `seq: 0` — informational
    /// only, never used to gate frames.
    pub seq: u64,
    bids: BTreeMap<Decimal, Decimal>,
    asks: BTreeMap<Decimal, Decimal>,
}

impl OrderbookState {
    pub fn new(orderbook_id: OrderBookId) -> Self {
        Self {
            orderbook_id,
            seq: 0,
            bids: BTreeMap::new(),
            asks: BTreeMap::new(),
        }
    }

    /// Apply a WS orderbook frame (snapshot-only stream, last-write-wins).
    ///
    /// `resync` frames take precedence and leave the book untouched — the
    /// caller must re-subscribe with the same parameters. Every other data
    /// frame is a full snapshot by contract and replaces the book wholesale
    /// (the `is_snapshot` flag is not consulted), including the `seq: 0`
    /// initial snapshot delivered after every (re)subscribe: gating on `seq`
    /// would freeze the book after a resync or aggregation change, so `seq`
    /// is stored as informational only.
    pub fn apply(&mut self, book: &OrderBook) -> ApplyResult {
        if book.resync {
            return ApplyResult::RefreshRequired(RefreshReason::ServerResync);
        }

        self.bids.clear();
        self.asks.clear();

        for order in &book.bids {
            if !order.size.is_zero() {
                self.bids.insert(order.price, order.size);
            }
        }
        for order in &book.asks {
            if !order.size.is_zero() {
                self.asks.insert(order.price, order.size);
            }
        }
        self.seq = book.seq;

        ApplyResult::Applied
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
    use crate::domain::orderbook::wire::WsBookLevel;
    use crate::shared::Side;
    use rust_decimal::Decimal;

    fn order_book(
        snapshot: bool,
        seq: u64,
        bids: Vec<(f64, f64)>,
        asks: Vec<(f64, f64)>,
    ) -> OrderBook {
        OrderBook {
            id: OrderBookId::from("ob_test"),
            is_snapshot: snapshot,
            seq,
            resync: false,
            bids: bids
                .into_iter()
                .map(|(price, size)| WsBookLevel {
                    side: Side::Bid,
                    price: Decimal::try_from(price).unwrap(),
                    size: Decimal::try_from(size).unwrap(),
                })
                .collect(),
            asks: asks
                .into_iter()
                .map(|(price, size)| WsBookLevel {
                    side: Side::Ask,
                    price: Decimal::try_from(price).unwrap(),
                    size: Decimal::try_from(size).unwrap(),
                })
                .collect(),
            n_sig_figs: None,
            mantissa: None,
        }
    }

    #[test]
    fn test_snapshot_replaces_state() {
        let mut snap = OrderbookState::new(OrderBookId::from("ob1"));
        assert_eq!(
            snap.apply(&order_book(true, 1, vec![(50.0, 10.0)], vec![(51.0, 5.0)])),
            ApplyResult::Applied
        );
        assert_eq!(snap.bids().len(), 1);
        assert_eq!(snap.asks().len(), 1);
        assert_eq!(snap.best_bid(), Some(Decimal::try_from(50.0).unwrap()));
        assert_eq!(snap.best_ask(), Some(Decimal::try_from(51.0).unwrap()));

        assert_eq!(
            snap.apply(&order_book(true, 2, vec![(49.0, 20.0)], vec![(52.0, 8.0)])),
            ApplyResult::Applied
        );
        assert_eq!(snap.bids().len(), 1);
        assert_eq!(snap.asks().len(), 1);
        assert_eq!(snap.best_bid(), Some(Decimal::try_from(49.0).unwrap()));
        assert_eq!(snap.best_ask(), Some(Decimal::try_from(52.0).unwrap()));
    }

    #[test]
    fn test_lower_seq_snapshot_still_applies_last_write_wins() {
        let mut snap = OrderbookState::new(OrderBookId::from("ob1"));
        assert_eq!(
            snap.apply(&order_book(true, 42, vec![(50.0, 10.0)], vec![])),
            ApplyResult::Applied
        );
        assert_eq!(snap.seq, 42);

        // A snapshot with a lower seq (e.g. queued behind a re-subscribe)
        // still replaces the book — seq never gates.
        assert_eq!(
            snap.apply(&order_book(true, 7, vec![(49.0, 20.0)], vec![])),
            ApplyResult::Applied
        );
        assert_eq!(snap.seq, 7);
        assert_eq!(snap.best_bid(), Some(Decimal::try_from(49.0).unwrap()));
    }

    #[test]
    fn test_post_resync_seq_zero_snapshot_applies() {
        let mut snap = OrderbookState::new(OrderBookId::from("ob1"));
        assert_eq!(
            snap.apply(&order_book(true, 42, vec![(50.0, 10.0)], vec![(51.0, 5.0)])),
            ApplyResult::Applied
        );

        let mut resync = order_book(false, 0, vec![], vec![]);
        resync.resync = true;
        assert_eq!(
            snap.apply(&resync),
            ApplyResult::RefreshRequired(RefreshReason::ServerResync)
        );
        // Resync leaves the book untouched.
        assert_eq!(snap.seq, 42);
        assert_eq!(snap.bids().len(), 1);

        // The fresh snapshot after re-subscribing is always seq 0 and MUST
        // apply — gating on seq here would freeze the book forever.
        assert_eq!(
            snap.apply(&order_book(true, 0, vec![(48.0, 5.0)], vec![(52.0, 2.0)])),
            ApplyResult::Applied
        );
        assert_eq!(snap.seq, 0);
        assert_eq!(snap.best_bid(), Some(Decimal::try_from(48.0).unwrap()));
        assert_eq!(snap.best_ask(), Some(Decimal::try_from(52.0).unwrap()));
    }

    #[test]
    fn test_data_frames_replace_regardless_of_snapshot_flag() {
        let mut snap = OrderbookState::new(OrderBookId::from("ob1"));
        assert_eq!(
            snap.apply(&order_book(true, 1, vec![(50.0, 10.0)], vec![(51.0, 5.0)])),
            ApplyResult::Applied
        );

        // Every non-resync data frame is a snapshot by contract — the
        // is_snapshot flag is not consulted, so a server omitting it cannot
        // freeze the book.
        assert_eq!(
            snap.apply(&order_book(false, 2, vec![(49.0, 20.0)], vec![])),
            ApplyResult::Applied
        );
        assert_eq!(snap.seq, 2);
        assert_eq!(snap.bids().len(), 1);
        assert_eq!(snap.asks().len(), 0);
        assert_eq!(snap.best_bid(), Some(Decimal::try_from(49.0).unwrap()));
    }

    #[test]
    fn test_zero_size_levels_are_skipped() {
        let mut snap = OrderbookState::new(OrderBookId::from("ob1"));
        assert_eq!(
            snap.apply(&order_book(
                true,
                1,
                vec![(50.0, 10.0), (49.0, 0.0)],
                vec![(51.0, 5.0)],
            )),
            ApplyResult::Applied
        );
        assert_eq!(snap.bids().len(), 1);
        assert_eq!(snap.best_bid(), Some(Decimal::try_from(50.0).unwrap()));
    }

    #[test]
    fn test_mid_price_and_spread() {
        let mut snap = OrderbookState::new(OrderBookId::from("ob1"));
        assert_eq!(
            snap.apply(&order_book(true, 1, vec![(50.0, 10.0)], vec![(52.0, 5.0)])),
            ApplyResult::Applied
        );
        assert_eq!(snap.mid_price(), Some(Decimal::try_from(51.0).unwrap()));
        assert_eq!(snap.spread(), Some(Decimal::try_from(2.0).unwrap()));
    }

    #[test]
    fn test_clear() {
        let mut snap = OrderbookState::new(OrderBookId::from("ob1"));
        assert_eq!(
            snap.apply(&order_book(true, 1, vec![(50.0, 10.0)], vec![(51.0, 5.0)])),
            ApplyResult::Applied
        );
        snap.clear();
        assert!(snap.is_empty());
        assert_eq!(snap.seq, 0);
    }
}
