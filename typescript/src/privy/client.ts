import { RetryPolicy, type LightconeHttp } from "../http";
import type {
  ExportWalletRequest,
  ExportWalletResponse,
  PrivyOrderEnvelope,
  SignAndCancelAllRequest,
  SignAndCancelOrderRequest,
  SignAndSendOrderRequest,
  SignAndSendTxRequest,
  SignAndSendTxResponse,
} from "./index";

interface ClientContext {
  http: LightconeHttp;
}

export class Privy {
  constructor(private readonly client: ClientContext) {}

  async signAndSendTx(walletId: string, base64Tx: string): Promise<SignAndSendTxResponse> {
    const url = `${this.client.http.baseUrl()}/api/privy/sign_and_send_tx`;
    const body: SignAndSendTxRequest = {
      wallet_id: walletId,
      base64_tx: base64Tx,
    };
    return this.client.http.post<SignAndSendTxResponse, SignAndSendTxRequest>(
      url,
      body,
      RetryPolicy.None
    );
  }

  async signAndSendOrder(walletId: string, order: PrivyOrderEnvelope): Promise<unknown> {
    const url = `${this.client.http.baseUrl()}/api/privy/sign_and_send_order`;
    const body: SignAndSendOrderRequest = {
      wallet_id: walletId,
      order,
    };
    return this.client.http.post<unknown, SignAndSendOrderRequest>(url, body, RetryPolicy.None);
  }

  async signAndCancelOrder(walletId: string, orderHash: string, maker: string): Promise<unknown> {
    const url = `${this.client.http.baseUrl()}/api/privy/sign_and_cancel_order`;
    const body: SignAndCancelOrderRequest = {
      wallet_id: walletId,
      maker,
      cancel_type: "limit",
      order_hash: orderHash,
    };
    return this.client.http.post<unknown, SignAndCancelOrderRequest>(url, body, RetryPolicy.None);
  }

  async signAndCancelTriggerOrder(
    walletId: string,
    triggerOrderId: string,
    maker: string
  ): Promise<unknown> {
    const url = `${this.client.http.baseUrl()}/api/privy/sign_and_cancel_order`;
    const body: SignAndCancelOrderRequest = {
      wallet_id: walletId,
      maker,
      cancel_type: "trigger",
      trigger_order_id: triggerOrderId,
    };
    return this.client.http.post<unknown, SignAndCancelOrderRequest>(url, body, RetryPolicy.None);
  }

  async signAndCancelAllOrders(
    walletId: string,
    userPubkey: string,
    orderbookId: string = "",
    timestamp: number
  ): Promise<unknown> {
    const url = `${this.client.http.baseUrl()}/api/privy/sign_and_cancel_all_orders`;
    const body: SignAndCancelAllRequest = {
      wallet_id: walletId,
      user_pubkey: userPubkey,
      orderbook_id: orderbookId,
      timestamp,
    };
    return this.client.http.post<unknown, SignAndCancelAllRequest>(url, body, RetryPolicy.None);
  }

  async exportWallet(walletId: string, decodePubkeyBase64: string): Promise<ExportWalletResponse> {
    const url = `${this.client.http.baseUrl()}/api/privy/wallet/export`;
    const body: ExportWalletRequest = {
      wallet_id: walletId,
      decode_pubkey_base64: decodePubkeyBase64,
    };
    return this.client.http.post<ExportWalletResponse, ExportWalletRequest>(
      url,
      body,
      RetryPolicy.None
    );
  }
}
