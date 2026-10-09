import Decimal from "decimal.js";

import type { FundingSource, OrderBookId, PubkeyStr, Side, TimeInForce } from "../../shared";

/** Supported resting-order kind. */
export enum OrderType {
  Limit = "limit",
}

export function orderTypeLabel(orderType: OrderType): string {
  switch (orderType) {
    case OrderType.Limit:
      return "Limit";
  }
}

/**
 * Committed lifecycle status of an order.
 *
 * Lowercase on the wire, as in `GET /api/users/order-fills`. Derived with the
 * backend's precedence: a closure reason wins, then a complete confirmed fill,
 * then pending (unconfirmed) fills, otherwise open.
 */
export enum OrderStatus {
  /** Resting, or awaiting its first match. */
  Open = "open",
  /** Some matched base awaits on-chain confirmation. */
  Pending = "pending",
  /** The whole original base is confirmed filled. */
  Filled = "filled",
  /**
   * Stopped resting (cancelled, expired, closure cutoff, or remainder of an
   * IOC/FOK order); see the closed reason.
   */
  Closed = "closed",
  /** A status this SDK version does not know (decoded only, never derived). */
  Unknown = "unknown",
}

/** Apply the backend's status precedence to committed base quantities. */
export function deriveOrderStatus(
  closedReason: string | undefined,
  originalBase: Decimal.Value,
  confirmedBase: Decimal.Value,
  pendingBase: Decimal.Value,
): OrderStatus {
  if (closedReason !== undefined) return OrderStatus.Closed;
  if (new Decimal(confirmedBase).eq(originalBase)) return OrderStatus.Filled;
  if (new Decimal(pendingBase).gt(0)) return OrderStatus.Pending;
  return OrderStatus.Open;
}

/**
 * A limit order's committed state. Size fields are decimal strings in
 * base-token units and satisfy
 * `size = filledSize + pendingSize + remainingSize + cancelledSize`.
 */
export interface LimitOrder {
  marketPubkey: PubkeyStr;
  orderbookId: OrderBookId;
  baseMint: PubkeyStr;
  quoteMint: PubkeyStr;
  orderHash: string;
  side: Side;
  /** Limit price, quote units per base unit. */
  price: string;
  /** Original order size. */
  size: string;
  /** Base filled by confirmed (on-chain authenticated) executions. */
  filledSize: string;
  /** Base matched but awaiting on-chain confirmation. */
  pendingSize: string;
  /** Base still resting on the book. */
  remainingSize: string;
  /** Base cancelled (explicitly, by expiry, IOC/FOK remainder, or closure). */
  cancelledSize: string;
  /** `undefined` when the backend reported a policy this SDK version does not know. */
  timeInForce?: TimeInForce;
  fundingSource: FundingSource;
  closedReason?: string;
  status: OrderStatus;
  /** Engine acceptance sequence; closures cut off by it. */
  acceptedSeq: bigint;
  /** Committed revision of this state; newer revisions supersede older ones. */
  committedRevision: bigint;
  createdAt: Date;
  /** Unix seconds; `0` means no expiration. */
  expiration: bigint;
}

/**
 * True while the order rests on the book or has fills awaiting confirmation
 * (which may still return to the book if they fail).
 */
export function limitOrderIsLive(order: LimitOrder): boolean {
  return new Decimal(order.remainingSize).gt(0) || new Decimal(order.pendingSize).gt(0);
}
