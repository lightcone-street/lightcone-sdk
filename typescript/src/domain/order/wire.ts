import { asPubkeyStr, TimeInForce, TriggerType } from "../../shared";
import type {
  OrderBookId,
  OrderUpdateType,
  PubkeyStr,
  Side,
  TriggerResultStatus,
  TriggerStatus,
  TriggerUpdateType,
} from "../../shared";
import type { Notification } from "../notification";
import type { OrderStatus, TriggerOrder } from "./index";

export interface ConditionalBalance {
  outcome_index: number;
  mint: PubkeyStr;
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

export interface GlobalDepositUpdate {
  mint: PubkeyStr;
  balance: string;
  timestamp: string;
}

export interface NonceUpdate {
  user_pubkey: PubkeyStr;
  new_nonce: number;
  timestamp: string;
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
  nonce?: number;
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
  maker_amount?: string;
  taker_amount?: string;
  tif?: TimeInForce;
}

export function triggerUpdateToTriggerOrder(update: TriggerOrderUpdate): TriggerOrder {
  const triggerType = update.trigger_above ? TriggerType.TakeProfit : TriggerType.StopLoss;
  return {
    triggerOrderId: update.trigger_order_id,
    orderHash: update.order_hash,
    marketPubkey: update.market_pubkey,
    orderbookId: update.orderbook_id,
    triggerPrice: update.trigger_price,
    triggerType,
    side: update.side,
    amountIn: update.maker_amount ?? "0",
    amountOut: update.taker_amount ?? "0",
    timeInForce: update.tif ?? TimeInForce.Gtc,
    createdAt: new Date(update.timestamp),
  };
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
  | ({ event_type: "global_deposit_update" } & GlobalDepositUpdate)
  | ({ event_type: "nonce" } & NonceUpdate)
  | ({ event_type: "notification" } & NotificationUpdate);

export type AuthUpdate =
  | { status: "authenticated"; wallet: PubkeyStr }
  | { status: "anonymous"; reason?: string };

// ─── User order fills (REST) ────────────────────────────────────────────────

export type Role = "maker" | "taker";

export type FillStatus = "filled" | "cancelled" | "partially_filled";

export interface OrderFillEvent {
  fill_amount: string;
  tx_signature: string;
  filled_at: number;
}

export interface UserOrderFill {
  order_hash: string;
  market_pubkey: PubkeyStr;
  orderbook_id: OrderBookId;
  side: Side;
  role: Role;
  price: string;
  size: string;
  filled_size: string;
  remaining_size: string;
  base_mint: PubkeyStr;
  quote_mint: PubkeyStr;
  outcome_index: number;
  status: FillStatus;
  created_at: number;
  fills: OrderFillEvent[];
}

export interface UserOrderFillsResponse {
  orders: UserOrderFill[];
  next_cursor?: string;
  has_more: boolean;
}

type RawConditionalBalance = Omit<ConditionalBalance, "mint"> & {
  mint?: string;
  conditional_token?: string;
};

type RawUserOrderUpdateBalance = {
  outcomes: RawConditionalBalance[];
};

type RawWsOrder = Omit<WsOrder, "balance"> & {
  balance?: RawUserOrderUpdateBalance;
};

type RawOrderUpdate = Omit<OrderUpdate, "order"> & {
  order: RawWsOrder;
};

type RawUserSnapshotOrderCommon = Omit<UserSnapshotOrderCommon, "amount_in" | "amount_out"> & {
  amount_in?: string;
  amount_out?: string;
  maker_amount?: string;
  taker_amount?: string;
};

type RawUserSnapshotOrder =
  | ({ order_type: "limit"; tx_signature?: string } & RawUserSnapshotOrderCommon)
  | ({
      order_type: "trigger";
      trigger_order_id: string;
      trigger_price: string;
      trigger_type: TriggerType;
      time_in_force?: TimeInForce;
    } & RawUserSnapshotOrderCommon);

type RawUserSnapshotBalance = Omit<UserSnapshotBalance, "outcomes"> & {
  outcomes: RawConditionalBalance[];
};

type RawUserSnapshot = Omit<UserSnapshot, "orders" | "balances"> & {
  orders: RawUserSnapshotOrder[];
  balances: Record<string, RawUserSnapshotBalance>;
};

type RawUserBalanceUpdate = Omit<UserBalanceUpdate, "balance"> & {
  balance: RawUserOrderUpdateBalance;
};

type RawUserUpdate =
  | ({ event_type: "snapshot" } & RawUserSnapshot)
  | ({ event_type: "order" } & ({ order_type: "limit" } & RawOrderUpdate))
  | ({ event_type: "order" } & ({ order_type: "trigger" } & TriggerOrderUpdate))
  | ({ event_type: "balance_update" } & RawUserBalanceUpdate)
  | ({ event_type: "global_deposit_update" } & GlobalDepositUpdate)
  | ({ event_type: "nonce" | "nonce_update" } & NonceUpdate)
  | ({ event_type: "notification" } & NotificationUpdate);

type RawUserOrdersPayload = {
  user_pubkey: PubkeyStr;
  orders?: RawUserSnapshotOrder[];
  balances?: RawUserSnapshotBalance[];
  next_cursor?: string | null;
  has_more?: boolean;
};

export function normalizeConditionalBalance(balance: RawConditionalBalance): ConditionalBalance {
  const mint = balance.mint ?? balance.conditional_token;
  if (!mint) {
    throw new Error("Invalid conditional balance: missing mint");
  }

  return {
    outcome_index: balance.outcome_index,
    mint: asPubkeyStr(mint),
    idle: balance.idle,
    on_book: balance.on_book,
  };
}

function normalizeUserOrderUpdateBalance(
  balance: RawUserOrderUpdateBalance
): UserOrderUpdateBalance {
  return {
    outcomes: balance.outcomes.map(normalizeConditionalBalance),
  };
}

function normalizeWsOrder(order: RawWsOrder): WsOrder {
  return {
    ...order,
    balance: order.balance
      ? normalizeUserOrderUpdateBalance(order.balance)
      : undefined,
  };
}

export function normalizeOrderUpdate(update: RawOrderUpdate): OrderUpdate {
  return {
    ...update,
    order: normalizeWsOrder(update.order),
  };
}

function normalizeUserSnapshotOrderCommon(
  order: RawUserSnapshotOrderCommon
): UserSnapshotOrderCommon {
  const amountIn = order.amount_in ?? order.maker_amount;
  const amountOut = order.amount_out ?? order.taker_amount;

  if (amountIn === undefined) {
    throw new Error("Invalid user snapshot order: missing amount_in/maker_amount");
  }
  if (amountOut === undefined) {
    throw new Error("Invalid user snapshot order: missing amount_out/taker_amount");
  }

  return {
    ...order,
    amount_in: amountIn,
    amount_out: amountOut,
  };
}

export function normalizeUserSnapshotOrder(
  order: RawUserSnapshotOrder
): UserSnapshotOrder {
  const common = normalizeUserSnapshotOrderCommon(order);

  if (order.order_type === "limit") {
    return {
      ...common,
      order_type: "limit",
      tx_signature: order.tx_signature,
    };
  }

  if (order.order_type === "trigger") {
    return {
      ...common,
      order_type: "trigger",
      trigger_order_id: order.trigger_order_id,
      trigger_price: order.trigger_price,
      trigger_type: order.trigger_type,
      time_in_force: order.time_in_force,
    };
  }

  const rawOrderType = (order as { order_type: string }).order_type;
  throw new Error(`Invalid user snapshot order: unsupported order_type "${rawOrderType}"`);
}

export function normalizeUserSnapshotBalance(
  balance: RawUserSnapshotBalance
): UserSnapshotBalance {
  return {
    ...balance,
    outcomes: balance.outcomes.map(normalizeConditionalBalance),
  };
}

function normalizeUserSnapshotBalancesRecord(
  balances: Record<string, RawUserSnapshotBalance>
): Record<string, UserSnapshotBalance> {
  return Object.fromEntries(
    Object.entries(balances).map(([orderbookId, balance]) => [
      orderbookId,
      normalizeUserSnapshotBalance(balance),
    ])
  );
}

export function normalizeUserOrdersPayload(
  response: RawUserOrdersPayload
): {
  user_pubkey: PubkeyStr;
  orders: UserSnapshotOrder[];
  balances: UserSnapshotBalance[];
  next_cursor?: string;
  has_more: boolean;
} {
  return {
    user_pubkey: response.user_pubkey,
    orders: (response.orders ?? []).map(normalizeUserSnapshotOrder),
    balances: (response.balances ?? []).map(normalizeUserSnapshotBalance),
    next_cursor: response.next_cursor ?? undefined,
    has_more: response.has_more ?? false,
  };
}

export function normalizeUserUpdate(raw: RawUserUpdate): UserUpdate {
  switch (raw.event_type) {
    case "snapshot":
      return {
        ...raw,
        event_type: "snapshot",
        orders: raw.orders.map(normalizeUserSnapshotOrder),
        balances: normalizeUserSnapshotBalancesRecord(raw.balances),
        global_deposits: raw.global_deposits ?? [],
        notifications: raw.notifications ?? [],
        nonce: raw.nonce ?? 0,
      };
    case "order":
      if (raw.order_type === "limit") {
        return {
          ...normalizeOrderUpdate(raw),
          event_type: "order",
          order_type: "limit",
        };
      }
      if (raw.order_type === "trigger") {
        return {
          ...raw,
          event_type: "order",
          order_type: "trigger",
        };
      }
      throw new Error(
        `Invalid user order event: unsupported order_type "${(raw as { order_type: string }).order_type}"`
      );
    case "balance_update":
      return {
        ...raw,
        event_type: "balance_update",
        balance: normalizeUserOrderUpdateBalance(raw.balance),
      };
    case "global_deposit_update":
      return raw;
    case "nonce":
    case "nonce_update":
      return {
        ...raw,
        event_type: "nonce",
      };
    case "notification":
      return raw;
  }
}
