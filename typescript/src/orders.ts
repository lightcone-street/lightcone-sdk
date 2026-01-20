import { PublicKey, Keypair } from "@solana/web3.js";
import { sign } from "tweetnacl";
import {
  FullOrder,
  CompactOrder,
  OrderSide,
  BidOrderParams,
  AskOrderParams,
} from "./types";
import { ORDER_SIZE } from "./constants";
import { keccak256, toU64Le, toI64Le, toU8, fromLeBytes, fromI64Le } from "./utils";

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
export function hashOrder(order: FullOrder): Buffer {
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
 * Get the message to sign for an order (the order hash)
 */
export function getOrderMessage(order: FullOrder): Buffer {
  return hashOrder(order);
}

// ============================================================================
// ORDER SIGNING
// ============================================================================

/**
 * Sign an order with a Keypair
 * Returns 64-byte Ed25519 signature
 */
export function signOrder(order: FullOrder, signer: Keypair): Buffer {
  const message = hashOrder(order);
  const signature = sign.detached(message, signer.secretKey);
  return Buffer.from(signature);
}

/**
 * Sign an order and return a new order with the signature attached
 */
export function signOrderFull(
  order: Omit<FullOrder, "signature">,
  signer: Keypair
): FullOrder {
  const orderWithEmptySig: FullOrder = {
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
 * Verify an order's signature
 */
export function verifyOrderSignature(order: FullOrder): boolean {
  const message = hashOrder(order);
  return sign.detached.verify(
    message,
    order.signature,
    order.maker.toBytes()
  );
}

// ============================================================================
// ORDER SERIALIZATION
// ============================================================================

/**
 * Serialize a full order to bytes (225 bytes)
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
export function serializeFullOrder(order: FullOrder): Buffer {
  const buffer = Buffer.alloc(ORDER_SIZE.FULL);
  let offset = 0;

  // nonce (u64)
  toU64Le(order.nonce).copy(buffer, offset);
  offset += 8;

  // maker (32 bytes)
  order.maker.toBuffer().copy(buffer, offset);
  offset += 32;

  // market (32 bytes)
  order.market.toBuffer().copy(buffer, offset);
  offset += 32;

  // baseMint (32 bytes)
  order.baseMint.toBuffer().copy(buffer, offset);
  offset += 32;

  // quoteMint (32 bytes)
  order.quoteMint.toBuffer().copy(buffer, offset);
  offset += 32;

  // side (u8)
  buffer[offset] = order.side;
  offset += 1;

  // makerAmount (u64)
  toU64Le(order.makerAmount).copy(buffer, offset);
  offset += 8;

  // takerAmount (u64)
  toU64Le(order.takerAmount).copy(buffer, offset);
  offset += 8;

  // expiration (i64)
  toI64Le(order.expiration).copy(buffer, offset);
  offset += 8;

  // signature (64 bytes)
  order.signature.copy(buffer, offset);

  return buffer;
}

/**
 * Deserialize a full order from bytes
 */
export function deserializeFullOrder(data: Buffer): FullOrder {
  if (data.length < ORDER_SIZE.FULL) {
    throw new Error(
      `Invalid full order length: ${data.length}, expected ${ORDER_SIZE.FULL}`
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

/**
 * Serialize a compact order to bytes (65 bytes)
 *
 * Layout:
 * [0..8]    nonce (u64)
 * [8..40]   maker (Pubkey)
 * [40]      side (u8)
 * [41..49]  makerAmount (u64)
 * [49..57]  takerAmount (u64)
 * [57..65]  expiration (i64)
 */
export function serializeCompactOrder(order: CompactOrder): Buffer {
  const buffer = Buffer.alloc(ORDER_SIZE.COMPACT);
  let offset = 0;

  // nonce (u64)
  toU64Le(order.nonce).copy(buffer, offset);
  offset += 8;

  // maker (32 bytes)
  order.maker.toBuffer().copy(buffer, offset);
  offset += 32;

  // side (u8)
  buffer[offset] = order.side;
  offset += 1;

  // makerAmount (u64)
  toU64Le(order.makerAmount).copy(buffer, offset);
  offset += 8;

  // takerAmount (u64)
  toU64Le(order.takerAmount).copy(buffer, offset);
  offset += 8;

  // expiration (i64)
  toI64Le(order.expiration).copy(buffer, offset);

  return buffer;
}

/**
 * Deserialize a compact order from bytes
 */
export function deserializeCompactOrder(data: Buffer): CompactOrder {
  if (data.length < ORDER_SIZE.COMPACT) {
    throw new Error(
      `Invalid compact order length: ${data.length}, expected ${ORDER_SIZE.COMPACT}`
    );
  }

  let offset = 0;

  const nonce = fromLeBytes(data.subarray(offset, offset + 8));
  offset += 8;

  const maker = new PublicKey(data.subarray(offset, offset + 32));
  offset += 32;

  const side = data[offset] as OrderSide;
  offset += 1;

  const makerAmount = fromLeBytes(data.subarray(offset, offset + 8));
  offset += 8;

  const takerAmount = fromLeBytes(data.subarray(offset, offset + 8));
  offset += 8;

  const expiration = fromI64Le(data.subarray(offset, offset + 8));

  return {
    nonce,
    maker,
    side,
    makerAmount,
    takerAmount,
    expiration,
  };
}

// ============================================================================
// ORDER CREATION HELPERS
// ============================================================================

/**
 * Create a BID order (buyer wants base tokens, pays with quote tokens)
 *
 * @param params.nonce - Order ID / replay protection
 * @param params.maker - The signer's public key
 * @param params.market - The market public key
 * @param params.baseMint - Token to buy (conditional token)
 * @param params.quoteMint - Token to pay with
 * @param params.makerAmount - Quote tokens to give
 * @param params.takerAmount - Base tokens to receive
 * @param params.expiration - Unix timestamp, 0 = no expiration
 */
export function createBidOrder(
  params: BidOrderParams
): Omit<FullOrder, "signature"> {
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
 *
 * @param params.nonce - Order ID / replay protection
 * @param params.maker - The signer's public key
 * @param params.market - The market public key
 * @param params.baseMint - Token to sell (conditional token)
 * @param params.quoteMint - Token to receive
 * @param params.makerAmount - Base tokens to give
 * @param params.takerAmount - Quote tokens to receive
 * @param params.expiration - Unix timestamp, 0 = no expiration
 */
export function createAskOrder(
  params: AskOrderParams
): Omit<FullOrder, "signature"> {
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
): FullOrder {
  const order = createBidOrder(params);
  return signOrderFull(order, signer);
}

/**
 * Create and sign an ASK order in one step
 */
export function createSignedAskOrder(
  params: AskOrderParams,
  signer: Keypair
): FullOrder {
  const order = createAskOrder(params);
  return signOrderFull(order, signer);
}

// ============================================================================
// ORDER VALIDATION
// ============================================================================

/**
 * Check if an order has expired
 * @param order - The order to check
 * @param currentTime - Current unix timestamp (defaults to now)
 */
export function isOrderExpired(
  order: FullOrder | CompactOrder,
  currentTime?: bigint
): boolean {
  if (order.expiration === 0n) {
    return false; // No expiration
  }
  const now = currentTime ?? BigInt(Math.floor(Date.now() / 1000));
  return order.expiration < now;
}

/**
 * Validate order crossing (orders are compatible for matching)
 * For a match: buyer.makerAmount * seller.makerAmount >= buyer.takerAmount * seller.takerAmount
 */
export function ordersCanCross(
  buyOrder: FullOrder,
  sellOrder: FullOrder
): boolean {
  if (buyOrder.side !== OrderSide.BID || sellOrder.side !== OrderSide.ASK) {
    return false;
  }

  // buyer pays buyOrder.makerAmount quote to receive buyOrder.takerAmount base
  // seller gives sellOrder.makerAmount base to receive sellOrder.takerAmount quote
  // For crossing: buyer's quote offer >= seller's quote ask for the base amounts
  // buyer.makerAmount / buyer.takerAmount >= seller.takerAmount / seller.makerAmount
  // Rearranged: buyer.makerAmount * seller.makerAmount >= buyer.takerAmount * seller.takerAmount
  return (
    buyOrder.makerAmount * sellOrder.makerAmount >=
    buyOrder.takerAmount * sellOrder.takerAmount
  );
}

/**
 * Calculate the fill amounts for a trade
 * @param makerOrder - The maker's order
 * @param fillAmount - Amount the maker is giving
 * @returns takerFillAmount - Amount the taker gives in return
 */
export function calculateTakerFill(
  makerOrder: FullOrder,
  makerFillAmount: bigint
): bigint {
  // takerFill = makerFill * maker.takerAmount / maker.makerAmount
  return (makerFillAmount * makerOrder.takerAmount) / makerOrder.makerAmount;
}
