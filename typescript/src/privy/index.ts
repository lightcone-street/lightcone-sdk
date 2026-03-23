export * from "./client";
import type { LimitOrderEnvelope, TriggerOrderEnvelope } from "../program/envelope";

export interface SignAndSendTxRequest {
  wallet_id: string;
  base64_tx: string;
}

export interface SignAndSendTxResponse {
  hash: string;
}

export interface PrivyOrderEnvelope {
  maker: string;
  nonce: number;
  salt: number;
  market_pubkey: string;
  base_token: string;
  quote_token: string;
  side: number;
  amount_in: number;
  amount_out: number;
  expiration?: number;
  orderbook_id: string;
  tif?: import("../shared").TimeInForce;
  trigger_price?: number;
  trigger_type?: import("../shared").TriggerType;
  deposit_source?: import("../shared").DepositSource;
}

export interface SignAndSendOrderRequest {
  wallet_id: string;
  order: PrivyOrderEnvelope;
}

export type CancelTarget =
  | { cancel_type: "limit"; order_hash: string }
  | { cancel_type: "trigger"; trigger_order_id: string };

export type SignAndCancelOrderRequest = {
  wallet_id: string;
  maker: string;
} & CancelTarget;

export interface SignAndCancelAllRequest {
  wallet_id: string;
  user_pubkey: string;
  orderbook_id: string;
  timestamp: number;
  salt: string;
}

export interface ExportWalletRequest {
  wallet_id: string;
  decode_pubkey_base64: string;
}

export interface ExportWalletResponse {
  encryption_type: string;
  ciphertext: string;
  encapsulated_key: string;
}

export interface OrderFill {
  counterparty: string;
  counterparty_order_hash: string;
  fill_amount: string;
  price: string;
  is_maker: boolean;
}

export interface LimitOrderResponse {
  order_hash: string;
  status: string;
  remaining: string;
  filled: string;
  fills: OrderFill[];
}

export interface TriggerOrderResponse {
  trigger_order_id: string;
  order_hash: string;
  status: string;
}

export type SignAndSendOrderResponse = LimitOrderResponse | TriggerOrderResponse;

export interface LimitCancelResponse {
  status: string;
  order_hash: string;
  remaining: string;
}

export interface TriggerCancelResponse {
  status: string;
  trigger_order_id: string;
}

export type SignAndCancelOrderResponse = LimitCancelResponse | TriggerCancelResponse;

export interface SignAndCancelAllResponse {
  status: string;
  user_pubkey: string;
  orderbook_id: string;
  cancelled_order_hashes: string[];
  count: number;
  message: string;
}

function requireDefined<T>(
  value: T | undefined,
  field: string
): T {
  if (value === undefined) {
    throw new Error(`Missing required field: ${field}`);
  }
  return value;
}

function bigintToSafeNumber(value: bigint, field: string): number {
  const max = BigInt(Number.MAX_SAFE_INTEGER);
  if (value > max || value < -max) {
    throw new Error(`${field} exceeds Number.MAX_SAFE_INTEGER`);
  }
  return Number(value);
}

export function privyOrderFromLimitEnvelope(
  envelope: LimitOrderEnvelope,
  orderbookId: string
): PrivyOrderEnvelope {
  const maker = requireDefined(envelope.getMaker(), "maker");
  const nonce = requireDefined(envelope.getNonce(), "nonce");
  const salt = requireDefined(envelope.getSalt(), "salt");
  const market = requireDefined(envelope.getMarket(), "market");
  const baseMint = requireDefined(envelope.getBaseMint(), "base_mint");
  const quoteMint = requireDefined(envelope.getQuoteMint(), "quote_mint");
  const side = requireDefined(envelope.getSide(), "side");
  const amountIn = requireDefined(envelope.getAmountIn(), "amount_in");
  const amountOut = requireDefined(envelope.getAmountOut(), "amount_out");

  return {
    maker: maker.toBase58(),
    nonce,
    salt: bigintToSafeNumber(salt, "salt"),
    market_pubkey: market.toBase58(),
    base_token: baseMint.toBase58(),
    quote_token: quoteMint.toBase58(),
    side,
    amount_in: bigintToSafeNumber(amountIn, "amount_in"),
    amount_out: bigintToSafeNumber(amountOut, "amount_out"),
    expiration: bigintToSafeNumber(envelope.getExpiration(), "expiration"),
    orderbook_id: orderbookId,
    deposit_source: envelope.getDepositSource(),
  };
}

export function privyOrderFromTriggerEnvelope(
  envelope: TriggerOrderEnvelope,
  orderbookId: string
): PrivyOrderEnvelope {
  const maker = requireDefined(envelope.getMaker(), "maker");
  const nonce = requireDefined(envelope.getNonce(), "nonce");
  const salt = requireDefined(envelope.getSalt(), "salt");
  const market = requireDefined(envelope.getMarket(), "market");
  const baseMint = requireDefined(envelope.getBaseMint(), "base_mint");
  const quoteMint = requireDefined(envelope.getQuoteMint(), "quote_mint");
  const side = requireDefined(envelope.getSide(), "side");
  const amountIn = requireDefined(envelope.getAmountIn(), "amount_in");
  const amountOut = requireDefined(envelope.getAmountOut(), "amount_out");

  return {
    maker: maker.toBase58(),
    nonce,
    salt: bigintToSafeNumber(salt, "salt"),
    market_pubkey: market.toBase58(),
    base_token: baseMint.toBase58(),
    quote_token: quoteMint.toBase58(),
    side,
    amount_in: bigintToSafeNumber(amountIn, "amount_in"),
    amount_out: bigintToSafeNumber(amountOut, "amount_out"),
    expiration: bigintToSafeNumber(envelope.getExpiration(), "expiration"),
    orderbook_id: orderbookId,
    tif: envelope.getTimeInForce(),
    trigger_price: envelope.getTriggerPrice(),
    trigger_type: envelope.getTriggerType(),
    deposit_source: envelope.getDepositSource(),
  };
}
