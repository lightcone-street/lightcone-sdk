import type { Trade } from "./index";
import type { TradeResponse, WsTrade } from "./wire";

export function tradeFromResponse(source: TradeResponse): Trade {
  return {
    orderbookId: source.orderbook_id,
    tradeId: String(source.id),
    timestamp: new Date(source.executed_at),
    price: source.price,
    size: source.size,
    side: source.side,
    sequence: 0,
  };
}

export function tradeFromWs(source: WsTrade): Trade {
  return {
    orderbookId: source.orderbook_id,
    tradeId: source.trade_id,
    timestamp: new Date(source.timestamp),
    price: source.price,
    size: source.size,
    side: source.side,
    sequence: source.sequence,
  };
}
