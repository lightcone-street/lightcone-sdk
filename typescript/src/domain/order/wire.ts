/**
 * Wire types for committed order state (REST) and the WS `user` channel.
 *
 * Quantities named `*_base` are decimal strings in base-token units. Order
 * amounts (`amount_in`/`amount_out`, `maker_amount`/`taker_amount`) are in the
 * units of the token each side gives or receives: a bid gives quote and
 * receives base, an ask gives base and receives quote. Revisions and sequence
 * numbers are exact integers (`bigint`) that arrive as JSON strings on REST
 * and in WS snapshots but as JSON numbers in live WS facts; both are accepted.
 */

import { quotePerBase } from "../../shared/price";
import {
  Side,
  TimeInForce,
  type CommitInfo,
  type FundingSource,
  type OrderBookId,
  type PubkeyStr,
} from "../../shared/types";
import {
  wireArray,
  wireBool,
  wireCommitInfo,
  wireDecimal,
  wireEnum,
  wireEnumOrUnknown,
  wireFundingSource,
  wireI128,
  wireI64,
  wireInt,
  wireIntOr,
  wireKnownOrUndefined,
  wireNonEmptyString,
  wireObject,
  wireOptString,
  wireOrderBookId,
  wirePubkey,
  wireRfc3339,
  wireSide,
  wireString,
  wireStringOr,
  wireTimestampMs,
  wireU64,
  type WireRecord,
} from "../../shared/wire_decode";
import type { Notification } from "../notification";
import { fundingAccountFromWire, fundingUpdateFromWire, type FundingAccount, type FundingUpdate } from "../position/wire";
import { wsTradeFromWire, type WsTrade } from "../trade/wire";
import { OrderStatus } from "./model";

const I16 = [-32_768, 32_767] as const;
const TIME_IN_FORCE = [TimeInForce.Gtc, TimeInForce.Ioc, TimeInForce.Fok] as const;
const ORDER_STATUSES = [
  OrderStatus.Open,
  OrderStatus.Pending,
  OrderStatus.Filled,
  OrderStatus.Closed,
] as const;

function wireI64Or(value: unknown, field: string, fallback: bigint): bigint {
  return value === undefined || value === null ? fallback : wireI64(value, field);
}

// ─── Committed order state (REST) ────────────────────────────────────────────

/**
 * Current committed cumulative state of an order, returned by order
 * submission. All `*_base` quantities are in base-token units.
 *
 * `original_base = confirmed_base + pending_base + open_base + cancelled_base`.
 */
export interface OrderState {
  /** Base size of the signed order. */
  original_base: string;
  /** Base filled by authenticated (on-chain confirmed) executions. */
  confirmed_base: string;
  /** Base matched but awaiting on-chain confirmation. */
  pending_base: string;
  /** Base resting on the book. */
  open_base: string;
  /** Base of `open_base` the engine will currently match. */
  executable_base: string;
  /** Base cancelled: explicitly, by expiry, IOC/FOK remainder, or closure. */
  cancelled_base: string;
  /** Whether the order is live for matching at this revision. */
  ready: boolean;
  /** Why the order stopped resting, when it did. */
  closed_reason?: string;
  committed_revision: bigint;
  /** Engine acceptance sequence (orders the wallet's accepted orders). */
  accepted_seq: bigint;
}

/**
 * Order quantities recorded at one database snapshot (REST user orders).
 * Unlike {@link OrderState} it makes no claim about live executability.
 */
export interface RecordedOrderState {
  /** Base size of the signed order, in base-token units. */
  original_base: string;
  /** Base filled by confirmed executions, in base-token units. */
  confirmed_base: string;
  /** Base matched but awaiting on-chain confirmation, in base-token units. */
  pending_base: string;
  /** Base still resting on the book, in base-token units. */
  open_base: string;
  /** Base cancelled (explicitly, by expiry, or by closure), in base-token units. */
  cancelled_base: string;
  closed_reason?: string;
  /** Committed database revision of this snapshot. */
  committed_revision: bigint;
  /** Engine acceptance sequence; closures cut off by it. */
  accepted_seq: bigint;
}

/** Progress of the executions the engine selected when it accepted an order. */
export enum InitialCohortState {
  /** The order did not cross the book at acceptance. */
  NoInitialExecutions = "no_initial_executions",
  /** Some initial executions still await on-chain confirmation. */
  Pending = "pending",
  /** Every initial execution is confirmed or failed. */
  Complete = "complete",
  /** A state this SDK version does not know. */
  Unknown = "unknown",
}

/** The fixed set of executions selected at acceptance. All `*_base` fields are in base-token units. */
export interface InitialCohort {
  state: InitialCohortState;
  /** Base selected for immediate execution. */
  selected_base: string;
  /** Base of the selection known to be confirmed on-chain. */
  known_confirmed_base: string;
  /** Base of the selection whose settlement failed and returned unfilled. */
  known_failed_unfilled_base: string;
  /** Base of the selection not yet resolved. */
  unresolved_base: string;
}

// ─── REST user orders ────────────────────────────────────────────────────────

/** One open or pending order from `GET /api/users/orders`. */
export interface UserOrder {
  order_hash: string;
  market_pubkey: PubkeyStr;
  orderbook_id: OrderBookId;
  side: Side;
  /** Amount the maker gives: quote units for bids, base units for asks. */
  amount_in: string;
  /** Amount the maker receives: base units for bids, quote units for asks. */
  amount_out: string;
  /** Implied limit price, quote units per base unit. */
  price: string;
  created_at: Date;
  /** Unix seconds; `0` means no expiration. */
  expiration: bigint;
  base_mint: PubkeyStr;
  quote_mint: PubkeyStr;
  /** Outcome index of the base token; the committed backend reports `-1`. */
  outcome_index: number;
  /** Recorded quantities; `undefined` if the engine omitted them. */
  state?: RecordedOrderState;
  /**
   * `undefined` when the backend reports a policy this SDK version does not
   * know (it sends `"unsupported"`).
   */
  tif?: TimeInForce;
  /** Custody source that funds the order. */
  source: FundingSource;
  /** Always `"limit"`. */
  order_type: string;
}

// ─── REST user order fills ───────────────────────────────────────────────────

/** Whether the order was the maker or taker of a fill. */
export enum Role {
  Maker = "maker",
  Taker = "taker",
  /** A role this SDK version does not know. */
  Unknown = "unknown",
}

/** A single authenticated fill of an order. */
export interface OrderFillEvent {
  /**
   * `"<execution_id>:<leg_index>:<projection_generation>"`, the same id as
   * REST `trade_id`.
   */
  fill_id: string;
  counterparty: PubkeyStr;
  counterparty_order_hash: string;
  /** This order's role in the fill. */
  role: Role;
  /** Filled size in base-token units. */
  base_amount: string;
  /** Filled notional in quote-token units. */
  quote_amount: string;
  /**
   * Signed fee estimate in raw atoms of `fee_mint` (negative = rebate). An
   * estimate from the captured rate, never a balance delta.
   */
  fee_estimate_atoms: bigint;
  fee_mint: PubkeyStr;
  /** Maker fee rate captured for the fill, in basis points (negative = rebate). */
  maker_fee_bps: number;
  /** Taker fee rate captured for the fill, in basis points. */
  taker_fee_bps: number;
  tx_signature: string;
  filled_at: Date;
}

/** Fill price in quote units per base unit (`quote_amount / base_amount`). */
export function orderFillEventPrice(fill: OrderFillEvent): string | undefined {
  return quotePerBase(fill.quote_amount, fill.base_amount);
}

/**
 * An order the wallet participated in, with its first page of fills. All
 * `*_base` quantities are in base-token units.
 */
export interface UserOrderFill {
  order_hash: string;
  market_pubkey: PubkeyStr;
  orderbook_id: OrderBookId;
  side: Side;
  base_mint: PubkeyStr;
  quote_mint: PubkeyStr;
  original_base: string;
  confirmed_base: string;
  pending_base: string;
  open_base: string;
  cancelled_base: string;
  /** Quote exchanged by confirmed fills, in quote-token units. */
  confirmed_quote: string;
  status: OrderStatus;
  closed_reason?: string;
  created_at: Date;
  /** Oldest-first page of at most 16 fills. */
  fills: OrderFillEvent[];
  fills_has_more: boolean;
  /** Continue with `order_hash` + `fill_cursor` on the order-fills endpoint. */
  fills_next_cursor?: string;
}

/** Response from `GET /api/users/order-fills` and `GET /api/users/{wallet}/order-fills`. */
export interface UserOrderFillsResponse {
  orders: UserOrderFill[];
  /** Summary cursor; pass back as `cursor` for the next page of orders. */
  next_cursor?: string;
  has_more: boolean;
  /** Database revision of the page. */
  committed_revision: bigint;
}

// ─── WS user channel ─────────────────────────────────────────────────────────

/**
 * One open or pending order in the WS `user` snapshot. Quantities are the
 * same recorded values as REST, flattened into the order.
 */
export interface UserSnapshotOrder {
  order_hash: string;
  market_pubkey: PubkeyStr;
  orderbook_id: OrderBookId;
  side: Side;
  /** Amount the maker gives: quote units for bids, base units for asks. */
  maker_amount: string;
  /** Amount the maker receives: base units for bids, quote units for asks. */
  taker_amount: string;
  original_base: string;
  confirmed_base: string;
  pending_base: string;
  open_base: string;
  cancelled_base: string;
  /** Empty on the wire when the order is not closed. */
  closed_reason?: string;
  committed_revision: bigint;
  accepted_seq: bigint;
  /** Implied limit price, quote units per base unit. */
  price: string;
  created_at: Date;
  /** Unix seconds; `0` means no expiration. */
  expiration: bigint;
  base_mint: PubkeyStr;
  quote_mint: PubkeyStr;
  tif: TimeInForce;
  funding_source: FundingSource;
}

/**
 * WS `user` snapshot, sent once per subscription (`event_type: "snapshot"`).
 *
 * It holds the first bounded page of open/pending orders and funding
 * accounts; fetch further pages over REST with the cursors.
 */
export interface UserSnapshot {
  orders: UserSnapshotOrder[];
  funding_accounts: FundingAccount[];
  /** Order cursor for `GET /api/users/orders` (empty on the wire when absent). */
  next_cursor?: string;
  has_more: boolean;
  /** Funding-account cursor (empty on the wire when absent). */
  next_funding_cursor?: string;
  funding_has_more: boolean;
  /** Database revision of the snapshot (a string here, a number in live facts). */
  committed_revision: bigint;
  projection_generation: bigint;
  notifications: Notification[];
}

/**
 * Live committed order fact on the WS `user` channel (`event_type: "order"`).
 *
 * Each fact is the order's complete cumulative state at `commit`; apply it by
 * replacing the stored order (see `UserOpenLimitOrders.apply`).
 */
export interface OrderUpdate {
  commit: CommitInfo;
  user_pubkey: PubkeyStr;
  order_hash: string;
  market_pubkey: PubkeyStr;
  orderbook_id: OrderBookId;
  base_mint: PubkeyStr;
  quote_mint: PubkeyStr;
  /** Numeric (`0` bid, `1` ask) on the wire. */
  side: Side;
  /** Amount the maker gives: quote units for bids, base units for asks. */
  maker_amount: string;
  /** Amount the maker receives: base units for bids, quote units for asks. */
  taker_amount: string;
  original_base: string;
  confirmed_base: string;
  pending_base: string;
  open_base: string;
  /** Base of `open_base` the engine will currently match. */
  executable_base: string;
  cancelled_base: string;
  accepted_seq: bigint;
  accepted_at: Date;
  /** Unix seconds; `0` means no expiration. */
  expiration: bigint;
  tif: TimeInForce;
  funding_source: FundingSource;
  funding_account: PubkeyStr;
  closed_reason?: string;
  /** Whether the order is live for matching (false when not actionable). */
  ready: boolean;
}

/** Limit price of signed amounts: a bid gives quote for base, an ask gives base for quote. */
export function limitPrice(side: Side, makerAmount: string, takerAmount: string): string | undefined {
  return side === Side.Bid
    ? quotePerBase(makerAmount, takerAmount)
    : quotePerBase(takerAmount, makerAmount);
}

/** Limit price in quote units per base unit, from the signed amounts. */
export function orderUpdatePrice(update: OrderUpdate): string | undefined {
  return limitPrice(update.side, update.maker_amount, update.taker_amount);
}

/** Scope kinds of a committed closure (`ClosureUpdate.scope_kind`). */
export enum ClosureScope {
  /** Every order on the exchange. */
  Exchange = 0,
  /** Orders of one market (`scope_key` = market pubkey). */
  Market = 1,
  /** Orders of one orderbook (`scope_key` = orderbook id). */
  Book = 2,
  /** Orders collateralized by one deposit token (`scope_key` = mint). */
  DepositToken = 3,
  /** Orders of one wallet (`scope_key` = wallet). */
  Wallet = 4,
  /** Orders of one wallet on one book (`scope_key` = `"<wallet>:<book>"`). */
  WalletBook = 5,
  /** One custody account. */
  Account = 6,
  /** One settlement profile. */
  SettlementProfile = 7,
}

/**
 * Committed accepted-order cutoff (`event_type: "closure"`): every order in
 * scope with `accepted_seq <= accepted_seq` is closed. Wallet-scoped closures
 * go to that wallet; other scopes are broadcast to every user subscription.
 * Individual order facts may follow later.
 */
export interface ClosureUpdate {
  commit: CommitInfo;
  /** Raw scope code; see {@link closureScope}. */
  scope_kind: number;
  scope_key: string;
  /** Inclusive acceptance-sequence cutoff. */
  accepted_seq: bigint;
  reason: string;
}

/** Typed scope of a closure, or `undefined` for an unknown code. */
export function closureScope(closure: ClosureUpdate): ClosureScope | undefined {
  const code = closure.scope_kind;
  return Number.isInteger(code) && code >= ClosureScope.Exchange && code <= ClosureScope.SettlementProfile
    ? (code as ClosureScope)
    : undefined;
}

/**
 * The engine finished rebuilding committed state
 * (`event_type: "recovery_completed"`). Cached user state from before the
 * rebuild is stale: resubscribe or refetch.
 */
export interface RecoveryCompleted {
  commit: CommitInfo;
  /** Projection generation the engine rebuilt to (compare with `commit.projection_generation`). */
  recovered_generation: bigint;
}

/** WS notification push event. */
export interface NotificationUpdate {
  notification: Notification;
}

/** WS `user` channel payload, discriminated by `event_type`. */
export type UserUpdate =
  | ({ event_type: "snapshot" } & UserSnapshot)
  | ({ event_type: "order" } & OrderUpdate)
  | ({ event_type: "funding" } & FundingUpdate)
  /** A fill involving the wallet as maker or taker (same payload as the `trades` channel). */
  | ({ event_type: "fill" } & WsTrade)
  | ({ event_type: "closure" } & ClosureUpdate)
  | ({ event_type: "recovery_completed" } & RecoveryCompleted)
  | ({ event_type: "notification" } & NotificationUpdate)
  /**
   * An event type this SDK version does not know (including the retired
   * `nonce`, `market_balance_update`, and `global_deposit_update`); safe to
   * ignore. `raw_event_type` is the wire tag.
   */
  | { event_type: "unknown"; raw_event_type: string };

export type AuthUpdate =
  | { status: "authenticated"; wallet: PubkeyStr }
  | { status: "anonymous"; reason?: string };

// ─── Decoders ────────────────────────────────────────────────────────────────

function closedReason(data: WireRecord, field: string): string | undefined {
  return wireNonEmptyString(data.closed_reason, `${field}.closed_reason`);
}

/** Decode the committed order state returned by submission. */
export function orderStateFromWire(value: unknown, field = "state"): OrderState {
  const data = wireObject(value, field);
  return {
    original_base: wireDecimal(data.original_base, `${field}.original_base`),
    confirmed_base: wireDecimal(data.confirmed_base, `${field}.confirmed_base`),
    pending_base: wireDecimal(data.pending_base, `${field}.pending_base`),
    open_base: wireDecimal(data.open_base, `${field}.open_base`),
    executable_base: wireDecimal(data.executable_base, `${field}.executable_base`),
    cancelled_base: wireDecimal(data.cancelled_base, `${field}.cancelled_base`),
    ready: wireBool(data.ready, `${field}.ready`),
    closed_reason: closedReason(data, field),
    committed_revision: wireU64(data.committed_revision, `${field}.committed_revision`),
    accepted_seq: wireU64(data.accepted_seq, `${field}.accepted_seq`),
  };
}

/** Decode recorded order quantities (REST user orders). */
export function recordedOrderStateFromWire(value: unknown, field = "state"): RecordedOrderState {
  const data = wireObject(value, field);
  return {
    original_base: wireDecimal(data.original_base, `${field}.original_base`),
    confirmed_base: wireDecimal(data.confirmed_base, `${field}.confirmed_base`),
    pending_base: wireDecimal(data.pending_base, `${field}.pending_base`),
    open_base: wireDecimal(data.open_base, `${field}.open_base`),
    cancelled_base: wireDecimal(data.cancelled_base, `${field}.cancelled_base`),
    closed_reason: closedReason(data, field),
    committed_revision: wireU64(data.committed_revision, `${field}.committed_revision`),
    accepted_seq: wireU64(data.accepted_seq, `${field}.accepted_seq`),
  };
}

const COHORT_STATES = [
  InitialCohortState.NoInitialExecutions,
  InitialCohortState.Pending,
  InitialCohortState.Complete,
] as const;

/** Decode the executions selected at acceptance. */
export function initialCohortFromWire(value: unknown, field = "initial_cohort"): InitialCohort {
  const data = wireObject(value, field);
  return {
    state: wireEnumOrUnknown(data.state, `${field}.state`, COHORT_STATES, InitialCohortState.Unknown),
    selected_base: wireDecimal(data.selected_base, `${field}.selected_base`),
    known_confirmed_base: wireDecimal(data.known_confirmed_base, `${field}.known_confirmed_base`),
    known_failed_unfilled_base: wireDecimal(
      data.known_failed_unfilled_base,
      `${field}.known_failed_unfilled_base`,
    ),
    unresolved_base: wireDecimal(data.unresolved_base, `${field}.unresolved_base`),
  };
}

/** Decode one REST user order. */
export function userOrderFromWire(value: unknown, field = "order"): UserOrder {
  const data = wireObject(value, field);
  return {
    order_hash: wireString(data.order_hash, `${field}.order_hash`),
    market_pubkey: wirePubkey(data.market_pubkey, `${field}.market_pubkey`),
    orderbook_id: wireOrderBookId(data.orderbook_id, `${field}.orderbook_id`),
    side: wireSide(data.side, `${field}.side`),
    amount_in: wireDecimal(data.amount_in, `${field}.amount_in`),
    amount_out: wireDecimal(data.amount_out, `${field}.amount_out`),
    price: wireDecimal(data.price, `${field}.price`),
    created_at: wireTimestampMs(data.created_at, `${field}.created_at`),
    expiration: wireI64Or(data.expiration, `${field}.expiration`, 0n),
    base_mint: wirePubkey(data.base_mint, `${field}.base_mint`),
    quote_mint: wirePubkey(data.quote_mint, `${field}.quote_mint`),
    outcome_index: wireIntOr(data.outcome_index, `${field}.outcome_index`, ...I16, -1),
    state:
      data.state === undefined || data.state === null
        ? undefined
        : recordedOrderStateFromWire(data.state, `${field}.state`),
    tif: wireKnownOrUndefined(data.tif, `${field}.tif`, TIME_IN_FORCE),
    source: wireFundingSource(data.source, `${field}.source`),
    order_type: wireStringOr(data.order_type, `${field}.order_type`, ""),
  };
}

const ROLES = [Role.Maker, Role.Taker] as const;

/** Decode one authenticated fill of an order. */
export function orderFillEventFromWire(value: unknown, field = "fill"): OrderFillEvent {
  const data = wireObject(value, field);
  return {
    fill_id: wireString(data.fill_id, `${field}.fill_id`),
    counterparty: wirePubkey(data.counterparty, `${field}.counterparty`),
    counterparty_order_hash: wireString(data.counterparty_order_hash, `${field}.counterparty_order_hash`),
    role: wireEnumOrUnknown(data.role, `${field}.role`, ROLES, Role.Unknown),
    base_amount: wireDecimal(data.base_amount, `${field}.base_amount`),
    quote_amount: wireDecimal(data.quote_amount, `${field}.quote_amount`),
    fee_estimate_atoms: wireI128(data.fee_estimate_atoms, `${field}.fee_estimate_atoms`),
    fee_mint: wirePubkey(data.fee_mint, `${field}.fee_mint`),
    maker_fee_bps: wireInt(data.maker_fee_bps, `${field}.maker_fee_bps`, ...I16),
    taker_fee_bps: wireInt(data.taker_fee_bps, `${field}.taker_fee_bps`, ...I16),
    tx_signature: wireString(data.tx_signature, `${field}.tx_signature`),
    filled_at: wireTimestampMs(data.filled_at, `${field}.filled_at`),
  };
}

/** Decode an order with its first page of fills. */
export function userOrderFillFromWire(value: unknown, field = "order"): UserOrderFill {
  const data = wireObject(value, field);
  return {
    order_hash: wireString(data.order_hash, `${field}.order_hash`),
    market_pubkey: wirePubkey(data.market_pubkey, `${field}.market_pubkey`),
    orderbook_id: wireOrderBookId(data.orderbook_id, `${field}.orderbook_id`),
    side: wireSide(data.side, `${field}.side`),
    base_mint: wirePubkey(data.base_mint, `${field}.base_mint`),
    quote_mint: wirePubkey(data.quote_mint, `${field}.quote_mint`),
    original_base: wireDecimal(data.original_base, `${field}.original_base`),
    confirmed_base: wireDecimal(data.confirmed_base, `${field}.confirmed_base`),
    pending_base: wireDecimal(data.pending_base, `${field}.pending_base`),
    open_base: wireDecimal(data.open_base, `${field}.open_base`),
    cancelled_base: wireDecimal(data.cancelled_base, `${field}.cancelled_base`),
    confirmed_quote: wireDecimal(data.confirmed_quote, `${field}.confirmed_quote`),
    status: wireEnumOrUnknown(data.status, `${field}.status`, ORDER_STATUSES, OrderStatus.Unknown),
    closed_reason: closedReason(data, field),
    created_at: wireTimestampMs(data.created_at, `${field}.created_at`),
    fills: wireArray(data.fills, `${field}.fills`, orderFillEventFromWire),
    fills_has_more: wireBool(data.fills_has_more, `${field}.fills_has_more`, false),
    fills_next_cursor: wireOptString(data.fills_next_cursor, `${field}.fills_next_cursor`),
  };
}

/** Decode `GET /api/users[/{wallet}]/order-fills`. */
export function userOrderFillsResponseFromWire(value: unknown): UserOrderFillsResponse {
  const data = wireObject(value, "order_fills");
  return {
    orders: wireArray(data.orders, "order_fills.orders", userOrderFillFromWire),
    next_cursor: wireOptString(data.next_cursor, "order_fills.next_cursor"),
    has_more: wireBool(data.has_more, "order_fills.has_more"),
    committed_revision: wireU64(data.committed_revision, "order_fills.committed_revision"),
  };
}

/** Decode one order of the WS `user` snapshot. */
export function userSnapshotOrderFromWire(value: unknown, field = "order"): UserSnapshotOrder {
  const data = wireObject(value, field);
  return {
    order_hash: wireString(data.order_hash, `${field}.order_hash`),
    market_pubkey: wirePubkey(data.market_pubkey, `${field}.market_pubkey`),
    orderbook_id: wireOrderBookId(data.orderbook_id, `${field}.orderbook_id`),
    side: wireSide(data.side, `${field}.side`),
    maker_amount: wireDecimal(data.maker_amount, `${field}.maker_amount`),
    taker_amount: wireDecimal(data.taker_amount, `${field}.taker_amount`),
    original_base: wireDecimal(data.original_base, `${field}.original_base`),
    confirmed_base: wireDecimal(data.confirmed_base, `${field}.confirmed_base`),
    pending_base: wireDecimal(data.pending_base, `${field}.pending_base`),
    open_base: wireDecimal(data.open_base, `${field}.open_base`),
    cancelled_base: wireDecimal(data.cancelled_base, `${field}.cancelled_base`),
    closed_reason: closedReason(data, field),
    committed_revision: wireU64(data.committed_revision, `${field}.committed_revision`),
    accepted_seq: wireU64(data.accepted_seq, `${field}.accepted_seq`),
    price: wireDecimal(data.price, `${field}.price`),
    created_at: wireTimestampMs(data.created_at, `${field}.created_at`),
    expiration: wireI64Or(data.expiration, `${field}.expiration`, 0n),
    base_mint: wirePubkey(data.base_mint, `${field}.base_mint`),
    quote_mint: wirePubkey(data.quote_mint, `${field}.quote_mint`),
    tif: wireEnum(data.tif, `${field}.tif`, TIME_IN_FORCE),
    funding_source: wireFundingSource(data.funding_source, `${field}.funding_source`),
  };
}

function userSnapshotFromWire(data: WireRecord, field: string): UserSnapshot {
  return {
    orders: wireArray(data.orders, `${field}.orders`, userSnapshotOrderFromWire),
    funding_accounts: wireArray(data.funding_accounts, `${field}.funding_accounts`, fundingAccountFromWire),
    next_cursor: wireNonEmptyString(data.next_cursor, `${field}.next_cursor`),
    has_more: wireBool(data.has_more, `${field}.has_more`, false),
    next_funding_cursor: wireNonEmptyString(data.next_funding_cursor, `${field}.next_funding_cursor`),
    funding_has_more: wireBool(data.funding_has_more, `${field}.funding_has_more`, false),
    committed_revision: wireU64(data.committed_revision, `${field}.committed_revision`),
    projection_generation: wireU64(data.projection_generation, `${field}.projection_generation`),
    notifications: wireArray(
      data.notifications,
      `${field}.notifications`,
      (item, itemField) => wireObject(item, itemField) as unknown as Notification,
      true,
    ),
  };
}

/** Decode a live `order` fact (the `event_type` tag is ignored). */
export function orderUpdateFromWire(value: unknown, field = "order"): OrderUpdate {
  const data = wireObject(value, field);
  return {
    commit: wireCommitInfo(data, field),
    user_pubkey: wirePubkey(data.user_pubkey, `${field}.user_pubkey`),
    order_hash: wireString(data.order_hash, `${field}.order_hash`),
    market_pubkey: wirePubkey(data.market_pubkey, `${field}.market_pubkey`),
    orderbook_id: wireOrderBookId(data.orderbook_id, `${field}.orderbook_id`),
    base_mint: wirePubkey(data.base_mint, `${field}.base_mint`),
    quote_mint: wirePubkey(data.quote_mint, `${field}.quote_mint`),
    side: wireSide(data.side, `${field}.side`),
    maker_amount: wireDecimal(data.maker_amount, `${field}.maker_amount`),
    taker_amount: wireDecimal(data.taker_amount, `${field}.taker_amount`),
    original_base: wireDecimal(data.original_base, `${field}.original_base`),
    confirmed_base: wireDecimal(data.confirmed_base, `${field}.confirmed_base`),
    pending_base: wireDecimal(data.pending_base, `${field}.pending_base`),
    open_base: wireDecimal(data.open_base, `${field}.open_base`),
    executable_base: wireDecimal(data.executable_base, `${field}.executable_base`),
    cancelled_base: wireDecimal(data.cancelled_base, `${field}.cancelled_base`),
    accepted_seq: wireU64(data.accepted_seq, `${field}.accepted_seq`),
    accepted_at: wireRfc3339(data.accepted_at, `${field}.accepted_at`),
    expiration: wireI64Or(data.expiration, `${field}.expiration`, 0n),
    tif: wireEnum(data.tif, `${field}.tif`, TIME_IN_FORCE),
    funding_source: wireFundingSource(data.funding_source, `${field}.funding_source`),
    funding_account: wirePubkey(data.funding_account, `${field}.funding_account`),
    closed_reason: closedReason(data, field),
    ready: wireBool(data.ready, `${field}.ready`, false),
  };
}

/** Decode a committed closure cutoff. */
export function closureUpdateFromWire(value: unknown, field = "closure"): ClosureUpdate {
  const data = wireObject(value, field);
  return {
    commit: wireCommitInfo(data, field),
    scope_kind: wireInt(data.scope_kind, `${field}.scope_kind`, ...I16),
    scope_key: wireString(data.scope_key, `${field}.scope_key`),
    accepted_seq: wireU64(data.accepted_seq, `${field}.accepted_seq`),
    reason: wireString(data.reason, `${field}.reason`),
  };
}

/** Decode one WS `user` channel payload. Unknown event types decode as `unknown`. */
export function userUpdateFromWire(value: unknown): UserUpdate {
  const data = wireObject(value, "user");
  const eventType = wireString(data.event_type, "user.event_type");
  switch (eventType) {
    case "snapshot":
      return { event_type: "snapshot", ...userSnapshotFromWire(data, "user.snapshot") };
    case "order":
      return { event_type: "order", ...orderUpdateFromWire(data, "user.order") };
    case "funding":
      return { event_type: "funding", ...fundingUpdateFromWire(data, "user.funding") };
    case "fill":
      return { event_type: "fill", ...wsTradeFromWire(data, "user.fill") };
    case "closure":
      return { event_type: "closure", ...closureUpdateFromWire(data, "user.closure") };
    case "recovery_completed":
      return {
        event_type: "recovery_completed",
        commit: wireCommitInfo(data, "user.recovery_completed"),
        recovered_generation: wireU64(
          data.recovered_generation,
          "user.recovery_completed.recovered_generation",
        ),
      };
    case "notification":
      return {
        event_type: "notification",
        notification: wireObject(data.notification, "user.notification.notification") as unknown as Notification,
      };
    default:
      return { event_type: "unknown", raw_event_type: eventType };
  }
}
