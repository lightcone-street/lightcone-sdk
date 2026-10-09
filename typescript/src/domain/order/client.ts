import { V1Transaction, type V1TransactionContext } from "../../program/transaction";
import bs58 from "bs58";
import { Keypair, type PublicKey, type TransactionInstruction } from "@solana/web3.js";
import type { ClientContext } from "../../context";
import { requireConnection, requireSigningStrategy } from "../../context";
import { SdkError } from "../../error";
import { ProgramSdkError } from "../../program/error";
import { RetryPolicy } from "../../http";
import { isUserCancellation } from "../../shared/signing";
import {
  buildCancelOrderIx,
  buildCloseOrderStatusIx,
} from "../../program/instructions";
import {
  hashOrder as programHashOrder,
  signOrder as programSignOrder,
  signOrderFull,
  createBidOrder as programCreateBidOrder,
  createAskOrder as programCreateAskOrder,
  createSignedBidOrder as programCreateSignedBidOrder,
  createSignedAskOrder as programCreateSignedAskOrder,
  signCancelOrder,
  signCancelAll,
  cancelOrderMessage,
  cancelAllMessage,
  generateCancelAllSalt as programGenerateCancelAllSalt,
} from "../../program/orders";
import { getOrderStatusPda } from "../../program/pda";
import { deserializeOrderStatus } from "../../program/accounts";
import type {
  SignedOrder,
  CloseOrderStatusParams,
  BidOrderParams,
  AskOrderParams,
  OrderStatus as ProgramOrderStatus,
} from "../../program/types";
import { OrderSide } from "../../program/types";
import {
  validateRawAmounts,
  type OrderBookId,
  type OrderbookRules,
  type PubkeyStr,
  type SubmitOrderRequest,
} from "../../shared";
import { LimitOrderEnvelope } from "../../program/envelope";
import { Orderbooks } from "../orderbook/client";
import { quotePerBase } from "../../shared/price";
import {
  wireArray,
  wireBool,
  wireDecimal,
  wireEnumOrUnknown,
  wireI128,
  wireInt,
  wireNonEmptyString,
  wireObject,
  wireOptString,
  wireOptU64,
  wireOrderBookId,
  wirePubkey,
  wireString,
  wireStringOr,
  wireU64,
} from "../../shared/wire_decode";
import { fundingAccountFromWire, type FundingAccount } from "../position/wire";
import {
  initialCohortFromWire,
  orderStateFromWire,
  userOrderFillsResponseFromWire,
  userOrderFromWire,
  type InitialCohort,
  type OrderState,
  type UserOrder,
  type UserOrderFillsResponse,
} from "./wire";

// ─── Request types ───────────────────────────────────────────────────────────

export interface CancelBody {
  order_hash: string;
  maker: PubkeyStr;
  signature: string;
}

export function cancelBodyFromBase58(
  orderHash: string,
  maker: PubkeyStr,
  signatureBase58: string
): CancelBody {
  return {
    order_hash: orderHash,
    maker,
    signature: Buffer.from(bs58.decode(signatureBase58)).toString("hex"),
  };
}

export function cancelBodySigned(
  orderHash: string,
  maker: PubkeyStr,
  keypair: Keypair
): CancelBody {
  const signature = signCancelOrder(orderHash, keypair);
  return { order_hash: orderHash, maker, signature };
}

export interface CancelAllBody {
  user_pubkey: PubkeyStr;
  orderbook_id: OrderBookId;
  signature: string;
  timestamp: number;
  salt: string;
}

export function cancelAllBodyFromBase58(
  userPubkey: PubkeyStr,
  orderbookId: OrderBookId,
  timestamp: number,
  salt: string,
  signatureBase58: string
): CancelAllBody {
  return {
    user_pubkey: userPubkey,
    orderbook_id: orderbookId,
    signature: Buffer.from(bs58.decode(signatureBase58)).toString("hex"),
    timestamp,
    salt,
  };
}

export function cancelAllBodySigned(
  userPubkey: PubkeyStr,
  orderbookId: OrderBookId,
  timestamp: number,
  salt: string,
  keypair: Keypair
): CancelAllBody {
  const signature = signCancelAll(userPubkey, orderbookId, timestamp, salt, keypair);
  return { user_pubkey: userPubkey, orderbook_id: orderbookId, signature, timestamp, salt };
}

// ─── Response types ──────────────────────────────────────────────────────────

/** Outcome of an accepted order submission. */
export enum SubmitOrderStatus {
  /**
   * Accepted; no fill awaits confirmation. The order may rest, or (IOC) its
   * unfilled remainder is already cancelled.
   */
  Accepted = "accepted",
  /**
   * Accepted with matched base awaiting on-chain confirmation, or the
   * acceptance is durable but its committed view was not yet readable (then
   * `state` and `initial_cohort` are `undefined`).
   */
  AcceptedPending = "accepted_pending",
  /** The whole original base is confirmed filled. */
  Filled = "filled",
  /** An accepted status this SDK version does not know. */
  Unknown = "unknown",
}

/** An authenticated fill captured with a submission response. */
export interface FillInfo {
  /**
   * `"<execution_id>:<leg_index>:<projection_generation>"`, the same id as
   * REST `trade_id`.
   */
  fill_id: string;
  execution_id: string;
  counterparty: PubkeyStr;
  counterparty_order_hash: string;
  /** Filled size in base-token units. */
  base_amount: string;
  /** Filled notional in quote-token units. */
  quote_amount: string;
  /** Whether the submitted order was the maker of this fill. */
  is_maker: boolean;
  /** Signed fee estimate in raw atoms of `fee_mint` (negative = rebate). */
  fee_estimate_atoms: bigint;
  /** Captured fee rate in basis points (negative = rebate). */
  fee_bps: number;
  fee_mint: PubkeyStr;
}

/** Fill price in quote units per base unit (`quote_amount / base_amount`). */
export function fillInfoPrice(fill: FillInfo): string | undefined {
  return quotePerBase(fill.quote_amount, fill.base_amount);
}

/**
 * Response of `POST /api/orders/submit` for an accepted order.
 *
 * Business rejections arrive as `SdkError` `ApiRejected` with a
 * `RejectionCode`. An `ENGINE_UNAVAILABLE` error means the outcome is
 * unknown: see {@link Orders.submit} for how to resolve it.
 */
export interface SubmitOrderResponse {
  order_hash: string;
  status: SubmitOrderStatus;
  /** Committed cumulative state; `undefined` when the view is not yet readable. */
  state?: OrderState;
  /** Executions selected at acceptance; `undefined` alongside `state`. */
  initial_cohort?: InitialCohort;
  /** Up to 16 authenticated fills, oldest first. */
  fills: FillInfo[];
  /** False when more fills exist or a newer revision prevented capture. */
  fills_complete: boolean;
  /** Continue with `Orders.getOrderFillPage` (order hash + this cursor). */
  fills_next_cursor?: string;
}

/** Base filled by confirmed fills, when the committed view is available. */
export function submitFilledBase(response: SubmitOrderResponse): string | undefined {
  return response.state?.confirmed_base;
}

/** Base still resting on the book, when the committed view is available. */
export function submitOpenBase(response: SubmitOrderResponse): string | undefined {
  return response.state?.open_base;
}

/** Disposition of a single-order cancellation. */
export enum CancelStatus {
  /** The open remainder was cancelled by this request. */
  Cancelled = "cancelled",
  /** The order was already closed (cancelled, expired, or cut off). */
  AlreadyClosed = "already_closed",
  /** The order was already completely filled. */
  AlreadyFilled = "already_filled",
  /** A disposition this SDK version does not know. */
  Unknown = "unknown",
}

/**
 * Cancellation quantities in raw base-token atoms (`quantity_unit` is
 * `"base_atoms"`), sent as integer strings.
 */
export interface CancelQuantities {
  /** Base atoms cancelled by this request. */
  newly_cancelled_base: bigint;
  /** Base atoms confirmed filled. */
  confirmed_base: bigint;
  /** Base atoms matched and awaiting on-chain confirmation. */
  pending_base: bigint;
  /** Base atoms still open after the request. */
  remaining_open_base: bigint;
}

/** Response of `POST /api/orders/cancel`. */
export interface CancelSuccess {
  status: CancelStatus;
  order_hash: string;
  /**
   * Present on every successful cancellation (an unknown hash is the
   * `ORDER_NOT_FOUND` rejection instead); `undefined` only if a backend omits
   * it, never invented zeroes.
   */
  quantities?: CancelQuantities;
  /** Unit of `quantities`: always `"base_atoms"`. */
  quantity_unit: string;
  /** Committed revision of the cancellation. */
  revision: bigint;
  /** Why the order is closed; empty on the wire when none. */
  closed_reason?: string;
}

/** Committed accepted-order cutoff returned by cancel-all. */
export interface ClosureAck {
  /** Closure operation id (UUID). */
  operation_id: string;
  committed_revision: bigint;
  /** `"Wallet:<wallet>"` or `"WalletBook:<wallet>:<orderbook>"`. */
  scope: string;
  /**
   * Orders with `accepted_seq` at or below this are closed; `undefined` when
   * the scope had no accepted orders.
   */
  accepted_seq_cutoff?: bigint;
  /**
   * True while bounded per-order cleanup is still running; individual order
   * facts follow on the `user` channel.
   */
  cleanup_pending: boolean;
}

/** Response of `POST /api/orders/cancel-all`. */
export interface CancelAllSuccess {
  /** Always `"success"`. */
  status: string;
  user_pubkey: PubkeyStr;
  /** Empty for a wallet-wide cancel-all. */
  orderbook_id: OrderBookId;
  closure?: ClosureAck;
  message: string;
}

/**
 * Response of `GET /api/users/orders`: one page of the wallet's open and
 * pending orders plus one page of its funding accounts.
 *
 * A page can be empty while `has_more` is true. Funding pages continue via
 * `positions().positionsPage(undefined, next_funding_cursor)`, which reads the
 * same committed funding view.
 */
export interface UserOrdersResponse {
  user_pubkey: PubkeyStr;
  orders: UserOrder[];
  funding_accounts: FundingAccount[];
  /** Opaque order cursor (`"<accepted_seq>:<hash>"`). */
  next_cursor?: string;
  has_more: boolean;
  next_funding_cursor?: string;
  funding_has_more: boolean;
  /** Database revision of this page; later pages may be newer. */
  committed_revision: bigint;
  projection_generation: bigint;
}

const I16 = [-32_768, 32_767] as const;
const SUBMIT_STATUSES = [
  SubmitOrderStatus.Accepted,
  SubmitOrderStatus.AcceptedPending,
  SubmitOrderStatus.Filled,
] as const;
const CANCEL_STATUSES = [
  CancelStatus.Cancelled,
  CancelStatus.AlreadyClosed,
  CancelStatus.AlreadyFilled,
] as const;

function fillInfoFromWire(value: unknown, field: string): FillInfo {
  const data = wireObject(value, field);
  return {
    fill_id: wireString(data.fill_id, `${field}.fill_id`),
    execution_id: wireString(data.execution_id, `${field}.execution_id`),
    counterparty: wirePubkey(data.counterparty, `${field}.counterparty`),
    counterparty_order_hash: wireString(data.counterparty_order_hash, `${field}.counterparty_order_hash`),
    base_amount: wireDecimal(data.base_amount, `${field}.base_amount`),
    quote_amount: wireDecimal(data.quote_amount, `${field}.quote_amount`),
    is_maker: wireBool(data.is_maker, `${field}.is_maker`),
    fee_estimate_atoms: wireI128(data.fee_estimate_atoms, `${field}.fee_estimate_atoms`),
    fee_bps: wireInt(data.fee_bps, `${field}.fee_bps`, ...I16),
    fee_mint: wirePubkey(data.fee_mint, `${field}.fee_mint`),
  };
}

/** Decode `POST /api/orders/submit`. */
export function submitOrderResponseFromWire(value: unknown): SubmitOrderResponse {
  const data = wireObject(value, "submit");
  return {
    order_hash: wireString(data.order_hash, "submit.order_hash"),
    status: wireEnumOrUnknown(data.status, "submit.status", SUBMIT_STATUSES, SubmitOrderStatus.Unknown),
    state:
      data.state === undefined || data.state === null
        ? undefined
        : orderStateFromWire(data.state, "submit.state"),
    initial_cohort:
      data.initial_cohort === undefined || data.initial_cohort === null
        ? undefined
        : initialCohortFromWire(data.initial_cohort, "submit.initial_cohort"),
    fills: wireArray(data.fills, "submit.fills", fillInfoFromWire, true),
    fills_complete: wireBool(data.fills_complete, "submit.fills_complete", false),
    fills_next_cursor: wireOptString(data.fills_next_cursor, "submit.fills_next_cursor"),
  };
}

/** Decode `POST /api/orders/cancel`. */
export function cancelSuccessFromWire(value: unknown): CancelSuccess {
  const data = wireObject(value, "cancel");
  let quantities: CancelQuantities | undefined;
  if (data.quantities !== undefined && data.quantities !== null) {
    const raw = wireObject(data.quantities, "cancel.quantities");
    quantities = {
      newly_cancelled_base: wireU64(raw.newly_cancelled_base, "cancel.quantities.newly_cancelled_base"),
      confirmed_base: wireU64(raw.confirmed_base, "cancel.quantities.confirmed_base"),
      pending_base: wireU64(raw.pending_base, "cancel.quantities.pending_base"),
      remaining_open_base: wireU64(raw.remaining_open_base, "cancel.quantities.remaining_open_base"),
    };
  }
  return {
    status: wireEnumOrUnknown(data.status, "cancel.status", CANCEL_STATUSES, CancelStatus.Unknown),
    order_hash: wireString(data.order_hash, "cancel.order_hash"),
    quantities,
    quantity_unit: wireString(data.quantity_unit, "cancel.quantity_unit"),
    revision: wireU64(data.revision, "cancel.revision"),
    closed_reason: wireNonEmptyString(data.closed_reason, "cancel.closed_reason"),
  };
}

/** Decode `POST /api/orders/cancel-all`. */
export function cancelAllSuccessFromWire(value: unknown): CancelAllSuccess {
  const data = wireObject(value, "cancel_all");
  let closure: ClosureAck | undefined;
  if (data.closure !== undefined && data.closure !== null) {
    const raw = wireObject(data.closure, "cancel_all.closure");
    closure = {
      operation_id: wireString(raw.operation_id, "cancel_all.closure.operation_id"),
      committed_revision: wireU64(raw.committed_revision, "cancel_all.closure.committed_revision"),
      scope: wireString(raw.scope, "cancel_all.closure.scope"),
      accepted_seq_cutoff: wireOptU64(raw.accepted_seq_cutoff, "cancel_all.closure.accepted_seq_cutoff"),
      cleanup_pending: wireBool(raw.cleanup_pending, "cancel_all.closure.cleanup_pending"),
    };
  }
  return {
    status: wireString(data.status, "cancel_all.status"),
    user_pubkey: wirePubkey(data.user_pubkey, "cancel_all.user_pubkey"),
    orderbook_id: wireOrderBookId(wireStringOr(data.orderbook_id, "cancel_all.orderbook_id", ""), "cancel_all.orderbook_id"),
    closure,
    message: wireString(data.message, "cancel_all.message"),
  };
}

/** Decode `GET /api/users/orders`. */
export function userOrdersResponseFromWire(value: unknown): UserOrdersResponse {
  const data = wireObject(value, "user_orders");
  return {
    user_pubkey: wirePubkey(data.user_pubkey, "user_orders.user_pubkey"),
    orders: wireArray(data.orders, "user_orders.orders", userOrderFromWire),
    funding_accounts: wireArray(
      data.funding_accounts,
      "user_orders.funding_accounts",
      fundingAccountFromWire,
      true,
    ),
    next_cursor: wireOptString(data.next_cursor, "user_orders.next_cursor"),
    has_more: wireBool(data.has_more, "user_orders.has_more"),
    next_funding_cursor: wireOptString(data.next_funding_cursor, "user_orders.next_funding_cursor"),
    funding_has_more: wireBool(data.funding_has_more, "user_orders.funding_has_more", false),
    committed_revision: wireU64(data.committed_revision, "user_orders.committed_revision"),
    projection_generation: wireU64(data.projection_generation, "user_orders.projection_generation"),
  };
}

/** Optional filters of `GET /api/users[/{wallet}]/order-fills`. */
interface OrderFillsQuery {
  marketPubkey?: string;
  limit?: number;
  cursor?: string;
  orderHash?: string;
  fillCursor?: string;
  fills?: number;
}

/**
 * Query for the order-fills routes. Only the parameters given are sent: the
 * backend rejects unknown parameters, combining `cursor` with `order_hash`,
 * and a `fill_cursor` with `fills` of `0`.
 */
function orderFillsUrl(path: string, query: OrderFillsQuery): string {
  const params = new URLSearchParams();
  if (query.marketPubkey !== undefined) params.set("market_pubkey", query.marketPubkey);
  if (query.limit !== undefined) params.set("limit", String(query.limit));
  if (query.cursor !== undefined) params.set("cursor", query.cursor);
  if (query.orderHash !== undefined) params.set("order_hash", query.orderHash);
  if (query.fillCursor !== undefined) params.set("fill_cursor", query.fillCursor);
  if (query.fills !== undefined) params.set("fills", String(query.fills));
  const qs = params.toString();
  return qs.length === 0 ? path : `${path}?${qs}`;
}

// ─── Sub-client ──────────────────────────────────────────────────────────────

export class Orders {
  constructor(private readonly client: ClientContext) {}

  // ── PDA helpers ──────────────────────────────────────────────────────

  statusPda(orderHash: Buffer): PublicKey {
    return getOrderStatusPda(orderHash, this.client.programId)[0];
  }

  // ── Envelope factories ────────────────────────────────────────────────

  limitOrder(): LimitOrderEnvelope {
    return LimitOrderEnvelope.new().depositSource(this.client.depositSource);
  }

  // ── Helpers ──────────────────────────────────────────────────────────

  generateCancelAllSalt(): string {
    return programGenerateCancelAllSalt();
  }

  // ── HTTP methods ─────────────────────────────────────────────────────

  /**
   * Submit a signed limit order. Not retried.
   *
   * An `ENGINE_UNAVAILABLE` error means the outcome is unknown. Resolve it by
   * submitting the identical signed `request` again: a `DUPLICATE_ORDER`
   * rejection or an `ALREADY_EXISTS` error proves the first submission was
   * accepted; any other result is the outcome of this one. Never sign a
   * replacement with a new salt, which is a different order. Order reads list
   * only open and pending orders, so they cannot prove an order was never
   * accepted.
   */
  async submit(request: SubmitOrderRequest): Promise<SubmitOrderResponse> {
    await this.preflightSubmit(request);
    const url = `${this.client.http.baseUrl()}/api/orders/submit`;
    return submitOrderResponseFromWire(
      await this.client.http.post<unknown, SubmitOrderRequest>(url, request, RetryPolicy.None),
    );
  }

  /** Cancel one order's open remainder. Not retried. */
  async cancel(body: CancelBody): Promise<CancelSuccess> {
    const url = `${this.client.http.baseUrl()}/api/orders/cancel`;
    return cancelSuccessFromWire(
      await this.client.http.post<unknown, CancelBody>(url, body, RetryPolicy.None),
    );
  }

  /**
   * Close every order of the wallet (optionally on one orderbook) accepted up
   * to a committed cutoff. Not retried. `body.timestamp` must be within 30
   * seconds of server time, and `body.salt` non-empty, at most 128 bytes, and
   * never reused.
   */
  async cancelAll(body: CancelAllBody): Promise<CancelAllSuccess> {
    const url = `${this.client.http.baseUrl()}/api/orders/cancel-all`;
    return cancelAllSuccessFromWire(
      await this.client.http.post<unknown, CancelAllBody>(url, body, RetryPolicy.None),
    );
  }

  /** Check the signed amounts against the orderbook's fetched admission rules. */
  private async preflightSubmit(request: SubmitOrderRequest): Promise<void> {
    const rules = await new Orderbooks(this.client).decimals(request.orderbook_id);
    if (request.side !== OrderSide.BID && request.side !== OrderSide.ASK) {
      throw ProgramSdkError.invalidSide(request.side);
    }
    validateRawAmounts(request.amount_in, request.amount_out, request.side, rules);
  }

  /**
   * Fetch the authenticated user's open and pending orders, with a first page
   * of funding accounts. Wallet is resolved server-side from the auth cookie,
   * so no parameter is required. `limit` defaults to 200 and is clamped to
   * 1..256 server-side; `cursor` is a previous `next_cursor`.
   */
  async getUserOrders(
    limit?: number,
    cursor?: string
  ): Promise<UserOrdersResponse> {
    const url = buildUserOrdersAuthenticatedUrl(this.client.http.baseUrl(), limit, cursor);
    return userOrdersResponseFromWire(
      await this.client.http.get<unknown>(url, RetryPolicy.Idempotent),
    );
  }

  /**
   * Same as {@link getUserOrders}, but uses the supplied `cookieHeader` for
   * this call instead of the SDK's process-wide cookie store. For
   * server-side cookie forwarding (SSR / route handlers).
   */
  async getUserOrdersWithCookies(
    limit: number | undefined,
    cursor: string | undefined,
    cookieHeader: string,
  ): Promise<UserOrdersResponse> {
    const url = buildUserOrdersAuthenticatedUrl(this.client.http.baseUrl(), limit, cursor);
    return userOrdersResponseFromWire(
      await this.client.http.getWithCookies<unknown>(url, RetryPolicy.Idempotent, cookieHeader),
    );
  }

  /**
   * Fetch the authenticated user's filled orders with nested fill events.
   * Wallet is resolved server-side from the auth cookie.
   *
   * Includes orders where the user was either maker or taker. Optionally
   * filter by market. Orders are sorted by most recent fill first; each
   * carries its oldest-first page of `fills` fills (default and maximum 16;
   * `0` reads none and only reports `fills_has_more`). `limit` defaults to 20
   * and is clamped to 1..100 server-side.
   */
  async getUserOrderFills(
    marketPubkey?: string,
    limit?: number,
    cursor?: string,
    fills?: number,
  ): Promise<UserOrderFillsResponse> {
    const url = orderFillsUrl(`${this.client.http.baseUrl()}/api/users/order-fills`, {
      marketPubkey,
      limit,
      cursor,
      fills,
    });
    return userOrderFillsResponseFromWire(
      await this.client.http.get<unknown>(url, RetryPolicy.Idempotent),
    );
  }

  /**
   * Same as {@link getUserOrderFills}, but uses the supplied `cookieHeader`
   * for this call instead of the SDK's process-wide cookie store. For
   * server-side cookie forwarding (SSR / route handlers).
   */
  async getUserOrderFillsWithCookies(
    marketPubkey: string | undefined,
    limit: number | undefined,
    cursor: string | undefined,
    fills: number | undefined,
    cookieHeader: string,
  ): Promise<UserOrderFillsResponse> {
    const url = orderFillsUrl(`${this.client.http.baseUrl()}/api/users/order-fills`, {
      marketPubkey,
      limit,
      cursor,
      fills,
    });
    return userOrderFillsResponseFromWire(
      await this.client.http.getWithCookies<unknown>(url, RetryPolicy.Idempotent, cookieHeader),
    );
  }

  /**
   * Public variant of {@link getUserOrderFills}. Takes the user's wallet
   * via the URL path (`GET /api/users/{wallet}/order-fills`) and requires
   * no auth.
   */
  async getUserOrderFillsByWallet(
    walletAddress: string,
    marketPubkey?: string,
    limit?: number,
    cursor?: string,
    fills?: number,
  ): Promise<UserOrderFillsResponse> {
    const url = orderFillsUrl(
      `${this.client.http.baseUrl()}/api/users/${encodeURIComponent(walletAddress)}/order-fills`,
      { marketPubkey, limit, cursor, fills },
    );
    return userOrderFillsResponseFromWire(
      await this.client.http.get<unknown>(url, RetryPolicy.Idempotent),
    );
  }

  /**
   * Fetch one order of the authenticated user with the next page of its
   * fills. Pass the order's `fills_next_cursor` (or a submission's
   * `fills_next_cursor`) as `fillCursor`; `undefined` starts at the first
   * fill. `fills` is the page size (default and maximum 16; a `fillCursor`
   * needs at least 1). `orderHash` must be 64 lowercase hex characters.
   */
  async getOrderFillPage(
    orderHash: string,
    fillCursor?: string,
    fills?: number,
  ): Promise<UserOrderFillsResponse> {
    const url = orderFillsUrl(`${this.client.http.baseUrl()}/api/users/order-fills`, {
      orderHash,
      fillCursor,
      fills,
    });
    return userOrderFillsResponseFromWire(
      await this.client.http.get<unknown>(url, RetryPolicy.Idempotent),
    );
  }

  /**
   * Same as {@link getOrderFillPage}, forwarding the supplied raw `Cookie`
   * header for server-side cookie forwarding.
   */
  async getOrderFillPageWithCookies(
    orderHash: string,
    fillCursor: string | undefined,
    fills: number | undefined,
    cookieHeader: string,
  ): Promise<UserOrderFillsResponse> {
    const url = orderFillsUrl(`${this.client.http.baseUrl()}/api/users/order-fills`, {
      orderHash,
      fillCursor,
      fills,
    });
    return userOrderFillsResponseFromWire(
      await this.client.http.getWithCookies<unknown>(url, RetryPolicy.Idempotent, cookieHeader),
    );
  }

  /** Public variant of {@link getOrderFillPage} for any wallet. */
  async getOrderFillPageByWallet(
    walletAddress: string,
    orderHash: string,
    fillCursor?: string,
    fills?: number,
  ): Promise<UserOrderFillsResponse> {
    const url = orderFillsUrl(
      `${this.client.http.baseUrl()}/api/users/${encodeURIComponent(walletAddress)}/order-fills`,
      { orderHash, fillCursor, fills },
    );
    return userOrderFillsResponseFromWire(
      await this.client.http.get<unknown>(url, RetryPolicy.Idempotent),
    );
  }

  // ── Unified cancel (dispatches based on client signing strategy) ────

  async cancelOrderSigned(
    orderHash: string,
    maker: PubkeyStr
  ): Promise<CancelSuccess> {
    const strategy = requireSigningStrategy(this.client);

    switch (strategy.type) {
      case "native": {
        const body = cancelBodySigned(orderHash, maker, strategy.keypair);
        return this.cancel(body);
      }
      case "walletAdapter": {
        const message = cancelOrderMessage(orderHash);
        const sigBytes = await strategy.signer
          .signMessage(message)
          .catch((err: unknown) => {
            const msg = err instanceof Error ? err.message : String(err);
            if (isUserCancellation(msg)) throw SdkError.userCancelled();
            throw SdkError.signing(msg);
          });
        const sigBs58 = bs58.encode(sigBytes);
        const body = cancelBodyFromBase58(orderHash, maker, sigBs58);
        return this.cancel(body);
      }
      case "privy":
        throw SdkError.validation(
          "Privy order signing is not supported; configure a native signer or an ExternalSigner"
        );
    }
  }

  async cancelAllSigned(
    userPubkey: PubkeyStr,
    timestamp: number,
    salt: string,
    orderbookId?: OrderBookId
  ): Promise<CancelAllSuccess> {
    const strategy = requireSigningStrategy(this.client);
    const resolvedOrderbookId = orderbookId ?? ("" as OrderBookId);

    switch (strategy.type) {
      case "native": {
        const body = cancelAllBodySigned(
          userPubkey,
          resolvedOrderbookId,
          timestamp,
          salt,
          strategy.keypair
        );
        return this.cancelAll(body);
      }
      case "walletAdapter": {
        const message = cancelAllMessage(
          userPubkey as string,
          resolvedOrderbookId as string,
          timestamp,
          salt
        );
        const sigBytes = await strategy.signer
          .signMessage(new TextEncoder().encode(message))
          .catch((err: unknown) => {
            const msg = err instanceof Error ? err.message : String(err);
            if (isUserCancellation(msg)) throw SdkError.userCancelled();
            throw SdkError.signing(msg);
          });
        const sigBs58 = bs58.encode(sigBytes);
        const body = cancelAllBodyFromBase58(
          userPubkey,
          resolvedOrderbookId,
          timestamp,
          salt,
          sigBs58
        );
        return this.cancelAll(body);
      }
      case "privy":
        throw SdkError.validation(
          "Privy order signing is not supported; configure a native signer or an ExternalSigner"
        );
    }
  }

  // ── On-chain transaction builders ────────────────────────────────────

  cancelOrderIx(
    operator: PublicKey,
    market: PublicKey,
    order: SignedOrder
  ): TransactionInstruction {
    return buildCancelOrderIx(operator, market, order, this.client.programId);
  }

  closeOrderStatusIx(params: CloseOrderStatusParams): TransactionInstruction {
    return buildCloseOrderStatusIx(params, this.client.programId);
  }

  // ── Transaction builders (_tx convenience wrappers) ─────────────────

  cancelOrderTx(
    operator: PublicKey,
    market: PublicKey,
    order: SignedOrder,
    context: V1TransactionContext
  ): V1Transaction {
    const ix = this.cancelOrderIx(operator, market, order);
    return V1Transaction.compile([ix], operator, context);
  }

  closeOrderStatusTx(
    params: CloseOrderStatusParams,
    context: V1TransactionContext
  ): V1Transaction {
    const ix = this.closeOrderStatusIx(params);
    return V1Transaction.compile([ix], params.operator, context);
  }

  // ── Order helpers ────────────────────────────────────────────────────

  createBidOrder(params: BidOrderParams): Omit<SignedOrder, "signature"> {
    return programCreateBidOrder(params);
  }

  createAskOrder(params: AskOrderParams): Omit<SignedOrder, "signature"> {
    return programCreateAskOrder(params);
  }

  createSignedBidOrder(
    params: BidOrderParams,
    signer: Keypair,
    rules: OrderbookRules
  ): SignedOrder {
    return programCreateSignedBidOrder(params, signer, rules);
  }

  createSignedAskOrder(
    params: AskOrderParams,
    signer: Keypair,
    rules: OrderbookRules
  ): SignedOrder {
    return programCreateSignedAskOrder(params, signer, rules);
  }

  hashOrder(order: SignedOrder): Buffer {
    return programHashOrder(order);
  }

  signOrder(
    order: SignedOrder,
    signer: Keypair,
    rules: OrderbookRules
  ): Buffer {
    return programSignOrder(order, signer, rules);
  }

  signFullOrder(
    order: Omit<SignedOrder, "signature">,
    signer: Keypair,
    rules: OrderbookRules
  ): SignedOrder {
    return signOrderFull(order, signer, rules);
  }

  // ── On-chain account fetchers (require Connection) ──────────────────

  async getStatus(orderHash: Buffer): Promise<ProgramOrderStatus | null> {
    const connection = requireConnection(this.client);
    const pda = this.statusPda(orderHash);
    const accountInfo = await connection.getAccountInfo(pda);
    if (!accountInfo) {
      return null;
    }
    return deserializeOrderStatus(accountInfo.data as Buffer);
  }
}

function buildUserOrdersAuthenticatedUrl(
  baseUrl: string,
  limit: number | undefined,
  cursor: string | undefined,
): string {
  const params = new URLSearchParams();
  if (limit !== undefined) params.set("limit", String(limit));
  if (cursor !== undefined) params.set("cursor", cursor);
  const qs = params.toString();
  return qs.length === 0
    ? `${baseUrl}/api/users/orders`
    : `${baseUrl}/api/users/orders?${qs}`;
}
