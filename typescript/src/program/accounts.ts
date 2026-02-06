import { PublicKey } from "@solana/web3.js";
import { DISCRIMINATOR, ACCOUNT_SIZE } from "./constants";
import {
  Exchange,
  Market,
  MarketStatus,
  OrderStatus,
  UserNonce,
  Position,
  Orderbook,
} from "./types";
import { fromLeBytes } from "./utils";

// ============================================================================
// DISCRIMINATOR VALIDATION
// ============================================================================

/**
 * Validate that a buffer starts with the expected discriminator
 */
function validateDiscriminator(
  data: Buffer,
  expected: Buffer,
  accountType: string
): void {
  const actual = data.subarray(0, 8);
  if (!actual.equals(expected)) {
    throw new Error(
      `Invalid ${accountType} discriminator. Expected ${expected.toString("hex")}, got ${actual.toString("hex")}`
    );
  }
}

/**
 * Check if a buffer has a valid Exchange discriminator
 */
export function isExchangeAccount(data: Buffer): boolean {
  if (data.length < 8) return false;
  return data.subarray(0, 8).equals(DISCRIMINATOR.EXCHANGE);
}

/**
 * Check if a buffer has a valid Market discriminator
 */
export function isMarketAccount(data: Buffer): boolean {
  if (data.length < 8) return false;
  return data.subarray(0, 8).equals(DISCRIMINATOR.MARKET);
}

/**
 * Check if a buffer has a valid OrderStatus discriminator
 */
export function isOrderStatusAccount(data: Buffer): boolean {
  if (data.length < 8) return false;
  return data.subarray(0, 8).equals(DISCRIMINATOR.ORDER_STATUS);
}

/**
 * Check if a buffer has a valid UserNonce discriminator
 */
export function isUserNonceAccount(data: Buffer): boolean {
  if (data.length < 8) return false;
  return data.subarray(0, 8).equals(DISCRIMINATOR.USER_NONCE);
}

/**
 * Check if a buffer has a valid Position discriminator
 */
export function isPositionAccount(data: Buffer): boolean {
  if (data.length < 8) return false;
  return data.subarray(0, 8).equals(DISCRIMINATOR.POSITION);
}

/**
 * Check if a buffer has a valid Orderbook discriminator
 */
export function isOrderbookAccount(data: Buffer): boolean {
  if (data.length < 8) return false;
  return data.subarray(0, 8).equals(DISCRIMINATOR.ORDERBOOK);
}

// ============================================================================
// ACCOUNT DESERIALIZATION
// ============================================================================

/**
 * Deserialize Exchange account data
 *
 * Layout (88 bytes):
 * - discriminator: [u8; 8]
 * - authority: Pubkey (32 bytes)
 * - operator: Pubkey (32 bytes)
 * - market_count: u64 (8 bytes)
 * - paused: u8 (1 byte)
 * - bump: u8 (1 byte)
 * - _padding: [u8; 6]
 */
export function deserializeExchange(data: Buffer): Exchange {
  if (data.length < ACCOUNT_SIZE.EXCHANGE) {
    throw new Error(
      `Invalid Exchange data length: ${data.length}, expected ${ACCOUNT_SIZE.EXCHANGE}`
    );
  }

  validateDiscriminator(data, DISCRIMINATOR.EXCHANGE, "Exchange");

  let offset = 0;

  const discriminator = data.subarray(offset, offset + 8);
  offset += 8;

  const authority = new PublicKey(data.subarray(offset, offset + 32));
  offset += 32;

  const operator = new PublicKey(data.subarray(offset, offset + 32));
  offset += 32;

  const marketCount = fromLeBytes(data.subarray(offset, offset + 8));
  offset += 8;

  const paused = data[offset] !== 0;
  offset += 1;

  const bump = data[offset];
  offset += 1;

  // Skip padding: 6 bytes

  return {
    discriminator,
    authority,
    operator,
    marketCount,
    paused,
    bump,
  };
}

/**
 * Deserialize Market account data
 *
 * Layout (120 bytes):
 * - discriminator: [u8; 8]
 * - market_id: u64 (8 bytes)
 * - num_outcomes: u8 (1 byte)
 * - status: u8 (1 byte)
 * - winning_outcome: u8 (1 byte)
 * - has_winning_outcome: u8 (1 byte)
 * - bump: u8 (1 byte)
 * - _padding: [u8; 3]
 * - oracle: Pubkey (32 bytes)
 * - question_id: [u8; 32]
 * - condition_id: [u8; 32]
 */
export function deserializeMarket(data: Buffer): Market {
  if (data.length < ACCOUNT_SIZE.MARKET) {
    throw new Error(
      `Invalid Market data length: ${data.length}, expected ${ACCOUNT_SIZE.MARKET}`
    );
  }

  validateDiscriminator(data, DISCRIMINATOR.MARKET, "Market");

  let offset = 0;

  const discriminator = data.subarray(offset, offset + 8);
  offset += 8;

  const marketId = fromLeBytes(data.subarray(offset, offset + 8));
  offset += 8;

  const numOutcomes = data[offset];
  offset += 1;

  const statusByte = data[offset];
  offset += 1;

  const winningOutcome = data[offset];
  offset += 1;

  const hasWinningOutcome = data[offset] !== 0;
  offset += 1;

  const bump = data[offset];
  offset += 1;

  // Skip padding: 3 bytes
  offset += 3;

  const oracle = new PublicKey(data.subarray(offset, offset + 32));
  offset += 32;

  const questionId = Buffer.from(data.subarray(offset, offset + 32));
  offset += 32;

  const conditionId = Buffer.from(data.subarray(offset, offset + 32));
  offset += 32;

  // Map status byte to enum
  let status: MarketStatus;
  switch (statusByte) {
    case 0:
      status = MarketStatus.Pending;
      break;
    case 1:
      status = MarketStatus.Active;
      break;
    case 2:
      status = MarketStatus.Resolved;
      break;
    case 3:
      status = MarketStatus.Cancelled;
      break;
    default:
      throw new Error(`Unknown market status: ${statusByte}`);
  }

  return {
    discriminator,
    marketId,
    numOutcomes,
    status,
    winningOutcome,
    hasWinningOutcome,
    bump,
    oracle,
    questionId,
    conditionId,
  };
}

/**
 * Deserialize OrderStatus account data
 *
 * Layout (24 bytes):
 * - discriminator: [u8; 8]
 * - remaining: u64 (8 bytes)
 * - is_cancelled: u8 (1 byte)
 * - _padding: [u8; 7]
 */
export function deserializeOrderStatus(data: Buffer): OrderStatus {
  if (data.length < ACCOUNT_SIZE.ORDER_STATUS) {
    throw new Error(
      `Invalid OrderStatus data length: ${data.length}, expected ${ACCOUNT_SIZE.ORDER_STATUS}`
    );
  }

  validateDiscriminator(data, DISCRIMINATOR.ORDER_STATUS, "OrderStatus");

  let offset = 0;

  const discriminator = data.subarray(offset, offset + 8);
  offset += 8;

  const remaining = fromLeBytes(data.subarray(offset, offset + 8));
  offset += 8;

  const isCancelled = data[offset] !== 0;
  offset += 1;

  // Skip padding: 7 bytes

  return {
    discriminator,
    remaining,
    isCancelled,
  };
}

/**
 * Deserialize UserNonce account data
 *
 * Layout (16 bytes):
 * - discriminator: [u8; 8]
 * - nonce: u64 (8 bytes)
 */
export function deserializeUserNonce(data: Buffer): UserNonce {
  if (data.length < ACCOUNT_SIZE.USER_NONCE) {
    throw new Error(
      `Invalid UserNonce data length: ${data.length}, expected ${ACCOUNT_SIZE.USER_NONCE}`
    );
  }

  validateDiscriminator(data, DISCRIMINATOR.USER_NONCE, "UserNonce");

  let offset = 0;

  const discriminator = data.subarray(offset, offset + 8);
  offset += 8;

  const nonce = fromLeBytes(data.subarray(offset, offset + 8));
  offset += 8;

  return {
    discriminator,
    nonce,
  };
}

/**
 * Deserialize Position account data
 *
 * Layout (80 bytes):
 * - discriminator: [u8; 8]
 * - owner: Pubkey (32 bytes)
 * - market: Pubkey (32 bytes)
 * - bump: u8 (1 byte)
 * - _padding: [u8; 7]
 */
export function deserializePosition(data: Buffer): Position {
  if (data.length < ACCOUNT_SIZE.POSITION) {
    throw new Error(
      `Invalid Position data length: ${data.length}, expected ${ACCOUNT_SIZE.POSITION}`
    );
  }

  validateDiscriminator(data, DISCRIMINATOR.POSITION, "Position");

  let offset = 0;

  const discriminator = data.subarray(offset, offset + 8);
  offset += 8;

  const owner = new PublicKey(data.subarray(offset, offset + 32));
  offset += 32;

  const market = new PublicKey(data.subarray(offset, offset + 32));
  offset += 32;

  const bump = data[offset];
  offset += 1;

  // Skip padding: 7 bytes

  return {
    discriminator,
    owner,
    market,
    bump,
  };
}

/**
 * Deserialize Orderbook account data
 *
 * Layout (144 bytes):
 * - discriminator: [u8; 8]
 * - market: Pubkey (32 bytes)
 * - mint_a: Pubkey (32 bytes)
 * - mint_b: Pubkey (32 bytes)
 * - lookup_table: Pubkey (32 bytes)
 * - bump: u8 (1 byte)
 * - _padding: [u8; 7]
 */
export function deserializeOrderbook(data: Buffer): Orderbook {
  if (data.length < ACCOUNT_SIZE.ORDERBOOK) {
    throw new Error(
      `Invalid Orderbook data length: ${data.length}, expected ${ACCOUNT_SIZE.ORDERBOOK}`
    );
  }

  validateDiscriminator(data, DISCRIMINATOR.ORDERBOOK, "Orderbook");

  let offset = 0;

  const discriminator = data.subarray(offset, offset + 8);
  offset += 8;

  const market = new PublicKey(data.subarray(offset, offset + 32));
  offset += 32;

  const mintA = new PublicKey(data.subarray(offset, offset + 32));
  offset += 32;

  const mintB = new PublicKey(data.subarray(offset, offset + 32));
  offset += 32;

  const lookupTable = new PublicKey(data.subarray(offset, offset + 32));
  offset += 32;

  const bump = data[offset];
  offset += 1;

  // Skip padding: 7 bytes

  return {
    discriminator,
    market,
    mintA,
    mintB,
    lookupTable,
    bump,
  };
}
