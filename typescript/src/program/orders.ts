import { PublicKey, Keypair } from "@solana/web3.js";
import { sign } from "tweetnacl";
import {
  SignedOrder,
  Order,
  OrderSide,
  BidOrderParams,
  AskOrderParams,
} from "./types";
import { ORDER_SIZE } from "./constants";
import { keccak256, toU64Le, toI64Le, toU8, toU32Le, fromLeBytes, fromI64Le } from "./utils";

// ============================================================================
// ORDER HASHING
// ============================================================================

/**
 * Hash an order using keccak256
 * Layout (161 bytes - order without signature):
 * nonce (8) || maker (32) || market (32) || baseMint (32) || quoteMint (32) ||
 * side (1) || makerAmount (8) || takerAmount (8) || expiration (8)
 *
 * @returns 32-byte keccak256 hash
 */
export function hashOrder(order: SignedOrder): Buffer {
  const data = Buffer.concat([
    toU64Le(order.nonce),
    order.maker.toBuffer(),
    order.market.toBuffer(),
    order.baseMint.toBuffer(),
    order.quoteMint.toBuffer(),
    toU8(order.side),
    toU64Le(order.makerAmount),
    toU64Le(order.takerAmount),
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
 * Serialize a signed order to bytes (225 bytes)
 *
 * Layout:
 * [0..8]     nonce (u64)
 * [8..40]    maker (Pubkey)
 * [40..72]   market (Pubkey)
 * [72..104]  baseMint (Pubkey)
 * [104..136] quoteMint (Pubkey)
 * [136]      side (u8)
 * [137..145] makerAmount (u64)
 * [145..153] takerAmount (u64)
 * [153..161] expiration (i64)
 * [161..225] signature (64 bytes)
 */
export function serializeSignedOrder(order: SignedOrder): Buffer {
  const buffer = Buffer.alloc(ORDER_SIZE.SIGNED_ORDER);
  let offset = 0;

  toU64Le(order.nonce).copy(buffer, offset);
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

  toU64Le(order.makerAmount).copy(buffer, offset);
  offset += 8;

  toU64Le(order.takerAmount).copy(buffer, offset);
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

  const nonce = fromLeBytes(data.subarray(offset, offset + 8));
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

  const makerAmount = fromLeBytes(data.subarray(offset, offset + 8));
  offset += 8;

  const takerAmount = fromLeBytes(data.subarray(offset, offset + 8));
  offset += 8;

  const expiration = fromI64Le(data.subarray(offset, offset + 8));
  offset += 8;

  const signature = Buffer.from(data.subarray(offset, offset + 64));

  return {
    nonce,
    maker,
    market,
    baseMint,
    quoteMint,
    side,
    makerAmount,
    takerAmount,
    expiration,
    signature,
  };
}

// ============================================================================
// ORDER SERIALIZATION (29 bytes)
// ============================================================================

/**
 * Serialize a compact order to bytes (29 bytes)
 *
 * Layout:
 * [0..4]    nonce (u32)
 * [4]       side (u8)
 * [5..13]   makerAmount (u64)
 * [13..21]  takerAmount (u64)
 * [21..29]  expiration (i64)
 */
export function serializeOrder(order: Order): Buffer {
  const buffer = Buffer.alloc(ORDER_SIZE.ORDER);
  let offset = 0;

  toU32Le(order.nonce).copy(buffer, offset);
  offset += 4;

  buffer[offset] = order.side;
  offset += 1;

  toU64Le(order.makerAmount).copy(buffer, offset);
  offset += 8;

  toU64Le(order.takerAmount).copy(buffer, offset);
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

  const side = data[offset] as OrderSide;
  offset += 1;

  const makerAmount = fromLeBytes(data.subarray(offset, offset + 8));
  offset += 8;

  const takerAmount = fromLeBytes(data.subarray(offset, offset + 8));
  offset += 8;

  const expiration = fromI64Le(data.subarray(offset, offset + 8));

  return {
    nonce,
    side,
    makerAmount,
    takerAmount,
    expiration,
  };
}

/**
 * Convert a SignedOrder to a compact Order (truncate nonce to u32, drop maker)
 */
export function signedOrderToOrder(order: SignedOrder): Order {
  return {
    nonce: Number(order.nonce & 0xFFFFFFFFn),
    side: order.side,
    makerAmount: order.makerAmount,
    takerAmount: order.takerAmount,
    expiration: order.expiration,
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
    maker: params.maker,
    market: params.market,
    baseMint: params.baseMint,
    quoteMint: params.quoteMint,
    side: OrderSide.BID,
    makerAmount: params.makerAmount,
    takerAmount: params.takerAmount,
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
    maker: params.maker,
    market: params.market,
    baseMint: params.baseMint,
    quoteMint: params.quoteMint,
    side: OrderSide.ASK,
    makerAmount: params.makerAmount,
    takerAmount: params.takerAmount,
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
  return (
    buyOrder.makerAmount * sellOrder.makerAmount >=
    buyOrder.takerAmount * sellOrder.takerAmount
  );
}

/**
 * Calculate the fill amounts for a trade
 */
export function calculateTakerFill(
  makerOrder: SignedOrder,
  makerFillAmount: bigint
): bigint {
  return (makerFillAmount * makerOrder.takerAmount) / makerOrder.makerAmount;
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
 * Derive orderbook ID from base and quote token addresses
 * Format: "{base[0:8]}_{quote[0:8]}"
 */
export function deriveOrderbookId(
  baseToken: string,
  quoteToken: string
): string {
  return `${baseToken.slice(0, 8)}_${quoteToken.slice(0, 8)}`;
}

/**
 * Convert a SignedOrder to a SubmitOrderRequest-compatible object
 */
export function toSubmitRequest(
  order: SignedOrder,
  orderbookId: string
): {
  maker: string;
  nonce: string;
  market_pubkey: string;
  base_token: string;
  quote_token: string;
  side: number;
  maker_amount: string;
  taker_amount: string;
  expiration: number;
  signature: string;
  orderbook_id: string;
} {
  return {
    maker: order.maker.toBase58(),
    nonce: order.nonce.toString(),
    market_pubkey: order.market.toBase58(),
    base_token: order.baseMint.toBase58(),
    quote_token: order.quoteMint.toBase58(),
    side: order.side,
    maker_amount: order.makerAmount.toString(),
    taker_amount: order.takerAmount.toString(),
    expiration: Number(order.expiration),
    signature: signatureHex(order),
    orderbook_id: orderbookId,
  };
}

// Legacy aliases for backwards compatibility during migration
/** @deprecated Use serializeSignedOrder instead */
export const serializeFullOrder = serializeSignedOrder;
/** @deprecated Use deserializeSignedOrder instead */
export const deserializeFullOrder = deserializeSignedOrder;
/** @deprecated Use serializeOrder instead */
export const serializeCompactOrder = serializeOrder;
/** @deprecated Use deserializeOrder instead */
export const deserializeCompactOrder = deserializeOrder;
