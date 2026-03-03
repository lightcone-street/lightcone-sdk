import bs58 from "bs58";
import { SdkError } from "../../error";
import { RetryPolicy, type LightconeHttp } from "../../http";
import type { OrderBookId, PubkeyStr } from "../../shared";
import type { UserSnapshotBalance, UserSnapshotOrder } from "./wire";

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

export interface CancelAllBody {
  user_pubkey: PubkeyStr;
  orderbook_id: OrderBookId;
  signature: string;
  timestamp: number;
}

export function cancelAllBodyFromBase58(
  userPubkey: PubkeyStr,
  orderbookId: OrderBookId,
  timestamp: number,
  signatureBase58: string
): CancelAllBody {
  return {
    user_pubkey: userPubkey,
    orderbook_id: orderbookId,
    signature: Buffer.from(bs58.decode(signatureBase58)).toString("hex"),
    timestamp,
  };
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

interface ClientContext {
  http: LightconeHttp;
}

export class Orders {
  constructor(private readonly client: ClientContext) {}

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
    return this.client.http.get<UserOrdersResponse>(url, RetryPolicy.Idempotent);
  }
}
