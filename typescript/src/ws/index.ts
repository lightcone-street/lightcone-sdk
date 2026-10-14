import type { MarketEvent } from "../domain/market";
import { normalizeUserUpdate, type AuthUpdate, type UserUpdate } from "../domain/order/wire";
import type { OrderBook, WsTickerData } from "../domain/orderbook";
import {
  FULL_PRECISION,
  normalizeAggregation,
  type BookAggregation,
} from "../domain/orderbook/aggregation";
import type {
  DepositAssetPriceEvent,
  DepositPrice,
  PriceHistory,
} from "../domain/price_history";
import type { WsTrade } from "../domain/trade";
import type {
  DepositTokenBalance,
  WalletDepositBalancesEvent,
} from "../domain/position";
import { WsError as WsErrorClass } from "../error";
import { LightconeEnv, wsUrl } from "../env";
import type { OrderBookId, PubkeyStr, Resolution } from "../shared";
import { parseJsonExact } from "../shared/json";

export * from "./client.node";
export * from "./subscriptions";
export type { IWsClient } from "./types";

export type MessageOut =
  | { method: "subscribe"; params: import("./subscriptions").SubscribeParams }
  | { method: "unsubscribe"; params: import("./subscriptions").UnsubscribeParams }
  | { method: "ping" };

/** Parsed inbound channel union; wallet balances contain a strict nested event union. */
export type MessageIn =
  | { type: "book_update"; version: number; data: OrderBook }
  | { type: "pong"; version: number; data: Record<string, never> }
  | { type: "user"; version: number; data: UserUpdate }
  | { type: "error"; version: number; data: WsError }
  | { type: "price_history"; version: number; data: PriceHistory }
  | { type: "trades"; version: number; data: WsTrade }
  | { type: "auth"; version: number; data: AuthUpdate }
  | { type: "ticker"; version: number; data: WsTickerData }
  | { type: "market"; version: number; data: MarketEvent }
  | { type: "deposit_price"; version: number; data: DepositPrice }
  | { type: "deposit_asset_price"; version: number; data: DepositAssetPriceEvent }
  | {
      type: "wallet_deposit_balances";
      version: number;
      data: WalletDepositBalancesEvent;
    };

export type Kind = MessageIn;

export interface WsError {
  error: string;
  code?: string;
  orderbook_id?: string;
  /**
   * Aggregation of the affected book subscription on book-scoped errors
   * (`ENGINE_UNAVAILABLE`, `SUBSCRIPTION_LIMIT_REACHED`,
   * `INVALID_ORDERBOOK_SUBSCRIPTION`). Absent = full precision; pass to
   * `aggregationFromFrame` to identify the `(orderbook, aggregation)` pair.
   */
  n_sig_figs?: number;
  mantissa?: number;
  wallet_address?: string;
  deposit_asset?: string;
  hint?: string;
  details?: string;
}

export type WsEvent =
  | { type: "Message"; message: MessageIn }
  | { type: "Connected" }
  | { type: "Disconnected"; code?: number; reason: string }
  | { type: "Error"; error: string }
  | { type: "MaxReconnectReached" };

export interface WsConfig {
  url: string;
  reconnect: boolean;
  maxReconnectAttempts: number;
  baseReconnectDelayMs: number;
  pingIntervalMs: number;
  pongTimeoutMs: number;
}

export const WS_DEFAULT_CONFIG: WsConfig = {
  url: wsUrl(LightconeEnv.Prod),
  reconnect: true,
  maxReconnectAttempts: 10,
  baseReconnectDelayMs: 1_000,
  pingIntervalMs: 30_000,
  pongTimeoutMs: 10_000,
};

export enum ReadyState {
  Connecting = 0,
  Open = 1,
  Closing = 2,
  Closed = 3,
}

export function readyStateFrom(value: number): ReadyState {
  switch (value) {
    case 0:
      return ReadyState.Connecting;
    case 1:
      return ReadyState.Open;
    case 2:
      return ReadyState.Closing;
    case 3:
    default:
      return ReadyState.Closed;
  }
}

export function ping(): MessageOut {
  return { method: "ping" };
}

/**
 * Subscribe to book snapshots, optionally aggregated (Hyperliquid-style).
 *
 * Omit `aggregation` for the raw book. The aggregation is normalized before
 * sending ((5, none) → (5, 1)); validate with `validateAggregation` first —
 * invalid combinations are rejected server-side with
 * `INVALID_ORDERBOOK_SUBSCRIPTION`. `undefined` fields are dropped by
 * `JSON.stringify`, so full precision stays byte-identical to the
 * pre-aggregation message.
 */
export function subscribeBooks(
  orderbookIds: OrderBookId[],
  aggregation: BookAggregation = FULL_PRECISION
): MessageOut {
  const sorted = [...orderbookIds].sort();
  const normalized = normalizeAggregation(aggregation);
  return {
    method: "subscribe",
    params: {
      type: "book_update",
      orderbook_ids: sorted,
      nSigFigs: normalized.nSigFigs,
      mantissa: normalized.mantissa,
    },
  };
}

/**
 * Unsubscribe a book subscription. The aggregation must match the one
 * subscribed (normalized) or the server removes nothing.
 */
export function unsubscribeBooks(
  orderbookIds: OrderBookId[],
  aggregation: BookAggregation = FULL_PRECISION
): MessageOut {
  const sorted = [...orderbookIds].sort();
  const normalized = normalizeAggregation(aggregation);
  return {
    method: "unsubscribe",
    params: {
      type: "book_update",
      orderbook_ids: sorted,
      nSigFigs: normalized.nSigFigs,
      mantissa: normalized.mantissa,
    },
  };
}

export function subscribeTrades(orderbookIds: OrderBookId[]): MessageOut {
  return {
    method: "subscribe",
    params: {
      type: "trades",
      orderbook_ids: [...orderbookIds].sort(),
    },
  };
}

export function unsubscribeTrades(orderbookIds: OrderBookId[]): MessageOut {
  return {
    method: "unsubscribe",
    params: {
      type: "trades",
      orderbook_ids: [...orderbookIds].sort(),
    },
  };
}

export function subscribeUser(walletAddress: PubkeyStr): MessageOut {
  return {
    method: "subscribe",
    params: {
      type: "user",
      wallet_address: walletAddress,
    },
  };
}

export function unsubscribeUser(walletAddress: PubkeyStr): MessageOut {
  return {
    method: "unsubscribe",
    params: {
      type: "user",
      wallet_address: walletAddress,
    },
  };
}

/**
 * Subscribe to a wallet owned by the authenticated user.
 * The channel begins with a complete replacement snapshot and then emits
 * absolute SPL/native updates plus non-mutating status events.
 */
export function subscribeWalletDepositBalances(
  walletAddress: PubkeyStr
): MessageOut {
  return {
    method: "subscribe",
    params: {
      type: "wallet_deposit_balances",
      wallet_address: walletAddress,
    },
  };
}

/**
 * Stop the authenticated wallet stream identified by this exact wallet address.
 * This wire operation is separate from clearing local reconnect tracking.
 */
export function unsubscribeWalletDepositBalances(
  walletAddress: PubkeyStr
): MessageOut {
  return {
    method: "unsubscribe",
    params: {
      type: "wallet_deposit_balances",
      wallet_address: walletAddress,
    },
  };
}

export function subscribePriceHistory(orderbookId: OrderBookId, resolution: Resolution): MessageOut {
  return {
    method: "subscribe",
    params: {
      type: "price_history",
      orderbook_id: orderbookId,
      resolution,
      include_ohlcv: false,
    },
  };
}

export function unsubscribePriceHistory(orderbookId: OrderBookId, resolution: Resolution): MessageOut {
  return {
    method: "unsubscribe",
    params: {
      type: "price_history",
      orderbook_id: orderbookId,
      resolution,
    },
  };
}

export function subscribeTicker(orderbookIds: OrderBookId[]): MessageOut {
  return {
    method: "subscribe",
    params: {
      type: "ticker",
      orderbook_ids: [...orderbookIds].sort(),
    },
  };
}

export function unsubscribeTicker(orderbookIds: OrderBookId[]): MessageOut {
  return {
    method: "unsubscribe",
    params: {
      type: "ticker",
      orderbook_ids: [...orderbookIds].sort(),
    },
  };
}

export function subscribeMarket(marketPubkey: PubkeyStr): MessageOut {
  return {
    method: "subscribe",
    params: {
      type: "market",
      market_pubkey: marketPubkey,
    },
  };
}

export function unsubscribeMarket(marketPubkey: PubkeyStr): MessageOut {
  return {
    method: "unsubscribe",
    params: {
      type: "market",
      market_pubkey: marketPubkey,
    },
  };
}

export function subscribeDepositPrice(depositAsset: string, resolution: Resolution): MessageOut {
  return {
    method: "subscribe",
    params: {
      type: "deposit_price",
      deposit_asset: depositAsset,
      resolution,
    },
  };
}

export function unsubscribeDepositPrice(depositAsset: string, resolution: Resolution): MessageOut {
  return {
    method: "unsubscribe",
    params: {
      type: "deposit_price",
      deposit_asset: depositAsset,
      resolution,
    },
  };
}

/**
 * Subscribe to the live spot price for one deposit asset (snapshot +
 * per-asset price ticks). Distinct from `subscribeDepositPrice`, which
 * carries OHLCV candles per resolution.
 */
export function subscribeDepositAssetPrice(depositAsset: string): MessageOut {
  return {
    method: "subscribe",
    params: { type: "deposit_asset_price", deposit_asset: depositAsset },
  };
}

export function unsubscribeDepositAssetPrice(depositAsset: string): MessageOut {
  return {
    method: "unsubscribe",
    params: { type: "deposit_asset_price", deposit_asset: depositAsset },
  };
}

const VALID_MESSAGE_TYPES = new Set([
  "book_update",
  "pong",
  "user",
  "error",
  "price_history",
  "trades",
  "auth",
  "ticker",
  "market",
  "deposit_price",
  "deposit_asset_price",
  "wallet_deposit_balances",
]);

/**
 * Parse and normalize one inbound frame.
 *
 * Wallet-balance payloads cross a runtime validation boundary here: malformed
 * nested discriminators, exact values, or slots throw a protocol `WsError`
 * instead of escaping behind TypeScript-only types.
 */
export function parseMessageIn(input: string): MessageIn {
  const parsed: unknown = parseJsonExact(input);
  if (typeof parsed !== "object" || parsed === null || !("type" in parsed)) {
    throw new WsErrorClass("ProtocolError", `Invalid WS message: missing "type" field`);
  }
  const obj = parsed as Record<string, unknown>;
  if (typeof obj.type !== "string" || !VALID_MESSAGE_TYPES.has(obj.type)) {
    throw new WsErrorClass("ProtocolError", `Invalid WS message type: "${String(obj.type)}"`);
  }
  if (!("version" in obj) || typeof obj.version !== "number") {
    throw new WsErrorClass("ProtocolError", `Invalid WS message: missing or invalid "version" field`);
  }
  if (!("data" in obj) || typeof obj.data !== "object" || obj.data === null) {
    throw new WsErrorClass("ProtocolError", `Invalid WS message: missing or invalid "data" field`);
  }
  let message = parsed as MessageIn;
  if (message.type === "book_update") {
    const data = message.data as unknown as Record<string, unknown>;
    const seq = data.seq;
    if ((typeof seq !== "number" && typeof seq !== "bigint") ||
        (typeof seq === "number" && (!Number.isSafeInteger(seq) || seq < 0)) ||
        (typeof seq === "bigint" && seq < 0n)) {
      throw new WsErrorClass("ProtocolError", "Invalid book_update seq");
    }
    // Validate only populated level arrays so existing empty/resync frame
    // tolerance is unchanged while exact quote liquidity cannot be omitted.
    for (const side of ["bids", "asks"] as const) {
      const levels = data[side];
      if (!Array.isArray(levels)) continue;
      for (const level of levels) {
        if (
          typeof level !== "object" ||
          level === null ||
          Array.isArray(level) ||
          typeof (level as Record<string, unknown>).quote_notional !== "string"
        ) {
          throw new WsErrorClass(
            "ProtocolError",
            `Invalid book_update ${side} level: missing or invalid quote_notional`,
          );
        }
      }
    }
    message = {
      ...message,
      data: { ...message.data, seq: BigInt(seq) },
    };
  }
  if (message.type === "user") {
    return {
      ...message,
      data: normalizeUserUpdate(message.data as Parameters<typeof normalizeUserUpdate>[0]),
    };
  }
  if (message.type === "wallet_deposit_balances") {
    return {
      ...message,
      data: normalizeWalletDepositBalancesEvent(message.data),
    };
  }
  return message;
}

/** Validate the nested wallet event before exposing its discriminated union. */
function normalizeWalletDepositBalancesEvent(
  input: unknown
): WalletDepositBalancesEvent {
  const data = requireObject(input, "wallet_deposit_balances data");
  const eventType = requireString(data, "event_type");
  const walletAddress = requireString(data, "wallet_address") as PubkeyStr;

  switch (eventType) {
    case "wallet_deposit_balance_snapshot": {
      const rawBalances = requireObject(data.balances, "balances");
      const balances = {} as Record<PubkeyStr, DepositTokenBalance>;
      for (const [mint, balance] of Object.entries(rawBalances)) {
        balances[mint as PubkeyStr] = normalizeDepositTokenBalance(balance);
      }
      return {
        event_type: eventType,
        wallet_address: walletAddress,
        context_slot: requireContextSlot(data),
        balances,
        native_sol_balance: requireNativeSolBalance(data),
      };
    }
    case "wallet_deposit_balance_update":
      return {
        event_type: eventType,
        wallet_address: walletAddress,
        context_slot: requireContextSlot(data),
        balance: normalizeDepositTokenBalance(data.balance),
      };
    case "wallet_native_sol_balance_update":
      return {
        event_type: eventType,
        wallet_address: walletAddress,
        context_slot: requireContextSlot(data),
        native_sol_balance: requireNativeSolBalance(data),
      };
    case "wallet_deposit_balance_status": {
      const status = requireString(data, "status");
      if (status !== "reconnecting" && status !== "metadata_unavailable") {
        throw protocolError(`Invalid wallet balance status: "${status}"`);
      }
      return {
        event_type: eventType,
        wallet_address: walletAddress,
        status,
        code: requireString(data, "code"),
      };
    }
    default:
      throw protocolError(`Invalid wallet balance event type: "${eventType}"`);
  }
}

/** Preserve exact strings and nullable icons while rejecting incomplete balances. */
function normalizeDepositTokenBalance(input: unknown): DepositTokenBalance {
  const balance = requireObject(input, "deposit-token balance");
  const normalized: DepositTokenBalance = {
    mint: requireString(balance, "mint") as PubkeyStr,
    idle: requireString(balance, "idle"),
    symbol: requireString(balance, "symbol"),
    name: requireString(balance, "name"),
  };
  for (const field of ["icon_url_low", "icon_url_medium", "icon_url_high"] as const) {
    const value = balance[field];
    if (value !== undefined) {
      if (value !== null && typeof value !== "string") {
        throw protocolError(`Invalid deposit-token balance ${field}`);
      }
      normalized[field] = value;
    }
  }
  return normalized;
}

function requireObject(input: unknown, field: string): Record<string, unknown> {
  if (typeof input !== "object" || input === null || Array.isArray(input)) {
    throw protocolError(`Invalid ${field}: expected object`);
  }
  return input as Record<string, unknown>;
}

function requireString(data: Record<string, unknown>, field: string): string {
  const value = data[field];
  if (typeof value !== "string") {
    throw protocolError(`Invalid wallet balance ${field}: expected string`);
  }
  return value;
}

function requireNativeSolBalance(data: Record<string, unknown>): string {
  // Canonical nine-place syntax maps directly to lamports without rounding or exponents.
  const value = requireString(data, "native_sol_balance");
  if (!/^(?:0|[1-9][0-9]*)\.[0-9]{9}$/.test(value)) {
    throw protocolError(
      "Invalid wallet balance native_sol_balance: expected exactly nine decimal places"
    );
  }
  return value;
}

function requireContextSlot(data: Record<string, unknown>): number {
  // Safe integers prevent precision loss when state compares JavaScript slots.
  const value = data.context_slot;
  if (!Number.isSafeInteger(value) || (value as number) < 0) {
    throw protocolError("Invalid wallet balance context_slot");
  }
  return value as number;
}

function protocolError(message: string): WsErrorClass {
  return new WsErrorClass("ProtocolError", message);
}
