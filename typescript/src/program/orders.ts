import { PublicKey, Keypair } from "@solana/web3.js";
import { sign } from "tweetnacl";
import bs58 from "bs58";
import type {
  DepositSource,
  OrderbookRules,
  SubmitOrderRequest,
  TimeInForce,
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
  fromLeBytes,
  fromI64Le,
  deriveConditionId,
} from "./utils";
import { ProgramSdkError } from "./error";
import { validateRawAmounts, validateSignedFields } from "../shared/scaling";

const U64_MAX = (1n << 64n) - 1n;

/**
 * Generate a random salt for order identity.
 *
 * The salt is the order's only identity field, so it spans the full u64
 * range.
 */
export function generateSalt(): bigint {
  const buf = new Uint8Array(8);
  globalThis.crypto.getRandomValues(buf);
  return fromLeBytes(Buffer.from(buf));
}

function decodeSide(value: number): OrderSide {
  if (value === OrderSide.BID || value === OrderSide.ASK) return value;
  throw ProgramSdkError.invalidSide(value);
}


// ============================================================================
// ORDER HASHING
// ============================================================================

/**
 * Build the 161-byte signing preimage from the signed fields (integers
 * little-endian):
 * salt (8) || maker (32) || market (32) || baseMint (32) || quoteMint (32) ||
 * side (1) || amountIn (8) || amountOut (8) || expiration (8)
 */
export function orderPreimage(order: SignedOrder | Omit<SignedOrder, "signature">): Buffer {
  return Buffer.concat([
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
}

/**
 * Hash an order: Keccak-256 of its 161-byte preimage. This is the order ID
 * and the order-status PDA seed.
 *
 * @returns 32-byte keccak256 hash
 */
export function hashOrder(order: SignedOrder | Omit<SignedOrder, "signature">): Buffer {
  return keccak256(orderPreimage(order));
}

/**
 * Get the hex-encoded hash of an order (lowercase, 64 characters)
 */
export function hashOrderHex(order: SignedOrder | Omit<SignedOrder, "signature">): string {
  return hashOrder(order).toString("hex");
}

/**
 * The bytes the maker signs: the 64 ASCII characters of the lowercase-hex
 * order hash, not the raw 32-byte hash.
 */
export function getOrderMessage(order: SignedOrder | Omit<SignedOrder, "signature">): Buffer {
  return Buffer.from(hashOrderHex(order), "ascii");
}

// ============================================================================
// ORDER SIGNING
// ============================================================================

/**
 * Sign an order with a Keypair.
 * Signs the hex-encoded keccak hash (64-char ASCII string) for cross-compatibility with Rust.
 * Returns 64-byte Ed25519 signature.
 */
export function signOrder(
  order: SignedOrder,
  signer: Keypair,
  rules: OrderbookRules
): Buffer {
  validateRawAmounts(order.amountIn, order.amountOut, order.side, rules);
  const signature = sign.detached(getOrderMessage(order), signer.secretKey);
  return Buffer.from(signature);
}

/**
 * Sign an order and return a new order with the signature attached
 */
export function signOrderFull(
  order: Omit<SignedOrder, "signature">,
  signer: Keypair,
  rules: OrderbookRules
): SignedOrder {
  const orderWithEmptySig: SignedOrder = {
    ...order,
    signature: Buffer.alloc(64),
  };
  const signature = signOrder(orderWithEmptySig, signer, rules);
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
  return sign.detached.verify(
    getOrderMessage(order),
    order.signature,
    order.maker.toBytes()
  );
}

// ============================================================================
// SIGNED ORDER SERIALIZATION (225 bytes)
// ============================================================================

/**
 * Serialize a signed order to bytes (225 bytes): the 161-byte preimage
 * followed by the signature.
 *
 * Layout (integers little-endian):
 * [0..8]     salt (u64)
 * [8..40]    maker (Pubkey)
 * [40..72]   market (Pubkey)
 * [72..104]  baseMint (Pubkey)
 * [104..136] quoteMint (Pubkey)
 * [136]      side (u8)
 * [137..145] amountIn (u64)
 * [145..153] amountOut (u64)
 * [153..161] expiration (i64)
 * [161..225] signature (64 bytes)
 */
export function serializeSignedOrder(order: SignedOrder): Buffer {
  if (order.signature.length !== ORDER_SIZE.SIGNATURE) {
    throw ProgramSdkError.invalidDataLength("signature", ORDER_SIZE.SIGNATURE, order.signature.length);
  }
  return Buffer.concat([orderPreimage(order), order.signature]);
}

/**
 * Deserialize a signed order from exactly 225 bytes.
 */
export function deserializeSignedOrder(data: Buffer): SignedOrder {
  if (data.length !== ORDER_SIZE.SIGNED_ORDER) {
    throw ProgramSdkError.invalidDataLength("SignedOrder", ORDER_SIZE.SIGNED_ORDER, data.length);
  }

  return {
    salt: fromLeBytes(data.subarray(0, 8)),
    maker: new PublicKey(data.subarray(8, 40)),
    market: new PublicKey(data.subarray(40, 72)),
    baseMint: new PublicKey(data.subarray(72, 104)),
    quoteMint: new PublicKey(data.subarray(104, 136)),
    side: decodeSide(data[136]),
    amountIn: fromLeBytes(data.subarray(137, 145)),
    amountOut: fromLeBytes(data.subarray(145, 153)),
    expiration: fromI64Le(data.subarray(153, 161)),
    signature: Buffer.from(data.subarray(161, 225)),
  };
}

// ============================================================================
// ORDER SERIALIZATION (33 bytes)
// ============================================================================

/**
 * Serialize a compact order to bytes (33 bytes)
 *
 * Layout (integers little-endian):
 * [0..8]    salt (u64)
 * [8]       side (u8)
 * [9..17]   amountIn (u64)
 * [17..25]  amountOut (u64)
 * [25..33]  expiration (i64)
 */
export function serializeOrder(order: Order): Buffer {
  return Buffer.concat([
    toU64Le(order.salt),
    toU8(order.side),
    toU64Le(order.amountIn),
    toU64Le(order.amountOut),
    toI64Le(order.expiration),
  ]);
}

/**
 * Deserialize a compact order from exactly 33 bytes.
 */
export function deserializeOrder(data: Buffer): Order {
  if (data.length !== ORDER_SIZE.ORDER) {
    throw ProgramSdkError.invalidDataLength("Order", ORDER_SIZE.ORDER, data.length);
  }

  return {
    salt: fromLeBytes(data.subarray(0, 8)),
    side: decodeSide(data[8]),
    amountIn: fromLeBytes(data.subarray(9, 17)),
    amountOut: fromLeBytes(data.subarray(17, 25)),
    expiration: fromI64Le(data.subarray(25, 33)),
  };
}

/**
 * Convert a SignedOrder to a compact Order (drop maker, market, and mints)
 */
export function signedOrderToOrder(order: SignedOrder): Order {
  return {
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
  signer: Keypair,
  rules: OrderbookRules
): SignedOrder {
  const order = createBidOrder(params);
  return signOrderFull(order, signer, rules);
}

/**
 * Create and sign an ASK order in one step
 */
export function createSignedAskOrder(
  params: AskOrderParams,
  signer: Keypair,
  rules: OrderbookRules
): SignedOrder {
  const order = createAskOrder(params);
  return signOrderFull(order, signer, rules);
}

// ============================================================================
// ORDER VALIDATION
// ============================================================================

/**
 * Check if an order has expired at `currentTime` (Unix seconds; defaults to
 * now).
 *
 * Matches the program: an order expires only when its expiration is nonzero
 * and strictly earlier than the current time, so it is still valid during the
 * second equal to its expiration.
 */
export function isOrderExpired(
  order: SignedOrder | Order,
  currentTime?: bigint
): boolean {
  if (order.expiration === 0n) {
    return false;
  }
  const now = currentTime ?? BigInt(Math.floor(Date.now() / 1000));
  return order.expiration < now;
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
 * Calculate the minimum taker fill for a maker fill, in raw atoms.
 *
 * `makerFillAmount` is what the maker gives, in atoms of the maker's given
 * mint. The result is the least the taker must give the maker, in atoms of
 * the maker's received mint, rounded up as the program requires:
 * `takerFill >= ceil(makerFill * maker.amountOut / maker.amountIn)`.
 */
export function calculateTakerFill(
  makerOrder: SignedOrder,
  makerFillAmount: bigint
): bigint {
  if (makerOrder.amountIn === 0n) {
    throw ProgramSdkError.divisionByZero();
  }

  const numerator = makerFillAmount * makerOrder.amountOut;
  const result = (numerator + makerOrder.amountIn - 1n) / makerOrder.amountIn;

  if (result > U64_MAX) {
    throw ProgramSdkError.overflow("Taker fill exceeds u64 max");
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
  signatureBs58: string,
  rules: OrderbookRules
): SignedOrder {
  validateSignedFields(order.amountIn, order.amountOut);
  validateRawAmounts(order.amountIn, order.amountOut, order.side, rules);
  const sigBytes = bs58.decode(signatureBs58);
  if (sigBytes.length !== 64) {
    throw ProgramSdkError.invalidSignature();
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
 * Sign a cancel order request.
 * Returns the signature as a 128-char hex string.
 */
export function signCancelOrder(orderHash: string, signer: Keypair): string {
  const message = cancelOrderMessage(orderHash);
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
 * Execution policy and collateral source for an ordinary signed order.
 */
export interface SubmitRequestOptions {
  timeInForce?: TimeInForce;
  depositSource?: DepositSource;
}

/**
 * Encodes a signed order as the REST submission request without changing its
 * exact integer fields. Rejects an unsigned order.
 */
export function toSubmitRequest(
  order: SignedOrder,
  orderbookId: string,
  options: SubmitRequestOptions = {}
): SubmitOrderRequest {
  if (!isSigned(order)) {
    throw ProgramSdkError.unsignedOrder();
  }
  validateSignedFields(order.amountIn, order.amountOut);
  return {
    maker: order.maker.toBase58(),
    salt: order.salt,
    market_pubkey: order.market.toBase58(),
    base_token: order.baseMint.toBase58(),
    quote_token: order.quoteMint.toBase58(),
    side: order.side,
    amount_in: order.amountIn,
    amount_out: order.amountOut,
    expiration: order.expiration,
    signature: signatureHex(order),
    orderbook_id: orderbookId,
    tif: options.timeInForce,
    deposit_source: options.depositSource,
  };
}

// Rust-compatible snake_case aliases.
export const is_order_expired = isOrderExpired;
export const orders_can_cross = ordersCanCross;
export const calculate_taker_fill = calculateTakerFill;
export const cancel_order_message = cancelOrderMessage;
export const cancel_all_message = cancelAllMessage;
export const generate_cancel_all_salt = generateCancelAllSalt;
export const derive_condition_id = deriveConditionId;
export const apply_signature = applySignature;
export const order_to_signed = orderToSigned;
