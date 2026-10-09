//! Order state containers — app-owned, SDK-provided update logic.

use crate::shared::{OrderBookId, PubkeyStr};

use super::wire::{self, ClosureScope};
#[cfg(feature = "trigger_orders")]
use super::TriggerOrder;
use super::{LimitOrder, OrderStatus};
use rust_decimal::Decimal;
use std::collections::HashMap;

// ─── UserOpenLimitOrders ────────────────────────────────────────────────────

/// Result of applying committed order state to [`UserOpenLimitOrders`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApplyOutcome {
    /// A live order that was not tracked yet.
    Inserted,
    /// A tracked order replaced by newer (or equal) committed state.
    Updated,
    /// A tracked order that is no longer live was removed.
    Removed,
    /// Older than the tracked state (e.g. a live fact racing a snapshot).
    Stale,
    /// Neither live nor tracked; nothing changed.
    Ignored,
}

/// A wallet's live limit orders (resting, or with fills awaiting on-chain
/// confirmation), grouped by market and orderbook.
///
/// Seed it from the WS `user` snapshot ([`convert_snapshot_orders`](super::convert_snapshot_orders))
/// or REST pages, then [`apply`](Self::apply) every live `order` fact. Each
/// fact carries the order's complete state, so application is a
/// revision-guarded replace. The container remembers the revision at which
/// each order stopped being live, and every applied closure, so older state
/// (a REST page or snapshot racing live facts) cannot reopen a closed order.
#[derive(Debug, Clone)]
pub struct UserOpenLimitOrders {
    pub orders: HashMap<PubkeyStr, HashMap<OrderBookId, Vec<LimitOrder>>>,
    /// Committed revision at which each untracked order stopped being live.
    retired: HashMap<String, u64>,
    /// Applied closures, re-applied to older state inserted later.
    closures: Vec<(ClosureScope, wire::ClosureUpdate)>,
}

impl UserOpenLimitOrders {
    pub fn new() -> Self {
        Self {
            orders: HashMap::new(),
            retired: HashMap::new(),
            closures: Vec::new(),
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

    /// Find a tracked order by hash.
    pub fn get_by_hash(&self, order_hash: &str) -> Option<&LimitOrder> {
        self.all().find(|order| order.order_hash == order_hash)
    }

    /// Iterate over every tracked order.
    pub fn all(&self) -> impl Iterator<Item = &LimitOrder> {
        self.orders
            .values()
            .flat_map(|by_orderbook| by_orderbook.values())
            .flat_map(|orders| orders.iter())
    }

    /// Apply a live WS `order` fact.
    pub fn apply(&mut self, update: &wire::OrderUpdate) -> ApplyOutcome {
        self.apply_order(LimitOrder::from(update.clone()))
    }

    /// Same as [`Self::apply`]; kept for callers of the previous API.
    pub fn upsert(&mut self, update: &wire::OrderUpdate) {
        self.apply(update);
    }

    /// Apply converted committed order state (live fact, snapshot, or REST).
    ///
    /// State older than the tracked `committed_revision`, or not newer than
    /// the revision at which the order stopped being live, is ignored. A
    /// closure committed after the state still closes it. A live order
    /// replaces the tracked copy; one that is no longer live is removed.
    pub fn apply_order(&mut self, mut order: LimitOrder) -> ApplyOutcome {
        if let Some(existing) = self.get_by_hash(&order.order_hash) {
            if order.committed_revision < existing.committed_revision {
                return ApplyOutcome::Stale;
            }
        } else if self
            .retired
            .get(&order.order_hash)
            .is_some_and(|&revision| order.committed_revision <= revision)
        {
            return ApplyOutcome::Stale;
        }
        for (scope, closure) in &self.closures {
            if closure.commit.committed_revision > order.committed_revision {
                close_covered(&mut order, closure, *scope);
            }
        }
        let was_tracked = self.take(&order.order_hash).is_some();
        if !order.is_live() {
            retire(&mut self.retired, &order);
            return if was_tracked {
                ApplyOutcome::Removed
            } else {
                ApplyOutcome::Ignored
            };
        }
        self.retired.remove(&order.order_hash);
        self.orders
            .entry(order.market_pubkey.clone())
            .or_default()
            .entry(order.orderbook_id.clone())
            .or_default()
            .push(order);
        if was_tracked {
            ApplyOutcome::Updated
        } else {
            ApplyOutcome::Inserted
        }
    }

    /// Apply a committed closure cutoff locally: orders in scope with
    /// `accepted_seq <= closure.accepted_seq` stop resting (their open base
    /// becomes cancelled) and are removed unless fills still await
    /// confirmation. The container is assumed to hold one wallet's orders.
    ///
    /// Returns the number of orders closed, or `None` when the scope cannot be
    /// evaluated from order data (deposit-token, account, settlement-profile,
    /// or unknown scopes): refetch the orders in that case. Later per-order
    /// facts remain authoritative.
    pub fn apply_closure(&mut self, closure: &wire::ClosureUpdate) -> Option<usize> {
        let scope = closure.scope()?;
        if matches!(
            scope,
            ClosureScope::DepositToken | ClosureScope::Account | ClosureScope::SettlementProfile
        ) {
            return None;
        }
        let mut closed = 0;
        for by_orderbook in self.orders.values_mut() {
            for orders in by_orderbook.values_mut() {
                for order in orders.iter_mut() {
                    if close_covered(order, closure, scope) {
                        closed += 1;
                    }
                }
                orders.retain(|order| {
                    let live = order.is_live();
                    if !live {
                        retire(&mut self.retired, order);
                    }
                    live
                });
            }
        }
        self.closures.push((scope, closure.clone()));
        Some(closed)
    }

    /// Stop tracking an order without recording why; older state can add it
    /// again.
    pub fn remove(&mut self, order_hash: &str) {
        self.take(order_hash);
    }

    fn take(&mut self, order_hash: &str) -> Option<LimitOrder> {
        for by_orderbook in self.orders.values_mut() {
            for orders in by_orderbook.values_mut() {
                if let Some(index) = orders
                    .iter()
                    .position(|order| order.order_hash == order_hash)
                {
                    return Some(orders.remove(index));
                }
            }
        }
        None
    }

    /// Forget every order, retired revision, and closure (e.g. before
    /// reseeding from a new snapshot).
    pub fn clear(&mut self) {
        self.orders.clear();
        self.retired.clear();
        self.closures.clear();
    }

    pub fn is_empty(&self) -> bool {
        self.orders
            .values()
            .all(|by_orderbook| by_orderbook.values().all(|orders| orders.is_empty()))
    }
}

/// Close `order` locally when `closure` covers it: in scope, accepted at or
/// before the cutoff, and still resting. Returns whether it was closed.
fn close_covered(
    order: &mut LimitOrder,
    closure: &wire::ClosureUpdate,
    scope: ClosureScope,
) -> bool {
    let key = closure.scope_key.as_str();
    let in_scope = match scope {
        ClosureScope::Market => order.market_pubkey.as_str() == key,
        ClosureScope::Book => order.orderbook_id.as_str() == key,
        ClosureScope::WalletBook => key
            .split_once(':')
            .is_some_and(|(_, book)| order.orderbook_id.as_str() == book),
        _ => true,
    };
    let covered = i64::try_from(order.accepted_seq).is_ok_and(|seq| seq <= closure.accepted_seq);
    if !(covered && in_scope && order.remaining_size > Decimal::ZERO) {
        return false;
    }
    order.cancelled_size += order.remaining_size;
    order.remaining_size = Decimal::ZERO;
    order
        .closed_reason
        .get_or_insert_with(|| closure.reason.clone());
    order.status = OrderStatus::Closed;
    // Older order facts must not reopen the closed order.
    order.committed_revision = order
        .committed_revision
        .max(closure.commit.committed_revision);
    true
}

/// Record the revision at which `order` stopped being live.
fn retire(retired: &mut HashMap<String, u64>, order: &LimitOrder) {
    let revision = retired.entry(order.order_hash.clone()).or_default();
    *revision = (*revision).max(order.committed_revision);
}

impl Default for UserOpenLimitOrders {
    fn default() -> Self {
        Self::new()
    }
}

// ─── UserTriggerOrders ──────────────────────────────────────────────────────

#[cfg(feature = "trigger_orders")]
#[derive(Debug, Clone)]
pub struct UserTriggerOrders {
    pub orders: HashMap<PubkeyStr, HashMap<OrderBookId, Vec<TriggerOrder>>>,
}

#[cfg(feature = "trigger_orders")]
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

#[cfg(feature = "trigger_orders")]
impl Default for UserTriggerOrders {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::order::wire::tests::live_order;
    use crate::shared::{OrderBookId, Side};

    const MARKET: &str = "A9Bxkkc4nah517EjgjwSafwspGnmU9s1Ei5PTo5ZkJd9";
    const BOOK: &str = "j749bQAbDsBAiyDs2Tj868heQj1b5KVp98ZrjiZd56a";

    fn tracked(container: &UserOpenLimitOrders) -> Vec<&LimitOrder> {
        container
            .get(&PubkeyStr::from(MARKET), &OrderBookId::from(BOOK))
            .map(|orders| orders.iter().collect())
            .unwrap_or_default()
    }

    fn closure(scope_kind: i16, scope_key: &str, accepted_seq: i64) -> wire::ClosureUpdate {
        serde_json::from_value(serde_json::json!({
            "effect_id": "900:0",
            "committed_revision": 900,
            "projection_generation": 1,
            "actionable": true,
            "scope_kind": scope_kind,
            "scope_key": scope_key,
            "accepted_seq": accepted_seq,
            "reason": "cancel_all"
        }))
        .unwrap()
    }

    #[test]
    fn apply_inserts_updates_and_removes_by_liveness() {
        let mut container = UserOpenLimitOrders::new();
        assert_eq!(
            container.apply(&live_order(10, "8.00000000", "0.00000000", None)),
            ApplyOutcome::Inserted
        );
        assert_eq!(
            container.apply(&live_order(11, "5.00000000", "3.00000000", None)),
            ApplyOutcome::Updated
        );
        let orders = tracked(&container);
        assert_eq!(orders.len(), 1);
        assert_eq!(orders[0].remaining_size, Decimal::from(5));
        assert_eq!(orders[0].pending_size, Decimal::from(3));
        assert_eq!(orders[0].side, Side::Bid);

        // Pending fills keep a fully matched order live until confirmation.
        assert_eq!(
            container.apply(&live_order(12, "0.00000000", "3.00000000", None)),
            ApplyOutcome::Updated
        );
        assert_eq!(
            container.apply(&live_order(13, "0.00000000", "0.00000000", None)),
            ApplyOutcome::Removed
        );
        assert!(container.is_empty());
        assert_eq!(
            container.apply(&live_order(14, "0.00000000", "0.00000000", None)),
            ApplyOutcome::Ignored
        );
    }

    #[test]
    fn apply_ignores_older_revisions() {
        let mut container = UserOpenLimitOrders::new();
        container.apply(&live_order(20, "5.00000000", "0.00000000", None));
        assert_eq!(
            container.apply(&live_order(19, "8.00000000", "0.00000000", None)),
            ApplyOutcome::Stale
        );
        assert_eq!(tracked(&container)[0].remaining_size, Decimal::from(5));
    }

    #[test]
    fn upsert_remains_an_alias_for_apply() {
        let mut container = UserOpenLimitOrders::new();
        container.upsert(&live_order(1, "5.00000000", "0.00000000", None));
        assert!(!container.is_empty());
        container.upsert(&live_order(
            2,
            "0.00000000",
            "0.00000000",
            Some("cancelled"),
        ));
        assert!(container.is_empty());
    }

    #[test]
    fn closure_closes_orders_in_scope_up_to_the_cutoff() {
        let mut container = UserOpenLimitOrders::new();
        container.apply(&live_order(10, "8.00000000", "0.00000000", None));
        // Cutoff below the order's accepted_seq (44) leaves it resting.
        assert_eq!(container.apply_closure(&closure(2, BOOK, 43)), Some(0));
        assert_eq!(tracked(&container).len(), 1);
        // Other books are out of scope.
        assert_eq!(
            container.apply_closure(&closure(2, "OtherBook", 44)),
            Some(0)
        );
        assert_eq!(
            container.apply_closure(&closure(5, &format!("wallet:{BOOK}"), 44)),
            Some(1)
        );
        assert!(container.is_empty());
    }

    #[test]
    fn closure_keeps_pending_claims_visible() {
        let mut container = UserOpenLimitOrders::new();
        container.apply(&live_order(10, "5.00000000", "3.00000000", None));
        assert_eq!(container.apply_closure(&closure(4, "wallet", 50)), Some(1));
        let order = tracked(&container)[0];
        assert_eq!(order.remaining_size, Decimal::ZERO);
        assert_eq!(order.cancelled_size, Decimal::from(5));
        assert_eq!(order.status, OrderStatus::Closed);
        assert_eq!(order.closed_reason.as_deref(), Some("cancel_all"));
        assert_eq!(order.committed_revision, 900);
        // A fact older than the closure cannot reopen the order.
        assert_eq!(
            container.apply(&live_order(11, "5.00000000", "3.00000000", None)),
            ApplyOutcome::Stale
        );
    }

    #[test]
    fn older_state_cannot_reopen_an_order_a_closure_removed() {
        let mut container = UserOpenLimitOrders::new();
        container.apply(&live_order(10, "8.00000000", "0.00000000", None));
        assert_eq!(container.apply_closure(&closure(4, "wallet", 50)), Some(1));
        assert!(container.is_empty());
        // A REST page or snapshot captured before the closure arrives late.
        assert_eq!(
            container.apply(&live_order(11, "8.00000000", "0.00000000", None)),
            ApplyOutcome::Stale
        );
        assert!(container.is_empty());
    }

    #[test]
    fn older_state_cannot_reopen_a_removed_order() {
        let mut container = UserOpenLimitOrders::new();
        container.apply(&live_order(10, "8.00000000", "0.00000000", None));
        assert_eq!(
            container.apply(&live_order(
                12,
                "0.00000000",
                "0.00000000",
                Some("cancelled")
            )),
            ApplyOutcome::Removed
        );
        assert_eq!(
            container.apply(&live_order(11, "8.00000000", "0.00000000", None)),
            ApplyOutcome::Stale
        );
        assert_eq!(
            container.apply(&live_order(12, "8.00000000", "0.00000000", None)),
            ApplyOutcome::Stale
        );
        assert!(container.is_empty());
        // Newer committed state stays authoritative.
        assert_eq!(
            container.apply(&live_order(13, "8.00000000", "0.00000000", None)),
            ApplyOutcome::Inserted
        );
    }

    #[test]
    fn closure_applies_to_older_state_seeded_after_it() {
        let mut container = UserOpenLimitOrders::new();
        assert_eq!(container.apply_closure(&closure(4, "wallet", 50)), Some(0));
        assert_eq!(
            container.apply(&live_order(11, "8.00000000", "0.00000000", None)),
            ApplyOutcome::Ignored
        );
        assert!(container.is_empty());

        // Pending fills keep the closed order visible, as with a tracked order.
        container.clear();
        container.apply_closure(&closure(4, "wallet", 50));
        assert_eq!(
            container.apply(&live_order(11, "5.00000000", "3.00000000", None)),
            ApplyOutcome::Inserted
        );
        let order = tracked(&container)[0];
        assert_eq!(order.remaining_size, Decimal::ZERO);
        assert_eq!(order.cancelled_size, Decimal::from(5));
        assert_eq!(order.status, OrderStatus::Closed);
        assert_eq!(order.committed_revision, 900);

        // Orders accepted after the cutoff, and newer state, are unaffected.
        container.clear();
        container.apply_closure(&closure(4, "wallet", 43));
        assert_eq!(
            container.apply(&live_order(11, "8.00000000", "0.00000000", None)),
            ApplyOutcome::Inserted
        );
        container.clear();
        container.apply_closure(&closure(4, "wallet", 50));
        assert_eq!(
            container.apply(&live_order(901, "8.00000000", "0.00000000", None)),
            ApplyOutcome::Inserted
        );
    }

    #[test]
    fn clear_forgets_retired_orders_and_closures() {
        let mut container = UserOpenLimitOrders::new();
        container.apply(&live_order(10, "8.00000000", "0.00000000", None));
        container.apply_closure(&closure(4, "wallet", 50));
        container.clear();
        assert_eq!(
            container.apply(&live_order(11, "8.00000000", "0.00000000", None)),
            ApplyOutcome::Inserted
        );
    }

    #[test]
    fn closure_scopes_without_order_data_request_a_refresh() {
        let mut container = UserOpenLimitOrders::new();
        container.apply(&live_order(10, "5.00000000", "0.00000000", None));
        assert_eq!(container.apply_closure(&closure(3, "mint", 50)), None);
        assert_eq!(container.apply_closure(&closure(99, "x", 50)), None);
        assert_eq!(tracked(&container).len(), 1);
    }

    #[test]
    fn remove_and_clear() {
        let mut container = UserOpenLimitOrders::new();
        container.apply(&live_order(10, "5.00000000", "0.00000000", None));
        assert!(container
            .get_by_hash(crate::domain::order::wire::tests::HASH)
            .is_some());
        container.remove(crate::domain::order::wire::tests::HASH);
        assert!(container.is_empty());
        container.apply(&live_order(11, "5.00000000", "0.00000000", None));
        container.clear();
        assert!(container.is_empty());
    }

    // ── UserTriggerOrders tests ─────────────────────────────────────────────

    #[cfg(feature = "trigger_orders")]
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
    #[cfg(feature = "trigger_orders")]
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
    #[cfg(feature = "trigger_orders")]
    fn test_trigger_orders_get_by_id() {
        let mut container = UserTriggerOrders::new();
        container.insert(make_trigger_order("t1", "mkt1", "ob1"));
        container.insert(make_trigger_order("t2", "mkt1", "ob2"));

        let order = container.get_by_id("t2").unwrap();
        assert_eq!(order.trigger_order_id, "t2");
        assert!(container.get_by_id("t99").is_none());
    }

    #[test]
    #[cfg(feature = "trigger_orders")]
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
    #[cfg(feature = "trigger_orders")]
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
    #[cfg(feature = "trigger_orders")]
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
    #[cfg(feature = "trigger_orders")]
    fn test_trigger_orders_clear() {
        let mut container = UserTriggerOrders::new();
        container.insert(make_trigger_order("t1", "mkt1", "ob1"));
        container.insert(make_trigger_order("t2", "mkt1", "ob2"));
        container.clear();
        assert!(container.is_empty());
    }

    #[test]
    #[cfg(feature = "trigger_orders")]
    fn test_trigger_orders_all() {
        let mut container = UserTriggerOrders::new();
        container.insert(make_trigger_order("t1", "mkt1", "ob1"));
        container.insert(make_trigger_order("t2", "mkt2", "ob2"));
        let all: Vec<_> = container.all().collect();
        assert_eq!(all.len(), 2);
    }
}
