import type { OrderBookId, OrderbookRules, Side, TradingRules } from "../../shared";

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
    throw new Error("invalid trading_rules response");
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

export function orderbookRulesFromWire(wire: DecimalsResponse): OrderbookRules {
  if (
    !wire ||
    typeof wire.orderbook_id !== "string" ||
    !isNonNegativeSafeInteger(wire.base_decimals) ||
    !isNonNegativeSafeInteger(wire.quote_decimals) ||
    !isNonNegativeSafeInteger(wire.price_decimals) ||
    !wire.trading_rules
  ) {
    throw new Error("invalid orderbook decimals response");
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
 * engine depth revision and is monotonic only within one subscription generation.
 */
export interface OrderBook {
  orderbook_id: OrderBookId;
  is_snapshot?: boolean;
  seq: bigint;
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
