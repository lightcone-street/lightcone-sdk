//! Wire types for committed order state (REST) and the WS `user` channel.
//!
//! Quantities named `*_base` are decimal strings in base-token units. Order
//! amounts (`amount_in`/`amount_out`, `maker_amount`/`taker_amount`) are in the
//! units of the token each side gives or receives: a bid gives quote and
//! receives base, an ask gives base and receives quote. Revisions and
//! sequence numbers are exact integers that arrive as JSON strings on REST and
//! in WS snapshots but as JSON numbers in live WS facts; both are accepted.

use super::OrderStatus;
use crate::domain::position::wire::{FundingAccount, FundingSource, FundingUpdate};
use crate::domain::trade::wire::WsTrade;
use crate::shared::{serde_util, OrderBookId, PubkeyStr, Side, TimeInForce};
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

// ─── Committed order state (REST) ────────────────────────────────────────────

/// Current committed cumulative state of an order, returned by order
/// submission. All `*_base` quantities are in base-token units.
///
/// `original_base = confirmed_base + pending_base + open_base + cancelled_base`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrderState {
    /// Base size of the signed order.
    pub original_base: Decimal,
    /// Base filled by authenticated (on-chain confirmed) executions.
    pub confirmed_base: Decimal,
    /// Base matched but awaiting on-chain confirmation.
    pub pending_base: Decimal,
    /// Base resting on the book.
    pub open_base: Decimal,
    /// Base of `open_base` the engine will currently match.
    pub executable_base: Decimal,
    /// Base cancelled: explicitly, by expiry, IOC/FOK remainder, or closure.
    pub cancelled_base: Decimal,
    /// Whether the order is live for matching at this revision.
    pub ready: bool,
    /// Why the order stopped resting, when it did.
    #[serde(default, deserialize_with = "serde_util::deserialize_nonempty_string")]
    pub closed_reason: Option<String>,
    #[serde(with = "serde_util::u64_text")]
    pub committed_revision: u64,
    /// Engine acceptance sequence (orders the wallet's accepted orders).
    #[serde(with = "serde_util::u64_text")]
    pub accepted_seq: u64,
}

/// Order quantities recorded at one database snapshot (REST user orders).
/// Unlike [`OrderState`] it makes no claim about live executability.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordedOrderState {
    pub original_base: Decimal,
    pub confirmed_base: Decimal,
    pub pending_base: Decimal,
    pub open_base: Decimal,
    pub cancelled_base: Decimal,
    #[serde(default, deserialize_with = "serde_util::deserialize_nonempty_string")]
    pub closed_reason: Option<String>,
    #[serde(with = "serde_util::u64_text")]
    pub committed_revision: u64,
    #[serde(with = "serde_util::u64_text")]
    pub accepted_seq: u64,
}

/// Progress of the executions the engine selected when it accepted an order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InitialCohortState {
    /// The order did not cross the book at acceptance.
    NoInitialExecutions,
    /// Some initial executions still await on-chain confirmation.
    Pending,
    /// Every initial execution is confirmed or failed.
    Complete,
    /// A state this SDK version does not know.
    #[serde(other)]
    Unknown,
}

/// The fixed set of executions selected at acceptance. All `*_base` fields
/// are in base-token units.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InitialCohort {
    pub state: InitialCohortState,
    /// Base selected for immediate execution.
    pub selected_base: Decimal,
    /// Base of the selection known to be confirmed on-chain.
    pub known_confirmed_base: Decimal,
    /// Base of the selection whose settlement failed and returned unfilled.
    pub known_failed_unfilled_base: Decimal,
    /// Base of the selection not yet resolved.
    pub unresolved_base: Decimal,
}

/// Commit metadata carried by every live committed fact on the WS `user`
/// and `trades` channels (flattened into the payload).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommitInfo {
    /// Durable publication id (`"<revision>:<index>"`).
    pub effect_id: String,
    /// Committed database revision that produced this fact.
    #[serde(with = "serde_util::u64_text")]
    pub committed_revision: u64,
    /// Projection generation; a change means state was rebuilt.
    #[serde(with = "serde_util::u64_text")]
    pub projection_generation: u64,
    /// False when the fact was replayed after it stopped being live
    /// (readiness fields are then forced off).
    #[serde(default)]
    pub actionable: bool,
}

// ─── REST user orders ────────────────────────────────────────────────────────

/// One open or pending order from `GET /api/users/orders`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserOrder {
    pub order_hash: String,
    pub market_pubkey: PubkeyStr,
    pub orderbook_id: OrderBookId,
    pub side: Side,
    /// Amount the maker gives: quote units for bids, base units for asks.
    pub amount_in: Decimal,
    /// Amount the maker receives: base units for bids, quote units for asks.
    pub amount_out: Decimal,
    /// Implied limit price, quote units per base unit.
    pub price: Decimal,
    #[serde(with = "serde_util::timestamp_ms")]
    pub created_at: DateTime<Utc>,
    /// Unix seconds; `0` means no expiration.
    #[serde(default)]
    pub expiration: i64,
    pub base_mint: PubkeyStr,
    pub quote_mint: PubkeyStr,
    /// Outcome index of the base token; the committed backend reports `-1`.
    #[serde(default = "unknown_outcome_index")]
    pub outcome_index: i32,
    /// Recorded quantities; `None` if the engine omitted them.
    #[serde(default)]
    pub state: Option<RecordedOrderState>,
    /// `None` when the backend reports a policy this SDK version does not
    /// know (it sends `"unsupported"`).
    #[serde(default, deserialize_with = "serde_util::known_or_none")]
    pub tif: Option<TimeInForce>,
    /// Custody source that funds the order.
    pub source: FundingSource,
    /// Always `"limit"`.
    #[serde(default)]
    pub order_type: String,
}

fn unknown_outcome_index() -> i32 {
    -1
}

// ─── REST user order fills ───────────────────────────────────────────────────

/// Response from `GET /api/users/order-fills` and
/// `GET /api/users/{wallet}/order-fills`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserOrderFillsResponse {
    pub orders: Vec<UserOrderFill>,
    /// Summary cursor; pass back as `cursor` for the next page of orders.
    pub next_cursor: Option<String>,
    pub has_more: bool,
    /// Database revision of the page.
    #[serde(with = "serde_util::u64_text")]
    pub committed_revision: u64,
}

/// An order the wallet participated in, with its first page of fills.
/// All `*_base` quantities are in base-token units.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserOrderFill {
    pub order_hash: String,
    pub market_pubkey: PubkeyStr,
    pub orderbook_id: OrderBookId,
    pub side: Side,
    pub base_mint: PubkeyStr,
    pub quote_mint: PubkeyStr,
    pub original_base: Decimal,
    pub confirmed_base: Decimal,
    pub pending_base: Decimal,
    pub open_base: Decimal,
    pub cancelled_base: Decimal,
    /// Quote exchanged by confirmed fills, in quote-token units.
    pub confirmed_quote: Decimal,
    pub status: OrderStatus,
    #[serde(default, deserialize_with = "serde_util::deserialize_nonempty_string")]
    pub closed_reason: Option<String>,
    #[serde(with = "serde_util::timestamp_ms")]
    pub created_at: DateTime<Utc>,
    /// Oldest-first page of at most 16 fills.
    pub fills: Vec<OrderFillEvent>,
    #[serde(default)]
    pub fills_has_more: bool,
    /// Continue with `order_hash` + `fill_cursor` on the order-fills endpoint.
    #[serde(default)]
    pub fills_next_cursor: Option<String>,
}

/// A single authenticated fill of an order.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrderFillEvent {
    /// `"<execution_id>:<leg_index>:<projection_generation>"`, the same id as
    /// REST `trade_id`.
    pub fill_id: String,
    pub counterparty: PubkeyStr,
    pub counterparty_order_hash: String,
    /// This order's role in the fill.
    pub role: Role,
    /// Filled size in base-token units.
    pub base_amount: Decimal,
    /// Filled notional in quote-token units.
    pub quote_amount: Decimal,
    /// Signed fee estimate in raw atoms of `fee_mint` (negative = rebate).
    /// An estimate from the captured rate, never a balance delta.
    #[serde(with = "serde_util::i128_text")]
    pub fee_estimate_atoms: i128,
    pub fee_mint: PubkeyStr,
    /// Fee rates captured for the fill, in basis points.
    pub maker_fee_bps: i16,
    pub taker_fee_bps: i16,
    pub tx_signature: String,
    #[serde(with = "serde_util::timestamp_ms")]
    pub filled_at: DateTime<Utc>,
}

impl OrderFillEvent {
    /// Fill price in quote units per base unit (`quote_amount / base_amount`).
    pub fn price(&self) -> Option<Decimal> {
        quote_per_base(self.quote_amount, self.base_amount)
    }
}

/// Whether the order was the maker or taker of a fill.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Maker,
    Taker,
    /// A role this SDK version does not know.
    #[serde(other)]
    Unknown,
}

// ─── WS user channel ─────────────────────────────────────────────────────────

/// One open or pending order in the WS `user` snapshot. Quantities are the
/// same recorded values as REST, flattened into the order.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UserSnapshotOrder {
    pub order_hash: String,
    pub market_pubkey: PubkeyStr,
    pub orderbook_id: OrderBookId,
    #[serde(with = "serde_util::side_text_or_number")]
    pub side: Side,
    /// Amount the maker gives: quote units for bids, base units for asks.
    pub maker_amount: Decimal,
    /// Amount the maker receives: base units for bids, quote units for asks.
    pub taker_amount: Decimal,
    pub original_base: Decimal,
    pub confirmed_base: Decimal,
    pub pending_base: Decimal,
    pub open_base: Decimal,
    pub cancelled_base: Decimal,
    /// Empty on the wire when the order is not closed.
    #[serde(default, deserialize_with = "serde_util::deserialize_nonempty_string")]
    pub closed_reason: Option<String>,
    #[serde(with = "serde_util::u64_text")]
    pub committed_revision: u64,
    #[serde(with = "serde_util::u64_text")]
    pub accepted_seq: u64,
    /// Implied limit price, quote units per base unit.
    pub price: Decimal,
    #[serde(with = "serde_util::timestamp_ms")]
    pub created_at: DateTime<Utc>,
    /// Unix seconds; `0` means no expiration.
    #[serde(default)]
    pub expiration: i64,
    pub base_mint: PubkeyStr,
    pub quote_mint: PubkeyStr,
    pub tif: TimeInForce,
    pub funding_source: FundingSource,
}

/// WS `user` snapshot, sent once per subscription (`event_type: "snapshot"`).
///
/// It holds the first bounded page of open/pending orders and funding
/// accounts; fetch further pages over REST with the cursors.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct UserSnapshot {
    pub orders: Vec<UserSnapshotOrder>,
    pub funding_accounts: Vec<FundingAccount>,
    /// Order cursor for `GET /api/users/orders` (empty on the wire when absent).
    #[serde(default, deserialize_with = "serde_util::deserialize_nonempty_string")]
    pub next_cursor: Option<String>,
    #[serde(default)]
    pub has_more: bool,
    /// Funding-account cursor (empty on the wire when absent).
    #[serde(default, deserialize_with = "serde_util::deserialize_nonempty_string")]
    pub next_funding_cursor: Option<String>,
    #[serde(default)]
    pub funding_has_more: bool,
    /// Database revision of the snapshot (a string here, a number in live facts).
    #[serde(with = "serde_util::u64_text")]
    pub committed_revision: u64,
    #[serde(with = "serde_util::u64_text")]
    pub projection_generation: u64,
    #[serde(default)]
    pub notifications: Vec<crate::domain::notification::Notification>,
}

/// Live committed order fact on the WS `user` channel (`event_type: "order"`).
///
/// Each fact is the order's complete cumulative state at `commit`; apply it
/// by replacing the stored order (see
/// [`UserOpenLimitOrders::apply`](super::UserOpenLimitOrders::apply)).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OrderUpdate {
    #[serde(flatten)]
    pub commit: CommitInfo,
    pub user_pubkey: PubkeyStr,
    pub order_hash: String,
    pub market_pubkey: PubkeyStr,
    pub orderbook_id: OrderBookId,
    pub base_mint: PubkeyStr,
    pub quote_mint: PubkeyStr,
    /// Numeric (`0` bid, `1` ask) on the wire.
    #[serde(with = "serde_util::side_text_or_number")]
    pub side: Side,
    /// Amount the maker gives: quote units for bids, base units for asks.
    pub maker_amount: Decimal,
    /// Amount the maker receives: base units for bids, quote units for asks.
    pub taker_amount: Decimal,
    pub original_base: Decimal,
    pub confirmed_base: Decimal,
    pub pending_base: Decimal,
    pub open_base: Decimal,
    /// Base of `open_base` the engine will currently match.
    pub executable_base: Decimal,
    pub cancelled_base: Decimal,
    #[serde(with = "serde_util::u64_text")]
    pub accepted_seq: u64,
    pub accepted_at: DateTime<Utc>,
    /// Unix seconds; `0` means no expiration.
    #[serde(default)]
    pub expiration: i64,
    pub tif: TimeInForce,
    pub funding_source: FundingSource,
    pub funding_account: PubkeyStr,
    #[serde(default, deserialize_with = "serde_util::deserialize_nonempty_string")]
    pub closed_reason: Option<String>,
    /// Whether the order is live for matching (false when not actionable).
    #[serde(default)]
    pub ready: bool,
}

impl OrderUpdate {
    /// Limit price in quote units per base unit, from the signed amounts.
    pub fn price(&self) -> Option<Decimal> {
        limit_price(self.side, self.maker_amount, self.taker_amount)
    }
}

/// Scope kinds of a committed closure (`ClosureUpdate::scope_kind`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ClosureScope {
    /// Every order on the exchange.
    Exchange,
    /// Orders of one market (`scope_key` = market pubkey).
    Market,
    /// Orders of one orderbook (`scope_key` = orderbook id).
    Book,
    /// Orders collateralized by one deposit token (`scope_key` = mint).
    DepositToken,
    /// Orders of one wallet (`scope_key` = wallet).
    Wallet,
    /// Orders of one wallet on one book (`scope_key` = `"<wallet>:<book>"`).
    WalletBook,
    /// One custody account.
    Account,
    /// One settlement profile.
    SettlementProfile,
}

impl ClosureScope {
    /// Map the wire `scope_kind` code.
    pub fn from_code(code: i16) -> Option<Self> {
        Some(match code {
            0 => Self::Exchange,
            1 => Self::Market,
            2 => Self::Book,
            3 => Self::DepositToken,
            4 => Self::Wallet,
            5 => Self::WalletBook,
            6 => Self::Account,
            7 => Self::SettlementProfile,
            _ => return None,
        })
    }
}

/// Committed accepted-order cutoff (`event_type: "closure"`): every order in
/// scope with `accepted_seq <= accepted_seq` is closed. Wallet-scoped
/// closures go to that wallet; other scopes are broadcast to every user
/// subscription. Individual order facts may follow later.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ClosureUpdate {
    #[serde(flatten)]
    pub commit: CommitInfo,
    /// Raw scope code; see [`Self::scope`].
    pub scope_kind: i16,
    pub scope_key: String,
    /// Inclusive acceptance-sequence cutoff.
    pub accepted_seq: i64,
    pub reason: String,
}

impl ClosureUpdate {
    /// Typed scope, or `None` for an unknown code.
    pub fn scope(&self) -> Option<ClosureScope> {
        ClosureScope::from_code(self.scope_kind)
    }
}

/// The engine finished rebuilding committed state
/// (`event_type: "recovery_completed"`). Cached user state from before the
/// rebuild is stale: resubscribe or refetch.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecoveryCompleted {
    #[serde(flatten)]
    pub commit: CommitInfo,
    pub recovered_generation: i64,
}

/// WS notification push event.
#[derive(Deserialize, Debug, Clone, PartialEq)]
pub struct NotificationUpdate {
    pub notification: crate::domain::notification::Notification,
}

/// WS `user` channel payload, tagged by `event_type`.
#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(tag = "event_type")]
pub enum UserUpdate {
    #[serde(rename = "snapshot")]
    Snapshot(UserSnapshot),
    #[serde(rename = "order")]
    Order(OrderUpdate),
    #[serde(rename = "funding")]
    Funding(FundingUpdate),
    /// A fill involving the wallet as maker or taker (same payload as the
    /// `trades` channel).
    #[serde(rename = "fill")]
    Fill(WsTrade),
    #[serde(rename = "closure")]
    Closure(ClosureUpdate),
    #[serde(rename = "recovery_completed")]
    RecoveryCompleted(RecoveryCompleted),
    #[serde(rename = "notification")]
    Notification(NotificationUpdate),
    /// An event type this SDK version does not know; safe to ignore.
    #[serde(other)]
    Unknown,
}

/// WS auth update.
#[derive(Deserialize, Debug, Clone)]
pub struct Authenticated {
    pub wallet: PubkeyStr,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "status")]
pub enum AuthUpdate {
    #[serde(rename = "authenticated")]
    Authenticated(Authenticated),
    #[serde(rename = "anonymous")]
    Anonymous {
        #[serde(default)]
        reason: Option<String>,
    },
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

/// Quote per base, normalized; `None` for a zero base amount.
pub(crate) fn quote_per_base(quote: Decimal, base: Decimal) -> Option<Decimal> {
    if base.is_zero() {
        return None;
    }
    quote.checked_div(base).map(|price| price.normalize())
}

/// Limit price of signed amounts: a bid gives quote for base, an ask gives
/// base for quote.
pub(crate) fn limit_price(
    side: Side,
    maker_amount: Decimal,
    taker_amount: Decimal,
) -> Option<Decimal> {
    match side {
        Side::Bid => quote_per_base(maker_amount, taker_amount),
        Side::Ask => quote_per_base(taker_amount, maker_amount),
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::ws::{Kind, MessageIn};

    const WALLET: &str = "Wallet1111111111111111111111111111111111111";
    pub(crate) const HASH: &str =
        "4f1a4b1ab1c0c0ffee0000000000000000000000000000000000000000000001";

    fn user(data: serde_json::Value) -> UserUpdate {
        let message: MessageIn = serde_json::from_value(serde_json::json!({
            "type": "user",
            "version": 0.1,
            "data": data
        }))
        .unwrap();
        match message.kind {
            Kind::User(update) => update,
            other => panic!("expected user frame, got {other:?}"),
        }
    }

    /// Live order fact as `redis_listener::handle_committed` relays it:
    /// envelope numbers, numeric side, scaled quantities.
    pub(crate) fn live_order_fact(
        revision: u64,
        open_base: &str,
        pending_base: &str,
        closed_reason: Option<&str>,
    ) -> serde_json::Value {
        serde_json::json!({
            "effect_id": format!("{revision}:3"),
            "committed_revision": revision,
            "projection_generation": 1,
            "actionable": true,
            "event_type": "order",
            "user_pubkey": WALLET,
            "order_hash": HASH,
            "market_pubkey": "A9Bxkkc4nah517EjgjwSafwspGnmU9s1Ei5PTo5ZkJd9",
            "orderbook_id": "j749bQAbDsBAiyDs2Tj868heQj1b5KVp98ZrjiZd56a",
            "base_mint": "BaseMint111111111111111111111111111111111111",
            "quote_mint": "QuoteMint11111111111111111111111111111111111",
            "side": 0,
            "maker_amount": "5.500000",
            "taker_amount": "10.00000000",
            "original_base": "10.00000000",
            "confirmed_base": "2.00000000",
            "pending_base": pending_base,
            "open_base": open_base,
            "executable_base": open_base,
            "cancelled_base": "0.00000000",
            "accepted_seq": 44,
            "accepted_at": "2026-09-29T12:00:00+00:00",
            "expiration": 0,
            "tif": "GTC",
            "funding_source": "global",
            "funding_account": "Acct1111111111111111111111111111111111111111",
            "closed_reason": closed_reason,
            "ready": true
        })
    }

    pub(crate) fn live_order(
        revision: u64,
        open_base: &str,
        pending_base: &str,
        closed_reason: Option<&str>,
    ) -> OrderUpdate {
        match user(live_order_fact(
            revision,
            open_base,
            pending_base,
            closed_reason,
        )) {
            UserUpdate::Order(order) => order,
            other => panic!("expected order, got {other:?}"),
        }
    }

    #[test]
    fn live_order_fact_decodes_numeric_side_and_envelope_numbers() {
        let order = live_order(812, "7.00000000", "1.00000000", None);
        assert_eq!(order.commit.committed_revision, 812);
        assert_eq!(order.commit.projection_generation, 1);
        assert!(order.commit.actionable);
        assert_eq!(order.side, Side::Bid);
        assert_eq!(order.accepted_seq, 44);
        assert_eq!(order.open_base, Decimal::new(7, 0));
        assert_eq!(order.funding_source, FundingSource::Global);
        assert_eq!(order.tif, TimeInForce::Gtc);
        assert_eq!(order.price(), Some(Decimal::new(55, 2)));
        assert!(order.ready);
    }

    #[test]
    fn snapshot_decodes_string_revisions_and_empty_markers() {
        let update = user(serde_json::json!({
            "event_type": "snapshot",
            "orders": [{
                "order_hash": HASH,
                "market_pubkey": "A9Bxkkc4nah517EjgjwSafwspGnmU9s1Ei5PTo5ZkJd9",
                "orderbook_id": "j749bQAbDsBAiyDs2Tj868heQj1b5KVp98ZrjiZd56a",
                "side": "ask",
                "maker_amount": "4.00000000",
                "taker_amount": "2.200000",
                "original_base": "4.00000000",
                "confirmed_base": "0.00000000",
                "pending_base": "0.00000000",
                "open_base": "4.00000000",
                "cancelled_base": "0.00000000",
                "closed_reason": "",
                "committed_revision": "812",
                "accepted_seq": "45",
                "price": "0.550000",
                "created_at": 1790685521784_i64,
                "expiration": 0,
                "base_mint": "BaseMint111111111111111111111111111111111111",
                "quote_mint": "QuoteMint11111111111111111111111111111111111",
                "tif": "IOC",
                "funding_source": "conditional"
            }],
            "funding_accounts": [],
            "next_cursor": "",
            "has_more": false,
            "next_funding_cursor": "",
            "funding_has_more": false,
            "committed_revision": "812",
            "projection_generation": "1",
            "notifications": []
        }));
        let UserUpdate::Snapshot(snapshot) = update else {
            panic!("expected snapshot");
        };
        assert_eq!(snapshot.committed_revision, 812);
        assert_eq!(snapshot.next_cursor, None);
        assert_eq!(snapshot.next_funding_cursor, None);
        let order = &snapshot.orders[0];
        assert_eq!(order.side, Side::Ask);
        assert_eq!(order.closed_reason, None);
        assert_eq!(order.accepted_seq, 45);
        assert_eq!(order.tif, TimeInForce::Ioc);
        assert_eq!(order.funding_source, FundingSource::Conditional);
    }

    #[test]
    fn funding_fact_keeps_wide_aggregates_exact() {
        // Mirrors websocket `immutable_funding_fact_uses_exact_atoms_and_transient_permission`.
        let update = user(serde_json::json!({
            "effect_id": "7:2",
            "committed_revision": 7,
            "projection_generation": 1,
            "actionable": false,
            "event_type": "funding",
            "user_pubkey": WALLET,
            "account": "Acct1111111111111111111111111111111111111111",
            "mint": "Cond1111111111111111111111111111111111111111",
            "source": "conditional",
            "market_pubkey": "A9Bxkkc4nah517EjgjwSafwspGnmU9s1Ei5PTo5ZkJd9",
            "raw_observed": "18446744073709.551615",
            "usable_observed": "1.000000",
            "order_reserved": "340282366920938463463374607431768.211455",
            "execution_reserved": "0.000001",
            "signed_remaining": "-0.000001",
            "available": "0.000000",
            "ready": false,
            "accepted_boundary": "boundary",
            "slot": "7",
            "blockhash": null
        }));
        let UserUpdate::Funding(funding) = update else {
            panic!("expected funding");
        };
        assert!(!funding.commit.actionable);
        assert!(!funding.ready);
        assert_eq!(funding.available, Decimal::ZERO);
        assert_eq!(
            funding.raw_observed,
            Some("18446744073709.551615".parse().unwrap())
        );
        assert_eq!(
            funding.order_reserved.as_str(),
            "340282366920938463463374607431768.211455"
        );
        assert_eq!(
            funding
                .signed_remaining
                .and_then(|value| value.to_decimal()),
            Some(Decimal::new(-1, 6))
        );
        assert_eq!(funding.slot, Some(7));
        assert_eq!(funding.blockhash, None);
    }

    #[test]
    fn global_funding_fact_has_no_market() {
        let update = user(serde_json::json!({
            "effect_id": "8:0",
            "committed_revision": 8,
            "projection_generation": 1,
            "actionable": true,
            "event_type": "funding",
            "user_pubkey": WALLET,
            "account": "Glob1111111111111111111111111111111111111111",
            "mint": "7SrxsoXjNR7Y8T3koJCt1yV4FrNUumoAUrJExDt6tQez",
            "source": "global",
            "market_pubkey": null,
            "raw_observed": null,
            "usable_observed": null,
            "order_reserved": "0.000000",
            "execution_reserved": "0.000000",
            "signed_remaining": null,
            "available": "0.000000",
            "ready": false,
            "accepted_boundary": null,
            "slot": null,
            "blockhash": null
        }));
        let UserUpdate::Funding(funding) = update else {
            panic!("expected funding");
        };
        assert_eq!(funding.source, FundingSource::Global);
        assert_eq!(funding.market_pubkey, None);
        assert_eq!(funding.raw_observed, None);
        assert_eq!(funding.slot, None);
    }

    #[test]
    fn closure_and_recovery_facts_decode() {
        let UserUpdate::Closure(closure) = user(serde_json::json!({
            "effect_id": "9:0",
            "committed_revision": 9,
            "projection_generation": 1,
            "actionable": true,
            "event_type": "closure",
            "scope_kind": 5,
            "scope_key": format!("{WALLET}:j749bQAbDsBAiyDs2Tj868heQj1b5KVp98ZrjiZd56a"),
            "accepted_seq": 44,
            "reason": "cancel_all"
        })) else {
            panic!("expected closure");
        };
        assert_eq!(closure.scope(), Some(ClosureScope::WalletBook));
        assert_eq!(closure.accepted_seq, 44);

        let UserUpdate::RecoveryCompleted(recovery) = user(serde_json::json!({
            "effect_id": "10:0",
            "committed_revision": 10,
            "projection_generation": 2,
            "actionable": true,
            "event_type": "recovery_completed",
            "recovered_generation": 2
        })) else {
            panic!("expected recovery");
        };
        assert_eq!(recovery.recovered_generation, 2);
        assert_eq!(recovery.commit.projection_generation, 2);
    }

    #[test]
    fn removed_and_future_user_events_decode_as_unknown() {
        for event_type in [
            "nonce",
            "market_balance_update",
            "global_deposit_update",
            "future_event",
        ] {
            let update = user(serde_json::json!({
                "event_type": event_type,
                "new_nonce": 3,
                "timestamp": "2026-06-19T12:34:56Z"
            }));
            assert_eq!(update, UserUpdate::Unknown, "{event_type}");
        }
    }

    #[test]
    fn user_order_fills_decode_backend_shape() {
        let response: UserOrderFillsResponse = serde_json::from_value(serde_json::json!({
            "orders": [{
                "order_hash": HASH,
                "market_pubkey": "A9Bxkkc4nah517EjgjwSafwspGnmU9s1Ei5PTo5ZkJd9",
                "orderbook_id": "j749bQAbDsBAiyDs2Tj868heQj1b5KVp98ZrjiZd56a",
                "side": "bid",
                "base_mint": "BaseMint111111111111111111111111111111111111",
                "quote_mint": "QuoteMint11111111111111111111111111111111111",
                "original_base": "10.00000000",
                "confirmed_base": "4.00000000",
                "pending_base": "0.00000000",
                "open_base": "0.00000000",
                "cancelled_base": "6.00000000",
                "confirmed_quote": "2.200000",
                "status": "closed",
                "closed_reason": "expired",
                "created_at": 1790685521784_i64,
                "fills": [{
                    "fill_id": "5f7e3c2a-8a39-4f55-a1d6-2b6f0c1b9d11:0:1",
                    "counterparty": "Maker111111111111111111111111111111111111111",
                    "counterparty_order_hash": HASH,
                    "role": "taker",
                    "base_amount": "4.00000000",
                    "quote_amount": "2.200000",
                    "fee_estimate_atoms": "-170141183460469231731687303715884105728",
                    "fee_mint": "QuoteMint11111111111111111111111111111111111",
                    "maker_fee_bps": -5,
                    "taker_fee_bps": 30,
                    "tx_signature": "5VERv8NMvzbJMEkV8xnrLkEaWRtSz9CosKDYjCJjBRnbJLgp8uirBgmQpjKhoR4tjF3ZpRzrFmBV6UjKdiSZkQUW",
                    "filled_at": 1790685522000_i64
                }],
                "fills_has_more": false,
                "fills_next_cursor": null
            }],
            "next_cursor": "eyJhdCI6IjIwMjYtMDktMjlUMTI6MDA6MDBaIiwia2V5IjoiYWIifQ",
            "has_more": true,
            "committed_revision": "812"
        }))
        .unwrap();
        assert_eq!(response.committed_revision, 812);
        let order = &response.orders[0];
        assert_eq!(order.status, OrderStatus::Closed);
        assert_eq!(order.closed_reason.as_deref(), Some("expired"));
        let fill = &order.fills[0];
        assert_eq!(fill.role, Role::Taker);
        assert_eq!(fill.fee_estimate_atoms, i128::MIN);
        assert_eq!(fill.price(), Some(Decimal::new(55, 2)));
    }

    fn rest_user_order_json() -> serde_json::Value {
        serde_json::json!({
            "order_hash": HASH,
            "market_pubkey": "A9Bxkkc4nah517EjgjwSafwspGnmU9s1Ei5PTo5ZkJd9",
            "orderbook_id": "j749bQAbDsBAiyDs2Tj868heQj1b5KVp98ZrjiZd56a",
            "side": "bid",
            "amount_in": "5.500000",
            "amount_out": "10.00000000",
            "price": "0.5500",
            "created_at": 1790685521784_i64,
            "expiration": 0,
            "base_mint": "BaseMint111111111111111111111111111111111111",
            "quote_mint": "QuoteMint11111111111111111111111111111111111",
            "outcome_index": -1,
            "state": {
                "original_base": "100",
                "confirmed_base": "40",
                "pending_base": "20",
                "open_base": "40",
                "cancelled_base": "0",
                "closed_reason": null,
                "committed_revision": "7",
                "accepted_seq": "0"
            },
            "tif": "FOK",
            "source": "global",
            "order_type": "limit"
        })
    }

    #[test]
    fn rest_user_order_decodes_recorded_state() {
        let order: UserOrder = serde_json::from_value(rest_user_order_json()).unwrap();
        let state = order.state.unwrap();
        assert_eq!(state.pending_base, Decimal::from(20));
        assert_eq!(state.committed_revision, 7);
        assert_eq!(order.tif, Some(TimeInForce::Fok));
        assert_eq!(order.source, FundingSource::Global);
        assert_eq!(order.outcome_index, -1);
    }

    #[test]
    fn unknown_wire_values_do_not_fail_the_surrounding_payload() {
        let mut value = rest_user_order_json();
        value["tif"] = "unsupported".into();
        value["source"] = "vault".into();
        let order: UserOrder = serde_json::from_value(value).unwrap();
        assert_eq!(order.tif, None);
        assert_eq!(order.source, FundingSource::Unknown);
        assert_eq!(order.into_limit_order().unwrap().time_in_force, None);

        let mut value = rest_user_order_json();
        value.as_object_mut().unwrap().remove("tif");
        let order: UserOrder = serde_json::from_value(value).unwrap();
        assert_eq!(order.tif, None);

        assert_eq!(
            serde_json::from_value::<Role>(serde_json::json!("arbiter")).unwrap(),
            Role::Unknown
        );
        assert_eq!(
            serde_json::from_value::<OrderStatus>(serde_json::json!("settling")).unwrap(),
            OrderStatus::Unknown
        );
    }
}
