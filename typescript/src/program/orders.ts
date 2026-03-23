import { PublicKey, Keypair } from "@solana/web3.js";
import { sign } from "tweetnacl";
import bs58 from "bs58";
import type {
  DepositSource,
  SubmitOrderRequest,
  TimeInForce,
  TriggerType,
} from "../shared";
import {
  SignedOrder,
  Order,
  OrderSide,
  BidOrderParams,
  AskOrderParams,
} from "./types";
import { ORDER_SIZE } from "./constants";
import {
  keccak256,
  toU64Le,
  toI64Le,
  toU8,
  toU32Le,
  fromLeBytes,
  fromI64Le,
  deriveConditionId,
} from "./utils";

function bigintToSafeNumber(value: bigint, field: string): number {
  const max = BigInt(Number.MAX_SAFE_INTEGER);
  if (value > max || value < -max) {
    throw new Error(`${field} exceeds Number.MAX_SAFE_INTEGER`);
  }
  return Number(value);
}

/**
 * Generate a random salt for order uniqueness.
 * Capped to Number.MAX_SAFE_INTEGER (2^53 - 1) so the value round-trips
 * through JSON without precision loss.
 */
export function generateSalt(): bigint {
  const buf = new Uint8Array(8);
  globalThis.crypto.getRandomValues(buf);
  const raw = fromLeBytes(Buffer.from(buf));
  return raw % (BigInt(Number.MAX_SAFE_INTEGER) + 1n);
}


// ============================================================================
// ORDER HASHING
// ============================================================================

/**
 * Hash an order using keccak256
 * Layout (169 bytes - order without signature):
 * nonce (8) || salt (8) || maker (32) || market (32) || baseMint (32) || quoteMint (32) ||
 * side (1) || amountIn (8) || amountOut (8) || expiration (8)
 *
 * @returns 32-byte keccak256 hash
 */
export function hashOrder(order: SignedOrder): Buffer {
  const data = Buffer.concat([
    toU64Le(BigInt(order.nonce)),
    toU64Le(order.salt),
    order.maker.toBuffer(),
    order.market.toBuffer(),
    order.baseMint.toBuffer(),
    order.quoteMint.toBuffer(),
    toU8(order.side),
    toU64Le(order.amountIn),
    toU64Le(order.amountOut),
    toI64Le(order.expiration),
  ]);

  return keccak256(data);
}

/**
 * Get the hex-encoded hash of an order
 */
export function hashOrderHex(order: SignedOrder): string {
  return hashOrder(order).toString("hex");
}

/**
 * Get the message to sign for an order (the order hash)
 */
export function getOrderMessage(order: SignedOrder): Buffer {
  return hashOrder(order);
}

// ============================================================================
// ORDER SIGNING
// ============================================================================

/**
 * Sign an order with a Keypair.
 * Signs the hex-encoded keccak hash (64-char ASCII string) for cross-compatibility with Rust.
 * Returns 64-byte Ed25519 signature.
 */
export function signOrder(order: SignedOrder, signer: Keypair): Buffer {
  const hash = hashOrder(order);
  const hexString = hash.toString("hex");
  const messageBytes = Buffer.from(hexString, "ascii");
  const signature = sign.detached(messageBytes, signer.secretKey);
  return Buffer.from(signature);
}

/**
 * Sign an order and return a new order with the signature attached
 */
export function signOrderFull(
  order: Omit<SignedOrder, "signature">,
  signer: Keypair
): SignedOrder {
  const orderWithEmptySig: SignedOrder = {
    ...order,
    signature: Buffer.alloc(64),
  };
  const signature = signOrder(orderWithEmptySig, signer);
  return {
    ...order,
    signature,
  };
}

/**
 * Verify an order's signature.
 * Verifies against the hex-encoded keccak hash.
 */
export function verifyOrderSignature(order: SignedOrder): boolean {
  const hash = hashOrder(order);
  const hexString = hash.toString("hex");
  const messageBytes = Buffer.from(hexString, "ascii");
  return sign.detached.verify(
    messageBytes,
    order.signature,
    order.maker.toBytes()
  );
}

// ============================================================================
// SIGNED ORDER SERIALIZATION (225 bytes)
// ============================================================================

/**
 * Serialize a signed order to bytes (233 bytes)
 *
 * Layout:
 * [0..8]     nonce (u64)
 * [8..16]    salt (u64)
 * [16..48]   maker (Pubkey)
 * [48..80]   market (Pubkey)
 * [80..112]  baseMint (Pubkey)
 * [112..144] quoteMint (Pubkey)
 * [144]      side (u8)
 * [145..153] amountIn (u64)
 * [153..161] amountOut (u64)
 * [161..169] expiration (i64)
 * [169..233] signature (64 bytes)
 */
export function serializeSignedOrder(order: SignedOrder): Buffer {
  const buffer = Buffer.alloc(ORDER_SIZE.SIGNED_ORDER);
  let offset = 0;

  toU64Le(BigInt(order.nonce)).copy(buffer, offset);
  offset += 8;

  toU64Le(order.salt).copy(buffer, offset);
  offset += 8;

  order.maker.toBuffer().copy(buffer, offset);
  offset += 32;

  order.market.toBuffer().copy(buffer, offset);
  offset += 32;

  order.baseMint.toBuffer().copy(buffer, offset);
  offset += 32;

  order.quoteMint.toBuffer().copy(buffer, offset);
  offset += 32;

  buffer[offset] = order.side;
  offset += 1;

  toU64Le(order.amountIn).copy(buffer, offset);
  offset += 8;

  toU64Le(order.amountOut).copy(buffer, offset);
  offset += 8;

  toI64Le(order.expiration).copy(buffer, offset);
  offset += 8;

  order.signature.copy(buffer, offset);

  return buffer;
}

/**
 * Deserialize a signed order from bytes
 */
export function deserializeSignedOrder(data: Buffer): SignedOrder {
  if (data.length < ORDER_SIZE.SIGNED_ORDER) {
    throw new Error(
      `Invalid signed order length: ${data.length}, expected ${ORDER_SIZE.SIGNED_ORDER}`
    );
  }

  let offset = 0;

  const nonceU64 = fromLeBytes(data.subarray(offset, offset + 8));
  if (nonceU64 > 0xFFFFFFFFn) {
    throw new Error(`Nonce exceeds u32 range: ${nonceU64}`);
  }
  const nonce = Number(nonceU64);
  offset += 8;

  const salt = fromLeBytes(data.subarray(offset, offset + 8));
  offset += 8;

  const maker = new PublicKey(data.subarray(offset, offset + 32));
  offset += 32;

  const market = new PublicKey(data.subarray(offset, offset + 32));
  offset += 32;

  const baseMint = new PublicKey(data.subarray(offset, offset + 32));
  offset += 32;

  const quoteMint = new PublicKey(data.subarray(offset, offset + 32));
  offset += 32;

  const side = data[offset] as OrderSide;
  offset += 1;

  const amountIn = fromLeBytes(data.subarray(offset, offset + 8));
  offset += 8;

  const amountOut = fromLeBytes(data.subarray(offset, offset + 8));
  offset += 8;

  const expiration = fromI64Le(data.subarray(offset, offset + 8));
  offset += 8;

  const signature = Buffer.from(data.subarray(offset, offset + 64));

  return {
    nonce,
    salt,
    maker,
    market,
    baseMint,
    quoteMint,
    side,
    amountIn,
    amountOut,
    expiration,
    signature,
  };
}

// ============================================================================
// ORDER SERIALIZATION (29 bytes)
// ============================================================================

/**
 * Serialize a compact order to bytes (37 bytes)
 *
 * Layout:
 * [0..4]    nonce (u32)
 * [4..12]   salt (u64)
 * [12]      side (u8)
 * [13..21]  amountIn (u64)
 * [21..29]  amountOut (u64)
 * [29..37]  expiration (i64)
 */
export function serializeOrder(order: Order): Buffer {
  const buffer = Buffer.alloc(ORDER_SIZE.ORDER);
  let offset = 0;

  toU32Le(order.nonce).copy(buffer, offset);
  offset += 4;

  toU64Le(order.salt).copy(buffer, offset);
  offset += 8;

  buffer[offset] = order.side;
  offset += 1;

  toU64Le(order.amountIn).copy(buffer, offset);
  offset += 8;

  toU64Le(order.amountOut).copy(buffer, offset);
  offset += 8;

  toI64Le(order.expiration).copy(buffer, offset);

  return buffer;
}

/**
 * Deserialize a compact order from bytes
 */
export function deserializeOrder(data: Buffer): Order {
  if (data.length < ORDER_SIZE.ORDER) {
    throw new Error(
      `Invalid order length: ${data.length}, expected ${ORDER_SIZE.ORDER}`
    );
  }

  let offset = 0;

  const nonce = data.readUInt32LE(offset);
  offset += 4;

  const salt = fromLeBytes(data.subarray(offset, offset + 8));
  offset += 8;

  const side = data[offset] as OrderSide;
  offset += 1;

  const amountIn = fromLeBytes(data.subarray(offset, offset + 8));
  offset += 8;

  const amountOut = fromLeBytes(data.subarray(offset, offset + 8));
  offset += 8;

  const expiration = fromI64Le(data.subarray(offset, offset + 8));

  return {
    nonce,
    salt,
    side,
    amountIn,
    amountOut,
    expiration,
  };
}

/**
 * Convert a SignedOrder to a compact Order (drop maker)
 */
export function signedOrderToOrder(order: SignedOrder): Order {
  return {
    nonce: order.nonce,
    salt: order.salt,
    side: order.side,
    amountIn: order.amountIn,
    amountOut: order.amountOut,
    expiration: order.expiration,
  };
}

/**
 * Expand a compact Order back to a full SignedOrder using provided pubkeys and signature.
 * Rust equivalent: Order::to_signed()
 */
export function orderToSigned(
  order: Order,
  maker: PublicKey,
  market: PublicKey,
  baseMint: PublicKey,
  quoteMint: PublicKey,
  signature: Buffer
): SignedOrder {
  return {
    nonce: order.nonce,
    salt: order.salt,
    maker,
    market,
    baseMint,
    quoteMint,
    side: order.side,
    amountIn: order.amountIn,
    amountOut: order.amountOut,
    expiration: order.expiration,
    signature,
  };
}

// ============================================================================
// ORDER CREATION HELPERS
// ============================================================================

/**
 * Create a BID order (buyer wants base tokens, pays with quote tokens)
 */
export function createBidOrder(
  params: BidOrderParams
): Omit<SignedOrder, "signature"> {
  return {
    nonce: params.nonce,
    salt: params.salt ?? generateSalt(),
    maker: params.maker,
    market: params.market,
    baseMint: params.baseMint,
    quoteMint: params.quoteMint,
    side: OrderSide.BID,
    amountIn: params.amountIn,
    amountOut: params.amountOut,
    expiration: params.expiration ?? 0n,
  };
}

/**
 * Create an ASK order (seller offers base tokens, receives quote tokens)
 */
export function createAskOrder(
  params: AskOrderParams
): Omit<SignedOrder, "signature"> {
  return {
    nonce: params.nonce,
    salt: params.salt ?? generateSalt(),
    maker: params.maker,
    market: params.market,
    baseMint: params.baseMint,
    quoteMint: params.quoteMint,
    side: OrderSide.ASK,
    amountIn: params.amountIn,
    amountOut: params.amountOut,
    expiration: params.expiration ?? 0n,
  };
}

/**
 * Create and sign a BID order in one step
 */
export function createSignedBidOrder(
  params: BidOrderParams,
  signer: Keypair
): SignedOrder {
  const order = createBidOrder(params);
  return signOrderFull(order, signer);
}

/**
 * Create and sign an ASK order in one step
 */
export function createSignedAskOrder(
  params: AskOrderParams,
  signer: Keypair
): SignedOrder {
  const order = createAskOrder(params);
  return signOrderFull(order, signer);
}

// ============================================================================
// ORDER VALIDATION
// ============================================================================

/**
 * Check if an order has expired
 */
export function isOrderExpired(
  order: SignedOrder | Order,
  currentTime?: bigint
): boolean {
  if (order.expiration === 0n) {
    return false;
  }
  const now = currentTime ?? BigInt(Math.floor(Date.now() / 1000));
  return order.expiration <= now;
}

/**
 * Validate order crossing (orders are compatible for matching)
 */
export function ordersCanCross(
  buyOrder: SignedOrder,
  sellOrder: SignedOrder
): boolean {
  if (buyOrder.side !== OrderSide.BID || sellOrder.side !== OrderSide.ASK) {
    return false;
  }

  if (
    buyOrder.amountIn === 0n ||
    buyOrder.amountOut === 0n ||
    sellOrder.amountIn === 0n ||
    sellOrder.amountOut === 0n
  ) {
    return false;
  }

  return (
    buyOrder.amountIn * sellOrder.amountIn >=
    buyOrder.amountOut * sellOrder.amountOut
  );
}

/**
 * Calculate the fill amounts for a trade
 */
export function calculateTakerFill(
  makerOrder: SignedOrder,
  makerFillAmount: bigint
): bigint {
  if (makerOrder.amountIn === 0n) {
    throw new Error("Overflow: makerOrder.amountIn is zero");
  }

  const result = (makerFillAmount * makerOrder.amountOut) / makerOrder.amountIn;

  if (result > BigInt("18446744073709551615")) {
    throw new Error("Overflow: taker fill exceeds u64 max");
  }

  return result;
}

// ============================================================================
// SIGNED ORDER HELPERS
// ============================================================================

/**
 * Get signature as hex string (128 chars)
 */
export function signatureHex(order: SignedOrder): string {
  return order.signature.toString("hex");
}

/**
 * Check if a signed order has a non-zero signature
 */
export function isSigned(order: SignedOrder): boolean {
  return !order.signature.every((b) => b === 0);
}

/**
 * Apply an external base58-encoded signature to an unsigned order.
 * Rust equivalent: OrderPayload::apply_signature()
 */
export function applySignature(
  order: Omit<SignedOrder, "signature">,
  signatureBs58: string
): SignedOrder {
  const sigBytes = bs58.decode(signatureBs58);
  if (sigBytes.length !== 64) {
    throw new Error(`Invalid signature length: ${sigBytes.length}, expected 64`);
  }
  return { ...order, signature: Buffer.from(sigBytes) };
}

/**
 * Derive orderbook ID from base and quote token addresses.
 * Delegates to the canonical implementation in shared/types.
 */
export { deriveOrderbookId } from "../shared/types";

// ============================================================================
// CANCEL ORDER SIGNING
// ============================================================================

/**
 * Build the message bytes for cancelling an order.
 * The message is the order hash hex string as UTF-8 bytes (same protocol as order signing).
 */
export function cancelOrderMessage(orderHash: string): Uint8Array {
  return Buffer.from(orderHash, "ascii");
}

/**
 * Build the message bytes for cancelling a trigger order.
 * The message is the trigger order ID as UTF-8 bytes.
 */
export function cancelTriggerOrderMessage(triggerOrderId: string): Uint8Array {
  return Buffer.from(triggerOrderId, "ascii");
}

/**
 * Sign a cancel order request.
 * Returns the signature as a 128-char hex string.
 */
export function signCancelOrder(orderHash: string, signer: Keypair): string {
  const message = cancelOrderMessage(orderHash);
  const signature = sign.detached(message, signer.secretKey);
  return Buffer.from(signature).toString("hex");
}

/**
 * Sign a cancel trigger-order request.
 * Returns the signature as a 128-char hex string.
 */
export function signCancelTriggerOrder(triggerOrderId: string, signer: Keypair): string {
  const message = cancelTriggerOrderMessage(triggerOrderId);
  const signature = sign.detached(message, signer.secretKey);
  return Buffer.from(signature).toString("hex");
}

/**
 * Build the message string for cancelling all orders.
 * Format: "cancel_all:{pubkey}:{orderbook_id}:{timestamp}:{salt}"
 */
export function cancelAllMessage(
  userPubkey: string,
  orderbookId: string,
  timestamp: number,
  salt: string
): string {
  return `cancel_all:${userPubkey}:${orderbookId}:${timestamp}:${salt}`;
}

/**
 * Generate a random salt for cancel-all replay protection.
 * Returns an RFC 4122 UUID v4 string.
 */
export function generateCancelAllSalt(): string {
  return globalThis.crypto.randomUUID();
}

/**
 * Sign a cancel-all orders request.
 * Returns the signature as a 128-char hex string.
 */
export function signCancelAll(
  userPubkey: string,
  orderbookId: string,
  timestamp: number,
  salt: string,
  signer: Keypair
): string {
  const message = cancelAllMessage(userPubkey, orderbookId, timestamp, salt);
  const messageBytes = Buffer.from(message, "ascii");
  const signature = sign.detached(messageBytes, signer.secretKey);
  return Buffer.from(signature).toString("hex");
}

// ============================================================================
// SUBMIT REQUEST HELPERS
// ============================================================================

/**
 * Convert a SignedOrder to a SubmitOrderRequest-compatible object
 */
export interface SubmitRequestOptions {
  timeInForce?: TimeInForce;
  triggerPrice?: number;
  triggerType?: TriggerType;
  depositSource?: DepositSource;
}

export function toSubmitRequest(
  order: SignedOrder,
  orderbookId: string,
  options: SubmitRequestOptions = {}
): SubmitOrderRequest {
  return {
    maker: order.maker.toBase58(),
    nonce: order.nonce,
    salt: bigintToSafeNumber(order.salt, "salt"),
    market_pubkey: order.market.toBase58(),
    base_token: order.baseMint.toBase58(),
    quote_token: order.quoteMint.toBase58(),
    side: order.side,
    amount_in: bigintToSafeNumber(order.amountIn, "amount_in"),
    amount_out: bigintToSafeNumber(order.amountOut, "amount_out"),
    expiration: bigintToSafeNumber(order.expiration, "expiration"),
    signature: signatureHex(order),
    orderbook_id: orderbookId,
    tif: options.timeInForce,
    trigger_price: options.triggerPrice,
    trigger_type: options.triggerType,
    deposit_source: options.depositSource,
  };
}

// Rust-compatible snake_case aliases.
export const is_order_expired = isOrderExpired;
export const orders_can_cross = ordersCanCross;
export const calculate_taker_fill = calculateTakerFill;
export const cancel_order_message = cancelOrderMessage;
export const cancel_trigger_order_message = cancelTriggerOrderMessage;
export const cancel_all_message = cancelAllMessage;
export const generate_cancel_all_salt = generateCancelAllSalt;
export const derive_condition_id = deriveConditionId;
export const apply_signature = applySignature;
export const order_to_signed = orderToSigned;
