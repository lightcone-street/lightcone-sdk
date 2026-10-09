import { SdkError } from "../../error";
import { isDecimalText } from "../../shared/wire_decode";
import type { Trade } from "./index";
import { wsTradeId, wsTradePrice, type TradeResponse, type WsTrade } from "./wire";

const MAX_DATE_MS = 8_640_000_000_000_000n;

/** Fails on a price, size, or `executed_at` that is not a valid value. */
export function tradeFromResponse(source: TradeResponse): Trade {
  const invalid = (field: string, value: unknown) =>
    SdkError.validation(`trade ${source.trade_id}: invalid ${field} ${String(value)}`);
  if (source.executed_at > MAX_DATE_MS || source.executed_at < -MAX_DATE_MS) {
    throw invalid("executed_at", source.executed_at);
  }
  if (!isDecimalText(source.price)) throw invalid("price", JSON.stringify(source.price));
  if (!isDecimalText(source.size)) throw invalid("size", JSON.stringify(source.size));
  return {
    orderbookId: source.orderbook_id,
    tradeId: source.trade_id,
    cursorId: source.id,
    timestamp: new Date(Number(source.executed_at)),
    price: source.price,
    size: source.size,
    side: source.side,
  };
}

/** Fails when the fill has no base amount to price. */
export function tradeFromWs(source: WsTrade): Trade {
  const tradeId = wsTradeId(source);
  const price = wsTradePrice(source);
  if (price === undefined) {
    throw SdkError.validation(`trade ${tradeId}: zero base amount has no price`);
  }
  return {
    orderbookId: source.orderbook_id,
    tradeId,
    cursorId: undefined,
    timestamp: source.executed_at,
    price,
    size: source.base_amount,
    side: source.taker_side,
  };
}
