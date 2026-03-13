import type {
  OrderBookId,
  OrderUpdateType,
  PubkeyStr,
  Side,
  TimeInForce,
  TriggerResultStatus,
  TriggerStatus,
  TriggerType,
  TriggerUpdateType,
} from "../../shared";
import type { Notification } from "../notification";
import type { OrderStatus } from "./index";

export interface ConditionalBalance {
  outcome_index: number;
  conditional_token: PubkeyStr;
  idle: string;
  on_book: string;
}

export interface UserSnapshotBalance {
  market_pubkey: PubkeyStr;
  orderbook_id: OrderBookId;
  outcomes: ConditionalBalance[];
}

export interface GlobalDepositBalance {
  mint: PubkeyStr;
  balance: string;
}

export interface UserOrderUpdateBalance {
  outcomes: ConditionalBalance[];
}

export interface WsOrder {
  order_hash: string;
  price: string;
  is_maker: boolean;
  remaining: string;
  filled: string;
  fill_amount: string;
  side: Side;
  created_at: number;
  base_mint: PubkeyStr;
  quote_mint: PubkeyStr;
  outcome_index: number;
  status: OrderStatus;
  balance?: UserOrderUpdateBalance;
}

export interface OrderUpdate {
  market_pubkey: PubkeyStr;
  orderbook_id: OrderBookId;
  timestamp: string;
  tx_signature?: string;
  type?: OrderUpdateType;
  order: WsOrder;
}

export interface UserSnapshotOrderCommon {
  order_hash: string;
  market_pubkey: PubkeyStr;
  orderbook_id: OrderBookId;
  side: Side;
  amount_in: string;
  amount_out: string;
  remaining: string;
  filled: string;
  price: string;
  created_at: number;
  expiration: number;
  base_mint: PubkeyStr;
  quote_mint: PubkeyStr;
  outcome_index: number;
  status: OrderStatus;
}

export type UserSnapshotOrder =
  | ({ order_type: "limit"; tx_signature?: string } & UserSnapshotOrderCommon)
  | ({
      order_type: "trigger";
      trigger_order_id: string;
      trigger_price: string;
      trigger_type: TriggerType;
      time_in_force?: TimeInForce;
    } & UserSnapshotOrderCommon);

export interface UserSnapshot {
  orders: UserSnapshotOrder[];
  balances: Record<string, UserSnapshotBalance>;
  global_deposits: GlobalDepositBalance[];
  notifications: Notification[];
}

export interface TriggerOrderUpdate {
  trigger_order_id: string;
  user_pubkey?: PubkeyStr;
  market_pubkey: PubkeyStr;
  orderbook_id: OrderBookId;
  trigger_price: string;
  trigger_above: boolean;
  status: TriggerStatus;
  type?: TriggerUpdateType;
  order_hash: string;
  side: Side;
  result_status?: TriggerResultStatus;
  result_filled?: string;
  result_remaining?: string;
  timestamp: string;
}

export type OrderEvent =
  | ({ order_type: "limit" } & OrderUpdate)
  | ({ order_type: "trigger" } & TriggerOrderUpdate);

export interface NotificationUpdate {
  notification: Notification;
}

export interface UserBalanceUpdate {
  market_pubkey: PubkeyStr;
  orderbook_id: OrderBookId;
  balance: { outcomes: ConditionalBalance[] };
  timestamp: string;
}

export type UserUpdate =
  | ({ event_type: "snapshot" } & UserSnapshot)
  | ({ event_type: "order" } & OrderEvent)
  | ({ event_type: "balance_update" } & UserBalanceUpdate)
  | ({ event_type: "notification" } & NotificationUpdate);

export type AuthUpdate =
  | { status: "authenticated"; wallet: PubkeyStr }
  | { status: "anonymous" }
  | { status: "failed"; code: string; message?: string };
