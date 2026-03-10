import type { PubkeyStr } from "../../shared";

export * from "./client";

export interface MarketResolvedData {
  market_pubkey: PubkeyStr;
  market_slug?: string;
  market_name?: string;
  winning_outcome?: number;
}

export interface OrderFilledData {
  order_hash: string;
  market_pubkey: PubkeyStr;
  side: string;
  price: string;
  filled: string;
  remaining: string;
  market_slug?: string;
  market_name?: string;
  outcome_name?: string;
  outcome_icon_url?: string;
}

export interface MarketData {
  market_pubkey: PubkeyStr;
  market_slug?: string;
  market_name?: string;
}

export type NotificationKind =
  | { notification_type: "market_resolved"; data: MarketResolvedData }
  | { notification_type: "order_filled"; data: OrderFilledData }
  | { notification_type: "new_market"; data: MarketData }
  | { notification_type: "rules_clarified"; data: MarketData }
  | { notification_type: "global" };

export interface Notification {
  id: string;
  notification_type: string;
  data?: MarketResolvedData | OrderFilledData | MarketData;
  title: string;
  message: string;
  expires_at?: string;
  created_at: string;
}

export function isGlobal(notification: Notification): boolean {
  return notification.notification_type === "global";
}

export function marketSlug(notification: Notification): string | undefined {
  const data = notification.data;
  if (!data) return undefined;
  if ("market_slug" in data) return data.market_slug;
  return undefined;
}
