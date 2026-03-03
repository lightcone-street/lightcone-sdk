import { asOrderBookId, asPubkeyStr } from "../../shared";
import type { ConditionalToken } from "../market";
import { OrderBookValidationError, type OrderBookPair } from "./index";
import type { OrderbookResponse } from "./wire";

export function orderBookPairFromWire(
  source: OrderbookResponse,
  tokens: readonly ConditionalToken[]
): OrderBookPair {
  const errors: string[] = [];

  const base = tokens.find((token) => token.pubkey === source.base_token);
  if (!base) {
    errors.push(`Base token not found: ${source.base_token}`);
  }

  const quote = tokens.find((token) => token.pubkey === source.quote_token);
  if (!quote) {
    errors.push(`Quote token not found: ${source.quote_token}`);
  }

  if (errors.length > 0) {
    throw new OrderBookValidationError(source.orderbook_id, errors);
  }

  return {
    id: source.id,
    marketPubkey: asPubkeyStr(source.market_pubkey),
    orderbookId: asOrderBookId(source.orderbook_id),
    base: base as ConditionalToken,
    quote: quote as ConditionalToken,
    outcomeIndex: source.outcome_index ?? (base as ConditionalToken).outcomeIndex,
    tickSize: source.tick_size,
    totalBids: source.total_bids,
    totalAsks: source.total_asks,
    lastTradePrice: source.last_trade_price,
    lastTradeTime: source.last_trade_time ? new Date(source.last_trade_time) : undefined,
    active: source.active,
  };
}
