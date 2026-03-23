import bs58 from "bs58";
import { Keypair, Transaction, type PublicKey, type TransactionInstruction } from "@solana/web3.js";
import type { ClientContext } from "../../context";
import { requireConnection, requireSigningStrategy } from "../../context";
import { SdkError } from "../../error";
import { ProgramSdkError } from "../../program/error";
import { RetryPolicy } from "../../http";
import { Privy } from "../../privy";
import { isUserCancellation } from "../../shared/signing";
import {
  buildCancelOrderIx,
  buildIncrementNonceIx,
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
  signCancelTriggerOrder,
  signCancelAll,
  cancelOrderMessage,
  cancelTriggerOrderMessage,
  cancelAllMessage,
  generateCancelAllSalt as programGenerateCancelAllSalt,
} from "../../program/orders";
import {
  getOrderStatusPda,
  getUserNoncePda,
} from "../../program/pda";
import {
  deserializeOrderStatus,
  deserializeUserNonce,
} from "../../program/accounts";
import type {
  SignedOrder,
  BidOrderParams,
  AskOrderParams,
  OrderStatus as ProgramOrderStatus,
} from "../../program/types";
import { asOrderBookId, asPubkeyStr, type OrderBookId, type PubkeyStr } from "../../shared";
import { LimitOrderEnvelope, TriggerOrderEnvelope } from "../../program/envelope";
import type { UserSnapshotBalance, UserSnapshotOrder } from "./wire";

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

export interface CancelTriggerBody {
  trigger_order_id: string;
  maker: PubkeyStr;
  signature: string;
}

export function cancelTriggerBodyFromBase58(
  triggerOrderId: string,
  maker: PubkeyStr,
  signatureBase58: string
): CancelTriggerBody {
  return {
    trigger_order_id: triggerOrderId,
    maker,
    signature: Buffer.from(bs58.decode(signatureBase58)).toString("hex"),
  };
}

export function cancelTriggerBodySigned(
  triggerOrderId: string,
  maker: PubkeyStr,
  keypair: Keypair
): CancelTriggerBody {
  const signature = signCancelTriggerOrder(triggerOrderId, keypair);
  return { trigger_order_id: triggerOrderId, maker, signature };
}

// ─── Response types ──────────────────────────────────────────────────────────

export interface FillInfo {
  counterparty: PubkeyStr;
  counterparty_order_hash: string;
  fill_amount: string;
  price: string;
  is_maker: boolean;
}

export interface SubmitOrderResponse {
  order_hash: string;
  remaining: string;
  filled: string;
  fills: FillInfo[];
}

export type PlaceResponse =
  | ({ status: "accepted" } & SubmitOrderResponse)
  | ({ status: "partial_fill" } & SubmitOrderResponse)
  | ({ status: "filled" } & SubmitOrderResponse)
  | {
      status: "rejected";
      error?: string;
      details?: string;
      reason?: string;
      order_hash?: string;
      remaining?: string;
      filled?: string;
    }
  | { status: "error"; error?: string }
  | { status: "bad_request"; error?: string; details?: string }
  | { status: "not_found"; error?: string }
  | { status: "forbidden"; error?: string }
  | { status: "internal_error"; error?: string; details?: string }
  | { status: "configuration_error"; error?: string };

export interface CancelSuccess {
  order_hash: string;
  remaining: number;
}

export type CancelResponse =
  | { status: "cancelled"; order_hash: string; remaining: number }
  | { status: "error"; error: string }
  | { status: "bad_request"; error: string }
  | { status: "not_found"; error: string }
  | { status: "forbidden"; error: string }
  | { status: "internal_error"; error: string; details?: string };

export interface CancelAllSuccess {
  cancelled_order_hashes: string[];
  count: number;
  user_pubkey: PubkeyStr;
  orderbook_id: OrderBookId;
  message: string;
}

export type CancelAllResponse =
  | ({ status: "success" } & CancelAllSuccess)
  | { status: "error"; message: string }
  | { status: "bad_request"; error: string }
  | { status: "not_found"; error: string }
  | { status: "forbidden"; error: string }
  | { status: "internal_error"; error: string; details?: string };

export interface TriggerOrderResponse {
  trigger_order_id: string;
  order_hash: string;
}

export type TriggerSubmitResponse =
  | ({ status: "accepted" } & TriggerOrderResponse)
  | { status: "error"; error: string }
  | { status: "bad_request"; error: string }
  | { status: "not_found"; error: string }
  | { status: "forbidden"; error: string }
  | { status: "internal_error"; error: string; details?: string };

export interface CancelTriggerSuccess {
  trigger_order_id: string;
}

export type CancelTriggerResponse =
  | ({ status: "cancelled" } & CancelTriggerSuccess)
  | { status: "error"; error: string }
  | { status: "bad_request"; error: string }
  | { status: "not_found"; error: string }
  | { status: "forbidden"; error: string }
  | { status: "internal_error"; error: string; details?: string };

export interface UserOrdersResponse {
  user_pubkey: PubkeyStr;
  orders: UserSnapshotOrder[];
  balances: UserSnapshotBalance[];
  next_cursor?: string;
  has_more: boolean;
}

// ─── Sub-client ──────────────────────────────────────────────────────────────

export class Orders {
  constructor(private readonly client: ClientContext) {}

  // ── PDA helpers ──────────────────────────────────────────────────────

  statusPda(orderHash: Buffer): PublicKey {
    return getOrderStatusPda(orderHash, this.client.programId)[0];
  }

  noncePda(user: PublicKey): PublicKey {
    return getUserNoncePda(user, this.client.programId)[0];
  }

  // ── Envelope factories ────────────────────────────────────────────────

  limitOrder(): LimitOrderEnvelope {
    return LimitOrderEnvelope.new().depositSource(this.client.depositSource);
  }

  triggerOrder(): TriggerOrderEnvelope {
    return TriggerOrderEnvelope.new().depositSource(this.client.depositSource);
  }

  // ── Helpers ──────────────────────────────────────────────────────────

  generateCancelAllSalt(): string {
    return programGenerateCancelAllSalt();
  }

  // ── HTTP methods ─────────────────────────────────────────────────────

  async submit(request: object): Promise<SubmitOrderResponse> {
    const url = `${this.client.http.baseUrl()}/api/orders/submit`;
    const raw = await this.client.http.post<PlaceResponse, object>(url, request, RetryPolicy.None);

    switch (raw.status) {
      case "accepted":
      case "partial_fill":
      case "filled":
        return raw;
      case "rejected": {
        const message = [raw.error, raw.details].filter(Boolean).join(": ") || "Rejected";
        throw SdkError.from(new Error(message));
      }
      case "bad_request":
      case "internal_error":
        throw SdkError.from(new Error([raw.error, raw.details].filter(Boolean).join(": ")));
      case "error":
      case "not_found":
      case "forbidden":
      case "configuration_error":
        throw SdkError.from(new Error(raw.error || "Unknown error"));
      default:
        throw SdkError.from(new Error("Unknown submit response"));
    }
  }

  async cancel(body: CancelBody): Promise<CancelSuccess> {
    const url = `${this.client.http.baseUrl()}/api/orders/cancel`;
    const response = await this.client.http.post<CancelResponse, CancelBody>(url, body, RetryPolicy.None);

    if (response.status === "cancelled") {
      return { order_hash: response.order_hash, remaining: response.remaining };
    }

    if (response.status === "internal_error") {
      throw SdkError.from(new Error([response.error, response.details].filter(Boolean).join(": ")));
    }

    throw SdkError.from(new Error(response.error));
  }

  async cancelAll(body: CancelAllBody): Promise<CancelAllSuccess> {
    const url = `${this.client.http.baseUrl()}/api/orders/cancel-all`;
    const response = await this.client.http.post<CancelAllResponse, CancelAllBody>(
      url,
      body,
      RetryPolicy.None
    );

    if (response.status === "success") {
      return response;
    }

    if (response.status === "error") {
      throw SdkError.from(new Error(response.message));
    }

    if (response.status === "internal_error") {
      throw SdkError.from(new Error([response.error, response.details].filter(Boolean).join(": ")));
    }

    throw SdkError.from(new Error(response.error));
  }

  async submitTrigger(request: object): Promise<TriggerOrderResponse> {
    const url = `${this.client.http.baseUrl()}/api/orders/submit`;
    const response = await this.client.http.post<TriggerSubmitResponse, object>(
      url,
      request,
      RetryPolicy.None
    );

    if (response.status === "accepted") {
      return response;
    }

    if (response.status === "internal_error") {
      throw SdkError.from(new Error([response.error, response.details].filter(Boolean).join(": ")));
    }

    throw SdkError.from(new Error(response.error));
  }

  async cancelTrigger(body: CancelTriggerBody): Promise<CancelTriggerSuccess> {
    const url = `${this.client.http.baseUrl()}/api/orders/cancel`;
    const response = await this.client.http.post<CancelTriggerResponse, CancelTriggerBody>(
      url,
      body,
      RetryPolicy.None
    );

    if (response.status === "cancelled") {
      return response;
    }

    if (response.status === "internal_error") {
      throw SdkError.from(new Error([response.error, response.details].filter(Boolean).join(": ")));
    }

    throw SdkError.from(new Error(response.error));
  }

  async getUserOrders(
    walletAddress: string,
    limit?: number,
    cursor?: string
  ): Promise<UserOrdersResponse> {
    const params = new URLSearchParams({ wallet_address: walletAddress });
    if (limit !== undefined) params.set("limit", String(limit));
    if (cursor) params.set("cursor", cursor);

    const url = `${this.client.http.baseUrl()}/api/users/orders?${params.toString()}`;
    const response = await this.client.http.get<{
      user_pubkey: PubkeyStr;
      orders?: UserSnapshotOrder[];
      balances?: UserSnapshotBalance[];
      next_cursor?: string | null;
      has_more?: boolean;
    }>(url, RetryPolicy.Idempotent);

    return {
      user_pubkey: response.user_pubkey,
      orders: response.orders ?? [],
      balances: response.balances ?? [],
      next_cursor: response.next_cursor ?? undefined,
      has_more: response.has_more ?? false,
    };
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
      case "privy": {
        const privy = new Privy(this.client);
        const result = await privy.signAndCancelOrder(
          strategy.walletId,
          orderHash,
          maker as string
        );
        if ("order_hash" in result) {
          return { order_hash: result.order_hash, remaining: Number(result.remaining ?? 0) };
        }
        throw SdkError.from(new Error("Unexpected cancel response"));
      }
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
      case "privy": {
        const privy = new Privy(this.client);
        const result = await privy.signAndCancelAllOrders(
          strategy.walletId,
          userPubkey as string,
          resolvedOrderbookId as string,
          timestamp,
          salt
        );
        return {
          cancelled_order_hashes: result.cancelled_order_hashes,
          count: result.count,
          user_pubkey: asPubkeyStr(result.user_pubkey),
          orderbook_id: asOrderBookId(result.orderbook_id),
          message: result.message,
        };
      }
    }
  }

  async cancelTriggerSigned(
    triggerOrderId: string,
    maker: PubkeyStr
  ): Promise<CancelTriggerSuccess> {
    const strategy = requireSigningStrategy(this.client);

    switch (strategy.type) {
      case "native": {
        const body = cancelTriggerBodySigned(triggerOrderId, maker, strategy.keypair);
        return this.cancelTrigger(body);
      }
      case "walletAdapter": {
        const message = cancelTriggerOrderMessage(triggerOrderId);
        const sigBytes = await strategy.signer
          .signMessage(message)
          .catch((err: unknown) => {
            const msg = err instanceof Error ? err.message : String(err);
            if (isUserCancellation(msg)) throw SdkError.userCancelled();
            throw SdkError.signing(msg);
          });
        const sigBs58 = bs58.encode(sigBytes);
        const body = cancelTriggerBodyFromBase58(triggerOrderId, maker, sigBs58);
        return this.cancelTrigger(body);
      }
      case "privy": {
        const privy = new Privy(this.client);
        const result = await privy.signAndCancelTriggerOrder(
          strategy.walletId,
          triggerOrderId,
          maker as string
        );
        if ("trigger_order_id" in result) {
          return { trigger_order_id: result.trigger_order_id };
        }
        throw SdkError.from(new Error("Unexpected cancel-trigger response"));
      }
    }
  }

  // ── On-chain transaction builders ────────────────────────────────────

  cancelOrderIx(
    maker: PublicKey,
    market: PublicKey,
    order: SignedOrder
  ): TransactionInstruction {
    return buildCancelOrderIx(maker, market, order, this.client.programId);
  }

  incrementNonceIx(user: PublicKey): TransactionInstruction {
    return buildIncrementNonceIx(user, this.client.programId);
  }

  // ── Transaction builders (_tx convenience wrappers) ─────────────────

  cancelOrderTx(
    maker: PublicKey,
    market: PublicKey,
    order: SignedOrder
  ): Transaction {
    const ix = this.cancelOrderIx(maker, market, order);
    return new Transaction({ feePayer: maker }).add(ix);
  }

  incrementNonceTx(user: PublicKey): Transaction {
    const ix = this.incrementNonceIx(user);
    return new Transaction({ feePayer: user }).add(ix);
  }

  // ── Order helpers ────────────────────────────────────────────────────

  createBidOrder(params: BidOrderParams): Omit<SignedOrder, "signature"> {
    return programCreateBidOrder(params);
  }

  createAskOrder(params: AskOrderParams): Omit<SignedOrder, "signature"> {
    return programCreateAskOrder(params);
  }

  createSignedBidOrder(params: BidOrderParams, signer: Keypair): SignedOrder {
    return programCreateSignedBidOrder(params, signer);
  }

  createSignedAskOrder(params: AskOrderParams, signer: Keypair): SignedOrder {
    return programCreateSignedAskOrder(params, signer);
  }

  hashOrder(order: SignedOrder): Buffer {
    return programHashOrder(order);
  }

  signOrder(order: SignedOrder, signer: Keypair): Buffer {
    return programSignOrder(order, signer);
  }

  signFullOrder(
    order: Omit<SignedOrder, "signature">,
    signer: Keypair
  ): SignedOrder {
    return signOrderFull(order, signer);
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

  async getNonce(user: PublicKey): Promise<bigint> {
    const connection = requireConnection(this.client);
    const pda = this.noncePda(user);
    const accountInfo = await connection.getAccountInfo(pda);
    if (!accountInfo) {
      return 0n;
    }
    const userNonce = deserializeUserNonce(accountInfo.data as Buffer);
    return userNonce.nonce;
  }

  async currentNonce(user: PublicKey): Promise<number> {
    const nonce = await this.getNonce(user);
    if (nonce > 0xFFFFFFFFn) {
      throw ProgramSdkError.overflow(`Nonce exceeds u32 range: ${nonce}`);
    }
    return Number(nonce);
  }
}
