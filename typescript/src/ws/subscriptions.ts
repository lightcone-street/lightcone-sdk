import {
  aggregationKeySuffix,
  aggregationsEqual,
  isFullPrecision,
} from "../domain/orderbook/aggregation";
import type { OrderBookId, PubkeyStr, Resolution } from "../shared";

/**
 * Book subscriptions optionally carry a Hyperliquid-style aggregation
 * (`nSigFigs`/`mantissa`, wire spelling — omit both for full precision; the
 * backend rejects unknown/snake_case params). Each `(orderbook, aggregation)`
 * pair is a distinct subscription: one connection may hold multiple
 * aggregation views of the same orderbook, and unsubscribe must repeat the
 * same (normalized) aggregation to match.
 */
export type SubscribeParams =
  | {
      type: "book_update";
      orderbook_ids: OrderBookId[];
      nSigFigs?: number;
      mantissa?: number;
    }
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
  | { type: "deposit_price"; deposit_asset: string; resolution: Resolution }
  | { type: "deposit_asset_price"; deposit_asset: string };

export type UnsubscribeParams =
  | {
      type: "book_update";
      orderbook_ids: OrderBookId[];
      nSigFigs?: number;
      mantissa?: number;
    }
  | { type: "trades"; orderbook_ids: OrderBookId[] }
  | { type: "user"; wallet_address: PubkeyStr }
  | { type: "price_history"; orderbook_id: OrderBookId; resolution: Resolution }
  | { type: "ticker"; orderbook_ids: OrderBookId[] }
  | { type: "market"; market_pubkey: PubkeyStr }
  | { type: "deposit_price"; deposit_asset: string; resolution: Resolution }
  | { type: "deposit_asset_price"; deposit_asset: string };

export interface Subscription {
  toSubscribeParams(): SubscribeParams;
  toUnsubscribeParams(): UnsubscribeParams;
  matchesUnsubscribe(unsubscribe: UnsubscribeParams): boolean;
  subscriptionKey(): string;
}

export function subscriptionKey(params: SubscribeParams): string {
  switch (params.type) {
    case "book_update": {
      const aggregation = { nSigFigs: params.nSigFigs, mantissa: params.mantissa };
      // Full precision keeps the pre-aggregation key shape so existing
      // consumers' tracked subscriptions stay stable.
      return isFullPrecision(aggregation)
        ? `book:${idsKey(params.orderbook_ids)}`
        : `book:${idsKey(params.orderbook_ids)}:${aggregationKeySuffix(aggregation)}`;
    }
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
    case "deposit_asset_price":
      return `deposit_asset_price:${params.deposit_asset}`;
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
    case "book_update": {
      const unsubscribeBooks = unsubscribe as {
        orderbook_ids: OrderBookId[];
        nSigFigs?: number;
        mantissa?: number;
      };
      return (
        idsKey(subscribe.orderbook_ids) === idsKey(unsubscribeBooks.orderbook_ids) &&
        aggregationsEqual(
          { nSigFigs: subscribe.nSigFigs, mantissa: subscribe.mantissa },
          { nSigFigs: unsubscribeBooks.nSigFigs, mantissa: unsubscribeBooks.mantissa }
        )
      );
    }
    case "trades":
      return idsKey(subscribe.orderbook_ids) === idsKey((unsubscribe as { orderbook_ids: OrderBookId[] }).orderbook_ids);
    case "ticker":
      return idsKey(subscribe.orderbook_ids) === idsKey((unsubscribe as { orderbook_ids: OrderBookId[] }).orderbook_ids);
    case "user":
      return subscribe.wallet_address === (unsubscribe as { wallet_address: PubkeyStr }).wallet_address;
    case "price_history":
      return (
        subscribe.orderbook_id === (unsubscribe as { orderbook_id: OrderBookId }).orderbook_id &&
        subscribe.resolution === (unsubscribe as { resolution: Resolution }).resolution
      );
    case "market":
      return subscribe.market_pubkey === (unsubscribe as { market_pubkey: PubkeyStr }).market_pubkey;
    case "deposit_price":
      return (
        subscribe.deposit_asset === (unsubscribe as { deposit_asset: string }).deposit_asset &&
        subscribe.resolution === (unsubscribe as { resolution: Resolution }).resolution
      );
    case "deposit_asset_price":
      return subscribe.deposit_asset === (unsubscribe as { deposit_asset: string }).deposit_asset;
  }
}

function idsKey(ids: readonly string[]): string {
  return [...ids].sort().join(",");
}
