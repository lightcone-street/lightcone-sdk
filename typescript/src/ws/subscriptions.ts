import type { OrderBookId, PubkeyStr, Resolution } from "../shared";

export type SubscribeParams =
  | { type: "book_update"; orderbook_ids: OrderBookId[] }
  | { type: "trades"; orderbook_ids: OrderBookId[] }
  | { type: "user"; wallet_address: PubkeyStr }
  | {
      type: "price_history";
      orderbook_id: OrderBookId;
      resolution: Resolution;
      include_ohlcv?: boolean;
    }
  | { type: "ticker"; orderbook_ids: OrderBookId[] }
  | { type: "market"; market_pubkey: PubkeyStr }
  | { type: "deposit_price"; deposit_asset: string; resolution: Resolution };

export type UnsubscribeParams =
  | { type: "book_update"; orderbook_ids: OrderBookId[] }
  | { type: "trades"; orderbook_ids: OrderBookId[] }
  | { type: "user"; wallet_address: PubkeyStr }
  | { type: "price_history"; orderbook_id: OrderBookId; resolution: Resolution }
  | { type: "ticker"; orderbook_ids: OrderBookId[] }
  | { type: "market"; market_pubkey: PubkeyStr }
  | { type: "deposit_price"; deposit_asset: string; resolution: Resolution };

export interface Subscription {
  toSubscribeParams(): SubscribeParams;
  toUnsubscribeParams(): UnsubscribeParams;
  matchesUnsubscribe(unsubscribe: UnsubscribeParams): boolean;
  subscriptionKey(): string;
}

export function subscriptionKey(params: SubscribeParams): string {
  switch (params.type) {
    case "book_update":
      return `book:${idsKey(params.orderbook_ids)}`;
    case "trades":
      return `trades:${idsKey(params.orderbook_ids)}`;
    case "user":
      return `user:${params.wallet_address}`;
    case "price_history":
      return `price_history:${params.orderbook_id}:${params.resolution}`;
    case "ticker":
      return `ticker:${idsKey(params.orderbook_ids)}`;
    case "market":
      return `market:${params.market_pubkey}`;
    case "deposit_price":
      return `deposit_price:${params.deposit_asset}:${params.resolution}`;
  }
}

export function unsubscribeMatches(
  subscribe: SubscribeParams,
  unsubscribe: UnsubscribeParams
): boolean {
  if (subscribe.type !== unsubscribe.type) {
    return false;
  }

  switch (subscribe.type) {
    case "book_update":
    case "trades":
    case "ticker":
      return "orderbook_ids" in unsubscribe
        ? idsKey(subscribe.orderbook_ids) === idsKey((unsubscribe as { orderbook_ids: OrderBookId[] }).orderbook_ids)
        : false;
    case "user":
      return "wallet_address" in unsubscribe
        ? subscribe.wallet_address === (unsubscribe as { wallet_address: PubkeyStr }).wallet_address
        : false;
    case "price_history":
      return "orderbook_id" in unsubscribe && "resolution" in unsubscribe
        ? subscribe.orderbook_id === (unsubscribe as { orderbook_id: OrderBookId }).orderbook_id &&
          subscribe.resolution === (unsubscribe as { resolution: Resolution }).resolution
        : false;
    case "market":
      return "market_pubkey" in unsubscribe
        ? subscribe.market_pubkey === (unsubscribe as { market_pubkey: PubkeyStr }).market_pubkey
        : false;
    case "deposit_price":
      return "deposit_asset" in unsubscribe && "resolution" in unsubscribe
        ? subscribe.deposit_asset === (unsubscribe as { deposit_asset: string }).deposit_asset &&
          subscribe.resolution === (unsubscribe as { resolution: Resolution }).resolution
        : false;
  }
}

function idsKey(ids: readonly string[]): string {
  return [...ids].sort().join(",");
}
