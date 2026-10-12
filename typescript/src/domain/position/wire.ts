/**
 * Wire types for committed custody funding (REST positions and the WS `user`
 * channel).
 *
 * The backend reads balances from the committed trading engine: every custody
 * token account a wallet can trade from is a *funding account* (a global
 * deposit account, or a conditional-token account of one market position).
 * Amount fields are decimal strings in the account mint's own units unless
 * the field name ends in `_atoms`.
 */

import { type DecimalText, wireDecimalText, wireOptDecimalText } from "../../shared/decimal_text";
import type { CommitInfo, FundingSource, PubkeyStr } from "../../shared/types";
import {
  wireArray,
  wireBool,
  wireCommitInfo,
  wireDecimal,
  wireEnumOrUnknown,
  wireFundingSource,
  wireNonEmptyString,
  wireObject,
  wireOptDecimal,
  wireOptI128,
  wireOptString,
  wireOptU64,
  wirePubkey,
  wireStringOr,
  wireU64,
} from "../../shared/wire_decode";

/** Evidence state of the last custody observation for a funding account. */
export enum ObservationState {
  /** The token account exists and `raw_observed` holds its balance. */
  Present = "present",
  /** The token account was observed not to exist. */
  Absent = "absent",
  /** The account exists but is not a valid token account for this mint. */
  Invalid = "invalid",
  /** No coherent observation has been accepted yet. */
  Unobserved = "unobserved",
  /** A state this SDK version does not know. */
  Unknown = "unknown",
}

const OBSERVATION_STATES = [
  ObservationState.Present,
  ObservationState.Absent,
  ObservationState.Invalid,
  ObservationState.Unobserved,
] as const;

/**
 * One recorded funding account (REST positions, REST user orders, and the WS
 * `user` snapshot).
 *
 * These are the committed quantities at the response's database snapshot,
 * not live spending capacity: the engine alone decides whether an order can be
 * funded.
 */
export interface FundingAccount {
  /** Custody token account. */
  account: PubkeyStr;
  /** Token mint held by `account` (deposit mint or conditional mint). */
  mint: PubkeyStr;
  source: FundingSource;
  /** Market of a conditional account; `undefined` for global accounts (the backend sends `""`). */
  market_pubkey?: PubkeyStr;
  /** Deposit mint that collateralizes `mint`. */
  deposit_mint: PubkeyStr;
  /** Last observed token balance in `mint` units; `undefined` when unobserved. */
  raw_observed?: string;
  /** Amount reserved by open orders, in `mint` units (u128 aggregate). */
  order_reserved: DecimalText;
  /** Amount reserved by in-flight executions, in `mint` units (u128 aggregate). */
  execution_reserved: DecimalText;
  /**
   * Recorded balance minus reservations, in raw signed atoms of `mint`. Can be
   * negative and is not spendable capacity; `undefined` when unobserved.
   */
  signed_remaining_atoms?: bigint;
  /** Recorded observation boundary (opaque). */
  accepted_boundary: string;
  /** Solana slot of the last accepted observation. */
  observed_slot?: bigint;
  /** Blockhash of the last accepted observation (empty when unobserved). */
  observed_blockhash: string;
  observation_state: ObservationState;
}

/**
 * Live funding-account update on the WS `user` channel (`event_type: "funding"`).
 *
 * Unlike {@link FundingAccount} this carries the engine's transient readiness
 * view: `available` and `ready` describe spending capacity at publication time
 * and are forced to zero/`false` when the fact is not actionable.
 */
export interface FundingUpdate {
  commit: CommitInfo;
  user_pubkey: PubkeyStr;
  account: PubkeyStr;
  mint: PubkeyStr;
  source: FundingSource;
  /** Market of a conditional account; `undefined` for global accounts. */
  market_pubkey?: PubkeyStr;
  /** Last observed token balance in `mint` units. */
  raw_observed?: string;
  /** Observed balance usable for trading, in `mint` units. */
  usable_observed?: string;
  /** Amount reserved by open orders, in `mint` units. */
  order_reserved: DecimalText;
  /** Amount reserved by in-flight executions, in `mint` units. */
  execution_reserved: DecimalText;
  /** Usable balance minus reservations, in `mint` units; may be negative. */
  signed_remaining?: DecimalText;
  /** Spendable amount for new orders, in `mint` units. */
  available: string;
  /** Whether the engine will fund new orders from this account. */
  ready: boolean;
  accepted_boundary?: string;
  /** Solana slot of the observation. */
  slot?: bigint;
  blockhash?: string;
}

/**
 * Response for all four positions routes: `GET /api/users/positions`,
 * `/api/users/{user}/positions`, `/api/users/markets/{market}/positions` and
 * `/api/users/{user}/markets/{market}/positions`.
 *
 * Market-scoped routes keep global accounts and the conditional accounts of
 * that market. Pages are filtered after a bounded scan, so a page can be empty
 * while `has_more` is true.
 */
export interface PositionsResponse {
  owner: PubkeyStr;
  /** Market filter of a market-scoped route; `undefined` for wallet-wide routes. */
  market_pubkey?: PubkeyStr;
  funding_accounts: FundingAccount[];
  /** Database revision of this page; later pages may be newer. */
  committed_revision: bigint;
  projection_generation: bigint;
  has_more: boolean;
  /**
   * Funding-account cursor for the next page (authenticated routes only; the
   * public `{user}` routes reject every query string).
   */
  next_cursor?: string;
}

/** Market-scoped positions share the wallet-wide response shape. */
export type MarketPositionsResponse = PositionsResponse;

function optPubkey(value: unknown, field: string): PubkeyStr | undefined {
  const text = wireNonEmptyString(value, field);
  return text === undefined ? undefined : (text as PubkeyStr);
}

/** Decode one funding account. */
export function fundingAccountFromWire(value: unknown, field = "funding_account"): FundingAccount {
  const data = wireObject(value, field);
  return {
    account: wirePubkey(data.account, `${field}.account`),
    mint: wirePubkey(data.mint, `${field}.mint`),
    source: wireFundingSource(data.source, `${field}.source`),
    market_pubkey: optPubkey(data.market_pubkey, `${field}.market_pubkey`),
    deposit_mint: wirePubkey(data.deposit_mint, `${field}.deposit_mint`),
    raw_observed: wireOptDecimal(data.raw_observed, `${field}.raw_observed`),
    order_reserved: wireDecimalText(data.order_reserved, `${field}.order_reserved`),
    execution_reserved: wireDecimalText(data.execution_reserved, `${field}.execution_reserved`),
    signed_remaining_atoms: wireOptI128(
      data.signed_remaining_atoms,
      `${field}.signed_remaining_atoms`,
    ),
    accepted_boundary: wireStringOr(data.accepted_boundary, `${field}.accepted_boundary`, ""),
    observed_slot: wireOptU64(data.observed_slot, `${field}.observed_slot`),
    observed_blockhash: wireStringOr(data.observed_blockhash, `${field}.observed_blockhash`, ""),
    observation_state: wireEnumOrUnknown(
      data.observation_state,
      `${field}.observation_state`,
      OBSERVATION_STATES,
      ObservationState.Unknown,
    ),
  };
}

/** Decode a live `funding` fact (the `event_type` tag is ignored). */
export function fundingUpdateFromWire(value: unknown, field = "funding"): FundingUpdate {
  const data = wireObject(value, field);
  return {
    commit: wireCommitInfo(data, field),
    user_pubkey: wirePubkey(data.user_pubkey, `${field}.user_pubkey`),
    account: wirePubkey(data.account, `${field}.account`),
    mint: wirePubkey(data.mint, `${field}.mint`),
    source: wireFundingSource(data.source, `${field}.source`),
    market_pubkey: optPubkey(data.market_pubkey, `${field}.market_pubkey`),
    raw_observed: wireOptDecimal(data.raw_observed, `${field}.raw_observed`),
    usable_observed: wireOptDecimal(data.usable_observed, `${field}.usable_observed`),
    order_reserved: wireDecimalText(data.order_reserved, `${field}.order_reserved`),
    execution_reserved: wireDecimalText(data.execution_reserved, `${field}.execution_reserved`),
    signed_remaining: wireOptDecimalText(data.signed_remaining, `${field}.signed_remaining`),
    available: wireDecimal(data.available, `${field}.available`),
    ready: wireBool(data.ready, `${field}.ready`),
    accepted_boundary: wireOptString(data.accepted_boundary, `${field}.accepted_boundary`),
    slot: wireOptU64(data.slot, `${field}.slot`),
    blockhash: wireOptString(data.blockhash, `${field}.blockhash`),
  };
}

/** Decode a positions page (all four positions routes). */
export function positionsResponseFromWire(value: unknown): PositionsResponse {
  const data = wireObject(value, "positions");
  const marketPubkey = wireOptString(data.market_pubkey, "positions.market_pubkey");
  return {
    owner: wirePubkey(data.owner, "positions.owner"),
    market_pubkey: marketPubkey === undefined ? undefined : (marketPubkey as PubkeyStr),
    funding_accounts: wireArray(data.funding_accounts, "positions.funding_accounts", fundingAccountFromWire),
    committed_revision: wireU64(data.committed_revision, "positions.committed_revision"),
    projection_generation: wireU64(data.projection_generation, "positions.projection_generation"),
    has_more: wireBool(data.has_more, "positions.has_more"),
    next_cursor: wireOptString(data.next_cursor, "positions.next_cursor"),
  };
}
