import { quotePerBase } from "../../shared/price";
import type { CommitInfo, OrderBookId, PubkeyStr, Side } from "../../shared/types";
import {
  wireArray,
  wireBool,
  wireCommitInfo,
  wireDecimal,
  wireInt,
  wireI64,
  wireObject,
  wireOptDecimal,
  wireOptI64,
  wireOrderBookId,
  wirePubkey,
  wireRfc3339,
  wireSide,
  wireSideText,
  wireString,
  type WireRecord,
} from "../../shared/wire_decode";

// ─── REST wire types ─────────────────────────────────────────────────────────

/** A single trade from the REST API. */
export interface TradeResponse {
  /** Fill-fact row id (i64); pass as `cursor` to page backwards. */
  id: bigint;
  /**
   * `"<execution_id>:<leg_index>:<projection_generation>"`; see
   * {@link wsTradeId} for the matching live id.
   */
  trade_id: string;
  orderbook_id: OrderBookId;
  taker_pubkey: string;
  maker_pubkey: string;
  /** Taker side. */
  side: Side;
  /** Size in base-token units (decimal text, validated on conversion). */
  size: string;
  /** Maker price in quote units per base unit (truncated to price decimals). */
  price: string;
  /** Signed taker fee estimate (negative = rebate), scaled with the quote token's decimals. */
  taker_fee_estimate?: string;
  /** Signed maker fee estimate (negative = rebate), scaled with the quote token's decimals. */
  maker_fee_estimate?: string;
  /** Unix milliseconds. */
  executed_at: bigint;
}

/** REST decimals metadata for trades. */
export interface TradesDecimals {
  price?: number;
  size?: number;
  fee?: number;
}

/** REST response for trades list. */
export interface TradesResponse {
  orderbook_id: OrderBookId;
  trades: TradeResponse[];
  next_cursor?: bigint;
  has_more: boolean;
  decimals?: TradesDecimals;
}

/** REST response for market-level trades (all orderbooks in a market). */
export interface MarketTradesResponse {
  market_pubkey: PubkeyStr;
  trades: TradeResponse[];
  next_cursor?: bigint;
  has_more: boolean;
}

// ─── WS wire types ───────────────────────────────────────────────────────────

/**
 * Committed fill fact on the WS `trades` channel, also delivered to the maker
 * and taker as a `user` channel `fill` event.
 *
 * The payload carries exact base and quote amounts rather than a price; use
 * {@link wsTradePrice}. Amount fields are decimal strings in the named token's
 * units.
 */
export interface WsTrade {
  commit: CommitInfo;
  /**
   * Relay dedup id: `"<evidence_id>:<event_index>:<leg_index>"`. Not the REST
   * `trade_id`; see {@link wsTradeId}.
   */
  fill_id: string;
  execution_id: string;
  evidence_id: string;
  /** Index of the settlement event within the confirming evidence. */
  event_index: number;
  /** Index of this leg within the execution. */
  leg_index: number;
  orderbook_id: OrderBookId;
  market_pubkey: PubkeyStr;
  base_mint: PubkeyStr;
  quote_mint: PubkeyStr;
  /** Filled size in base-token units. */
  base_amount: string;
  /** Filled notional in quote-token units. */
  quote_amount: string;
  maker_order_hash: string;
  taker_order_hash: string;
  maker_wallet: PubkeyStr;
  taker_wallet: PubkeyStr;
  /** Taker side; numeric (`0` bid, `1` ask) on the wire. */
  taker_side: Side;
  /** Signed maker fee estimate in `fee_mint` units (negative = rebate). */
  maker_fee_estimate: string;
  /** Signed taker fee estimate in `fee_mint` units. */
  taker_fee_estimate: string;
  fee_mint: PubkeyStr;
  /** Maker fee rate in basis points (negative = rebate). */
  maker_fee_bps: number;
  /** Taker fee rate in basis points. */
  taker_fee_bps: number;
  executed_at: Date;
}

/** Execution price in quote units per base unit (`quote_amount / base_amount`). */
export function wsTradePrice(trade: WsTrade): string | undefined {
  return quotePerBase(trade.quote_amount, trade.base_amount);
}

/**
 * The REST `trade_id` of this fill
 * (`"<execution_id>:<leg_index>:<projection_generation>"`), for merging live
 * trades with REST history.
 */
export function wsTradeId(trade: WsTrade): string {
  return `${trade.execution_id}:${trade.leg_index}:${trade.commit.projection_generation}`;
}

// ─── Decoders ────────────────────────────────────────────────────────────────

const I16 = [-32_768, 32_767] as const;
const I32 = [-2_147_483_648, 2_147_483_647] as const;
const U8 = [0, 255] as const;

/** Decode one REST trade. */
export function tradeResponseFromWire(value: unknown, field = "trade"): TradeResponse {
  const data = wireObject(value, field);
  return {
    id: wireI64(data.id, `${field}.id`),
    trade_id: wireString(data.trade_id, `${field}.trade_id`),
    orderbook_id: wireOrderBookId(data.orderbook_id, `${field}.orderbook_id`),
    taker_pubkey: wireString(data.taker_pubkey, `${field}.taker_pubkey`),
    maker_pubkey: wireString(data.maker_pubkey, `${field}.maker_pubkey`),
    side: wireSideText(data.side, `${field}.side`),
    size: wireString(data.size, `${field}.size`),
    price: wireString(data.price, `${field}.price`),
    taker_fee_estimate: wireOptDecimal(data.taker_fee_estimate, `${field}.taker_fee_estimate`),
    maker_fee_estimate: wireOptDecimal(data.maker_fee_estimate, `${field}.maker_fee_estimate`),
    executed_at: wireI64(data.executed_at, `${field}.executed_at`),
  };
}

function optSmallInt(data: WireRecord, key: string, field: string): number | undefined {
  const value = data[key];
  return value === undefined || value === null ? undefined : wireInt(value, `${field}.${key}`, ...U8);
}

/** Decode `GET /api/trades`. */
export function tradesResponseFromWire(value: unknown): TradesResponse {
  const data = wireObject(value, "trades");
  const decimals =
    data.decimals === undefined || data.decimals === null
      ? undefined
      : wireObject(data.decimals, "trades.decimals");
  return {
    orderbook_id: wireOrderBookId(data.orderbook_id, "trades.orderbook_id"),
    trades: wireArray(data.trades, "trades.trades", tradeResponseFromWire),
    next_cursor: wireOptI64(data.next_cursor, "trades.next_cursor"),
    has_more: wireBool(data.has_more, "trades.has_more", false),
    decimals:
      decimals === undefined
        ? undefined
        : {
            price: optSmallInt(decimals, "price", "trades.decimals"),
            size: optSmallInt(decimals, "size", "trades.decimals"),
            fee: optSmallInt(decimals, "fee", "trades.decimals"),
          },
  };
}

/** Decode `GET /api/trades/market`. */
export function marketTradesResponseFromWire(value: unknown): MarketTradesResponse {
  const data = wireObject(value, "market_trades");
  return {
    market_pubkey: wirePubkey(data.market_pubkey, "market_trades.market_pubkey"),
    trades: wireArray(data.trades, "market_trades.trades", tradeResponseFromWire),
    next_cursor: wireOptI64(data.next_cursor, "market_trades.next_cursor"),
    has_more: wireBool(data.has_more, "market_trades.has_more", false),
  };
}

/** Decode a committed fill fact (`trades` channel or `user` `fill` event). */
export function wsTradeFromWire(value: unknown, field = "trade"): WsTrade {
  const data = wireObject(value, field);
  return {
    commit: wireCommitInfo(data, field),
    fill_id: wireString(data.fill_id, `${field}.fill_id`),
    execution_id: wireString(data.execution_id, `${field}.execution_id`),
    evidence_id: wireString(data.evidence_id, `${field}.evidence_id`),
    event_index: wireInt(data.event_index, `${field}.event_index`, ...I32),
    leg_index: wireInt(data.leg_index, `${field}.leg_index`, ...I32),
    orderbook_id: wireOrderBookId(data.orderbook_id, `${field}.orderbook_id`),
    market_pubkey: wirePubkey(data.market_pubkey, `${field}.market_pubkey`),
    base_mint: wirePubkey(data.base_mint, `${field}.base_mint`),
    quote_mint: wirePubkey(data.quote_mint, `${field}.quote_mint`),
    base_amount: wireDecimal(data.base_amount, `${field}.base_amount`),
    quote_amount: wireDecimal(data.quote_amount, `${field}.quote_amount`),
    maker_order_hash: wireString(data.maker_order_hash, `${field}.maker_order_hash`),
    taker_order_hash: wireString(data.taker_order_hash, `${field}.taker_order_hash`),
    maker_wallet: wirePubkey(data.maker_wallet, `${field}.maker_wallet`),
    taker_wallet: wirePubkey(data.taker_wallet, `${field}.taker_wallet`),
    taker_side: wireSide(data.taker_side, `${field}.taker_side`),
    maker_fee_estimate: wireDecimal(data.maker_fee_estimate, `${field}.maker_fee_estimate`),
    taker_fee_estimate: wireDecimal(data.taker_fee_estimate, `${field}.taker_fee_estimate`),
    fee_mint: wirePubkey(data.fee_mint, `${field}.fee_mint`),
    maker_fee_bps: wireInt(data.maker_fee_bps, `${field}.maker_fee_bps`, ...I16),
    taker_fee_bps: wireInt(data.taker_fee_bps, `${field}.taker_fee_bps`, ...I16),
    executed_at: wireRfc3339(data.executed_at, `${field}.executed_at`),
  };
}
