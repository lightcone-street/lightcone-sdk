import type { OrderBookId, OrderbookRules, Side, TradingRules } from "../../shared";
import { SdkError } from "../../error";
import {
  wireArray,
  wireBool,
  wireDecimal,
  wireInt,
  wireObject,
  wireOptDecimal,
  wireOptString,
  wireOrderBookId,
  wireString,
  wireU64,
} from "../../shared/wire_decode";

export interface TradingRulesWire {
  base_size_decimals: number;
  max_price_decimals: number;
  max_price_significant_figures: number;
  integer_prices_always_allowed: boolean;
  price_quantum: string;
  price_quantum_raw: string;
  base_size_quantum: string;
  base_size_quantum_raw: string;
}

export interface OrderbookResponse {
  id: number;
  market_pubkey: string;
  orderbook_id: string;
  base_token: string;
  quote_token: string;
  outcome_index?: number;
  tick_size: number;
  total_bids: number;
  total_asks: number;
  last_trade_price?: string;
  last_trade_time?: string;
  active: boolean;
  created_at: string;
  updated_at: string;
}

export interface OrderbooksResponse {
  orderbooks: OrderbookResponse[];
  total: number;
}

export interface RestBookLevel {
  price: string;
  size: string;
  orders?: number;
}

/**
 * REST depth response. Depth is capped server-side at 20 levels per side.
 */
export interface OrderbookDepthResponse {
  orderbook_id: OrderBookId;
  market_pubkey?: string;
  best_bid?: string;
  best_ask?: string;
  spread?: string;
  /** Deprecated backend alias; never use for order admission. */
  tick_size?: string;
  price_quantum: string;
  /** Parsed exact rules; raw quantum strings are exposed as bigint values. */
  trading_rules: TradingRules;
  bids_truncated: boolean;
  asks_truncated: boolean;
  revision: bigint;
  captured_at_ms: bigint;
  bids: RestBookLevel[];
  asks: RestBookLevel[];
  /** Required display decimals; size is base-token on-chain precision. */
  decimals: OrderbookDepthDecimals;
  /**
   * Whether the engine's committed book is ready (matching live). A book that
   * is not ready may show stale or empty depth.
   */
  ready: boolean;
}

/**
 * Price/size display decimals from the depth endpoint. Distinct from
 * `DecimalsResponse` (the `/decimals` endpoint).
 */
export interface OrderbookDepthDecimals {
  price: number;
  size: number;
}

export interface DecimalsResponse {
  orderbook_id: string;
  base_decimals: number;
  quote_decimals: number;
  price_decimals: number;
  trading_rules: TradingRulesWire;
}

export function tradingRulesFromWire(wire: TradingRulesWire): TradingRules {
  if (
    !wire ||
    !isNonNegativeSafeInteger(wire.base_size_decimals) ||
    !isNonNegativeSafeInteger(wire.max_price_decimals) ||
    !isNonNegativeSafeInteger(wire.max_price_significant_figures) ||
    typeof wire.integer_prices_always_allowed !== "boolean" ||
    typeof wire.price_quantum !== "string" ||
    typeof wire.base_size_quantum !== "string" ||
    typeof wire.price_quantum_raw !== "string" ||
    typeof wire.base_size_quantum_raw !== "string" ||
    !/^\d+$/.test(wire.price_quantum_raw) ||
    !/^\d+$/.test(wire.base_size_quantum_raw)
  ) {
    throw SdkError.serde("trading_rules: invalid trading_rules response");
  }
  return {
    baseSizeDecimals: wire.base_size_decimals,
    maxPriceDecimals: wire.max_price_decimals,
    maxPriceSignificantFigures: wire.max_price_significant_figures,
    integerPricesAlwaysAllowed: wire.integer_prices_always_allowed,
    priceQuantum: wire.price_quantum,
    priceQuantumRaw: BigInt(wire.price_quantum_raw),
    baseSizeQuantum: wire.base_size_quantum,
    baseSizeQuantumRaw: BigInt(wire.base_size_quantum_raw),
  };
}

function restBookLevelFromWire(value: unknown, field: string): RestBookLevel {
  const data = wireObject(value, field);
  const orders = data.orders === undefined || data.orders === null
    ? undefined
    : wireInt(data.orders, `${field}.orders`, 0, 0xffff_ffff);
  return {
    price: wireDecimal(data.price, `${field}.price`),
    size: wireDecimal(data.size, `${field}.size`),
    ...(orders === undefined ? {} : { orders }),
  };
}

/**
 * Decode `GET /api/orderbook/{id}`. A body that does not match the Rust
 * SDK's `OrderbookDepthResponse` throws `SdkError` `Serde`.
 */
export function orderbookDepthFromWire(value: unknown): OrderbookDepthResponse {
  const data = wireObject(value, "orderbook_depth");
  const decimals = wireObject(data.decimals, "orderbook_depth.decimals");
  const optional = (key: string): Record<string, string> => {
    const text = wireOptString(data[key], `orderbook_depth.${key}`);
    return text === undefined ? {} : { [key]: text };
  };
  const optionalDecimal = (key: string): Record<string, string> => {
    const text = wireOptDecimal(data[key], `orderbook_depth.${key}`);
    return text === undefined ? {} : { [key]: text };
  };
  return {
    orderbook_id: wireOrderBookId(data.orderbook_id, "orderbook_depth.orderbook_id"),
    ...optional("market_pubkey"),
    ...optionalDecimal("best_bid"),
    ...optionalDecimal("best_ask"),
    ...optionalDecimal("spread"),
    ...optional("tick_size"),
    price_quantum: wireString(data.price_quantum, "orderbook_depth.price_quantum"),
    trading_rules: tradingRulesFromWire(
      wireObject(data.trading_rules, "orderbook_depth.trading_rules") as unknown as TradingRulesWire,
    ),
    bids_truncated: wireBool(data.bids_truncated, "orderbook_depth.bids_truncated", false),
    asks_truncated: wireBool(data.asks_truncated, "orderbook_depth.asks_truncated", false),
    revision: wireU64(data.revision, "orderbook_depth.revision"),
    captured_at_ms: wireU64(data.captured_at_ms, "orderbook_depth.captured_at_ms"),
    bids: wireArray(data.bids, "orderbook_depth.bids", restBookLevelFromWire),
    asks: wireArray(data.asks, "orderbook_depth.asks", restBookLevelFromWire),
    decimals: {
      price: wireInt(decimals.price, "orderbook_depth.decimals.price", 0, 255),
      size: wireInt(decimals.size, "orderbook_depth.decimals.size", 0, 255),
    },
    ready: wireBool(data.ready, "orderbook_depth.ready", false),
  };
}

export function orderbookRulesFromWire(wire: DecimalsResponse): OrderbookRules {
  if (
    !wire ||
    typeof wire.orderbook_id !== "string" ||
    !isNonNegativeSafeInteger(wire.base_decimals) ||
    !isNonNegativeSafeInteger(wire.quote_decimals) ||
    !isNonNegativeSafeInteger(wire.price_decimals) ||
    !wire.trading_rules
  ) {
    throw SdkError.serde("orderbook_decimals: invalid orderbook decimals response");
  }
  return {
    orderbookId: wire.orderbook_id,
    baseDecimals: wire.base_decimals,
    quoteDecimals: wire.quote_decimals,
    priceDecimals: wire.price_decimals,
    tradingRules: tradingRulesFromWire(wire.trading_rules),
  };
}

function isNonNegativeSafeInteger(value: unknown): value is number {
  return typeof value === "number" && Number.isSafeInteger(value) && value >= 0;
}

export interface WsBookLevel {
  side: Side;
  price: string;
  size: string;
  /**
   * Exact quote-token amount represented by the underlying maker orders.
   * For grouped books this is independent of the displayed bucket price.
   */
  quote_notional: string;
}

/**
 * WS orderbook snapshot frame.
 *
 * The stream is snapshot-only: every data frame carries the full top-20
 * levels per side and replaces the previous book wholesale. `seq` is the
 * engine depth revision and is monotonic only within one subscription
 * generation: drop frames with `seq <=` the last applied one. Resync frames
 * (`resync: true`) carry no `seq` and read as `0n`.
 */
export interface OrderBook {
  orderbook_id: OrderBookId;
  is_snapshot?: boolean;
  seq: bigint;
  /**
   * Whether the engine's committed book is ready (matching live). Absent
   * (`false`) on resync frames.
   */
  ready: boolean;
  resync?: boolean;
  timestamp?: string;
  bids: WsBookLevel[];
  asks: WsBookLevel[];
  bids_truncated?: boolean;
  asks_truncated?: boolean;
  /**
   * Aggregation tags echoed by the backend (omitted = full precision).
   * Always normalized server-side ((5, none) arrives as (5, 1)). Use
   * `aggregationFromFrame(book.n_sig_figs, book.mantissa)` to key
   * per-`(orderbook, aggregation)` book state.
   */
  n_sig_figs?: number;
  mantissa?: number;
}

export interface WsTickerData {
  orderbook_id: OrderBookId;
  best_bid?: string;
  best_ask?: string;
  mid?: string;
  timestamp?: string;
}
