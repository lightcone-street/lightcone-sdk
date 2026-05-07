import type { PubkeyStr } from "../../shared";
import type { MarketResolutionResponse } from "../market";

export * from "./client";

export interface MarketResolvedData {
  market_pubkey: PubkeyStr;
  market_slug?: string;
  market_name?: string;
  resolution?: MarketResolutionResponse;
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
  outcome_icon_url_low?: string;
  outcome_icon_url_medium?: string;
  outcome_icon_url_high?: string;
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

interface NotificationBase {
  id: string;
  title: string;
  message: string;
  expires_at?: string;
  created_at: string;
}

export type Notification = NotificationBase & NotificationKind;

export function isGlobal(notification: Notification): boolean {
  return notification.notification_type === "global";
}

export function marketSlug(notification: Notification): string | undefined {
  if (notification.notification_type === "global") {
    return undefined;
  }
  return notification.data.market_slug;
}
