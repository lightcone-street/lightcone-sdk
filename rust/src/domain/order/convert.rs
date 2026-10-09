//! Conversions: order wire types → [`LimitOrder`].

use super::wire;
use super::{LimitOrder, OrderStatus, UserOpenLimitOrders};
use rust_decimal::Decimal;

impl From<wire::OrderUpdate> for LimitOrder {
    fn from(update: wire::OrderUpdate) -> Self {
        let price = update.price().unwrap_or(Decimal::ZERO);
        LimitOrder {
            status: OrderStatus::derive(
                update.closed_reason.as_deref(),
                update.original_base,
                update.confirmed_base,
                update.pending_base,
            ),
            market_pubkey: update.market_pubkey,
            orderbook_id: update.orderbook_id,
            base_mint: update.base_mint,
            quote_mint: update.quote_mint,
            order_hash: update.order_hash,
            side: update.side,
            price,
            size: update.original_base,
            filled_size: update.confirmed_base,
            pending_size: update.pending_base,
            remaining_size: update.open_base,
            cancelled_size: update.cancelled_base,
            time_in_force: Some(update.tif),
            funding_source: update.funding_source,
            closed_reason: update.closed_reason,
            accepted_seq: update.accepted_seq,
            committed_revision: update.commit.committed_revision,
            created_at: update.accepted_at,
            expiration: update.expiration,
        }
    }
}

impl From<wire::UserSnapshotOrder> for LimitOrder {
    fn from(order: wire::UserSnapshotOrder) -> Self {
        LimitOrder {
            status: OrderStatus::derive(
                order.closed_reason.as_deref(),
                order.original_base,
                order.confirmed_base,
                order.pending_base,
            ),
            market_pubkey: order.market_pubkey,
            orderbook_id: order.orderbook_id,
            base_mint: order.base_mint,
            quote_mint: order.quote_mint,
            order_hash: order.order_hash,
            side: order.side,
            price: order.price,
            size: order.original_base,
            filled_size: order.confirmed_base,
            pending_size: order.pending_base,
            remaining_size: order.open_base,
            cancelled_size: order.cancelled_base,
            time_in_force: Some(order.tif),
            funding_source: order.funding_source,
            closed_reason: order.closed_reason,
            accepted_seq: order.accepted_seq,
            committed_revision: order.committed_revision,
            created_at: order.created_at,
            expiration: order.expiration,
        }
    }
}

impl wire::UserOrder {
    /// Convert a REST order into a [`LimitOrder`]; `None` when the backend
    /// omitted the recorded quantities.
    pub fn into_limit_order(self) -> Option<LimitOrder> {
        let state = self.state?;
        Some(LimitOrder {
            status: OrderStatus::derive(
                state.closed_reason.as_deref(),
                state.original_base,
                state.confirmed_base,
                state.pending_base,
            ),
            market_pubkey: self.market_pubkey,
            orderbook_id: self.orderbook_id,
            base_mint: self.base_mint,
            quote_mint: self.quote_mint,
            order_hash: self.order_hash,
            side: self.side,
            price: self.price,
            size: state.original_base,
            filled_size: state.confirmed_base,
            pending_size: state.pending_base,
            remaining_size: state.open_base,
            cancelled_size: state.cancelled_base,
            time_in_force: self.tif,
            funding_source: self.source,
            closed_reason: state.closed_reason,
            accepted_seq: state.accepted_seq,
            committed_revision: state.committed_revision,
            created_at: self.created_at,
            expiration: self.expiration,
        })
    }
}

/// Build the open-order container from a WS `user` snapshot page.
///
/// Orders that are neither resting nor awaiting fill confirmation are
/// skipped. The snapshot is the first bounded page only; merge later REST
/// pages with [`UserOpenLimitOrders::apply_order`].
pub fn convert_snapshot_orders(orders: Vec<wire::UserSnapshotOrder>) -> UserOpenLimitOrders {
    let mut open_orders = UserOpenLimitOrders::new();
    for order in orders {
        open_orders.apply_order(LimitOrder::from(order));
    }
    open_orders
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::order::wire::tests::{live_order, HASH};
    use crate::domain::position::wire::FundingSource;
    use crate::shared::{OrderBookId, PubkeyStr, Side, TimeInForce};
    use chrono::{DateTime, Utc};

    fn test_time() -> DateTime<Utc> {
        DateTime::<Utc>::from_timestamp_millis(1_790_685_521_784).unwrap()
    }

    fn snapshot_order(hash: &str, open: i64, pending: i64) -> wire::UserSnapshotOrder {
        wire::UserSnapshotOrder {
            order_hash: hash.to_string(),
            market_pubkey: PubkeyStr::from("mkt1"),
            orderbook_id: OrderBookId::from("ob1"),
            side: Side::Ask,
            maker_amount: Decimal::from(10),
            taker_amount: Decimal::from(5),
            original_base: Decimal::from(10),
            confirmed_base: Decimal::from(10 - open - pending),
            pending_base: Decimal::from(pending),
            open_base: Decimal::from(open),
            cancelled_base: Decimal::ZERO,
            closed_reason: None,
            committed_revision: 3,
            accepted_seq: 1,
            price: Decimal::new(5, 1),
            created_at: test_time(),
            expiration: 0,
            base_mint: PubkeyStr::from("b"),
            quote_mint: PubkeyStr::from("q"),
            tif: TimeInForce::Gtc,
            funding_source: FundingSource::Conditional,
        }
    }

    #[test]
    fn live_order_fact_converts_quantities_and_price() {
        let order = LimitOrder::from(live_order(812, "7.00000000", "1.00000000", None));
        assert_eq!(order.order_hash, HASH);
        assert_eq!(order.side, Side::Bid);
        assert_eq!(order.price, Decimal::new(55, 2));
        assert_eq!(order.size, Decimal::from(10));
        assert_eq!(order.filled_size, Decimal::from(2));
        assert_eq!(order.pending_size, Decimal::from(1));
        assert_eq!(order.remaining_size, Decimal::from(7));
        assert_eq!(order.status, OrderStatus::Pending);
        assert_eq!(order.committed_revision, 812);
        assert_eq!(order.accepted_seq, 44);
        assert_eq!(order.funding_source, FundingSource::Global);
    }

    #[test]
    fn closed_reason_wins_status_precedence() {
        let order = LimitOrder::from(live_order(813, "0.00000000", "1.00000000", Some("expired")));
        assert_eq!(order.status, OrderStatus::Closed);
        assert_eq!(order.closed_reason.as_deref(), Some("expired"));
    }

    #[test]
    fn snapshot_conversion_keeps_live_orders_only() {
        let open = convert_snapshot_orders(vec![
            snapshot_order("resting", 4, 0),
            snapshot_order("claiming", 0, 2),
            snapshot_order("done", 0, 0),
        ]);
        let orders = open
            .get(&PubkeyStr::from("mkt1"), &OrderBookId::from("ob1"))
            .unwrap();
        let hashes: Vec<_> = orders
            .iter()
            .map(|order| order.order_hash.as_str())
            .collect();
        assert_eq!(hashes, ["resting", "claiming"]);
        assert_eq!(orders[0].status, OrderStatus::Open);
        assert_eq!(orders[1].status, OrderStatus::Pending);
    }

    #[test]
    fn rest_order_without_state_does_not_convert() {
        let order: wire::UserOrder = serde_json::from_value(serde_json::json!({
            "order_hash": HASH,
            "market_pubkey": "mkt1",
            "orderbook_id": "ob1",
            "side": "ask",
            "amount_in": "1",
            "amount_out": "1",
            "price": "1",
            "created_at": 1,
            "base_mint": "b",
            "quote_mint": "q",
            "state": null,
            "tif": "GTC",
            "source": "conditional"
        }))
        .unwrap();
        assert!(order.into_limit_order().is_none());
    }
}
