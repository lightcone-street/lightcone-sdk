import { PublicKey } from "@solana/web3.js";
import { keccak_256 } from "js-sha3";
import {
  TOKEN_PROGRAM_ID,
  TOKEN_2022_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "./constants";

// ============================================================================
// BUFFER UTILITIES - Little Endian
// ============================================================================

/**
 * Convert a bigint to little-endian bytes
 * @param value - The value to convert
 * @param bytes - Number of bytes in the output
 */
export function toLeBytes(value: bigint, bytes: number): Buffer {
  const buffer = Buffer.alloc(bytes);
  let remaining = value;
  for (let i = 0; i < bytes; i++) {
    buffer[i] = Number(remaining & 0xffn);
    remaining = remaining >> 8n;
  }
  return buffer;
}

/**
 * Convert little-endian bytes to bigint
 * @param buffer - The buffer to convert
 */
export function fromLeBytes(buffer: Buffer): bigint {
  let result = 0n;
  for (let i = buffer.length - 1; i >= 0; i--) {
    result = (result << 8n) | BigInt(buffer[i]);
  }
  return result;
}

/**
 * Convert a bigint to big-endian bytes
 * @param value - The value to convert
 * @param bytes - Number of bytes in the output
 */
export function toBeBytes(value: bigint, bytes: number): Buffer {
  const buffer = Buffer.alloc(bytes);
  let remaining = value;
  for (let i = bytes - 1; i >= 0; i--) {
    buffer[i] = Number(remaining & 0xffn);
    remaining = remaining >> 8n;
  }
  return buffer;
}

/**
 * Convert big-endian bytes to bigint
 * @param buffer - The buffer to convert
 */
export function fromBeBytes(buffer: Buffer): bigint {
  let result = 0n;
  for (let i = 0; i < buffer.length; i++) {
    result = (result << 8n) | BigInt(buffer[i]);
  }
  return result;
}

/**
 * Convert a number to u8 buffer
 */
export function toU8(value: number): Buffer {
  const buffer = Buffer.alloc(1);
  buffer[0] = value & 0xff;
  return buffer;
}

/**
 * Convert a bigint to u64 little-endian buffer
 */
export function toU64Le(value: bigint): Buffer {
  return toLeBytes(value, 8);
}

/**
 * Convert a bigint to i64 little-endian buffer (signed)
 */
export function toI64Le(value: bigint): Buffer {
  // For negative values, we need to handle two's complement
  if (value < 0n) {
    // Add 2^64 to get two's complement representation
    value = value + (1n << 64n);
  }
  return toLeBytes(value, 8);
}

/**
 * Convert a buffer to i64 (signed)
 */
export function fromI64Le(buffer: Buffer): bigint {
  const unsigned = fromLeBytes(buffer);
  // Check if the high bit is set (negative number)
  if (unsigned >= 1n << 63n) {
    return unsigned - (1n << 64n);
  }
  return unsigned;
}

// ============================================================================
// KECCAK256 HASHING
// ============================================================================

/**
 * Compute keccak256 hash of data
 * @param data - The data to hash
 * @returns 32-byte hash buffer
 */
export function keccak256(data: Buffer): Buffer {
  return Buffer.from(keccak_256.arrayBuffer(data));
}

/**
 * Derive condition ID from oracle, question ID, and number of outcomes
 * This matches the on-chain derivation: keccak256(oracle || questionId || numOutcomes)
 */
export function deriveConditionId(
  oracle: PublicKey,
  questionId: Buffer,
  numOutcomes: number
): Buffer {
  const data = Buffer.concat([
    oracle.toBuffer(),
    questionId,
    toU8(numOutcomes),
  ]);
  return keccak256(data);
}

// ============================================================================
// ASSOCIATED TOKEN ADDRESS HELPERS
// ============================================================================

/**
 * Derive Associated Token Address
 * @param mint - The token mint
 * @param owner - The owner of the ATA
 * @param token2022 - Whether to use Token-2022 program
 */
export function getAssociatedTokenAddress(
  mint: PublicKey,
  owner: PublicKey,
  token2022: boolean = false
): PublicKey {
  const tokenProgramId = token2022 ? TOKEN_2022_PROGRAM_ID : TOKEN_PROGRAM_ID;

  const [address] = PublicKey.findProgramAddressSync(
    [owner.toBuffer(), tokenProgramId.toBuffer(), mint.toBuffer()],
    ASSOCIATED_TOKEN_PROGRAM_ID
  );

  return address;
}

/**
 * Get ATA for a conditional token (always Token-2022)
 */
export function getConditionalTokenAta(
  mint: PublicKey,
  owner: PublicKey
): PublicKey {
  return getAssociatedTokenAddress(mint, owner, true);
}

/**
 * Get ATA for a deposit token (SPL Token)
 */
export function getDepositTokenAta(
  mint: PublicKey,
  owner: PublicKey
): PublicKey {
  return getAssociatedTokenAddress(mint, owner, false);
}

// ============================================================================
// STRING SERIALIZATION
// ============================================================================

/**
 * Serialize a string with u16 length prefix (little-endian)
 */
export function serializeString(str: string): Buffer {
  const strBuffer = Buffer.from(str, "utf-8");
  const lengthBuffer = Buffer.alloc(2);
  lengthBuffer.writeUInt16LE(strBuffer.length, 0);
  return Buffer.concat([lengthBuffer, strBuffer]);
}

/**
 * Deserialize a string with u16 length prefix
 * @returns [string, bytesConsumed]
 */
export function deserializeString(
  buffer: Buffer,
  offset: number
): [string, number] {
  const length = buffer.readUInt16LE(offset);
  const str = buffer.toString("utf-8", offset + 2, offset + 2 + length);
  return [str, 2 + length];
}

// ============================================================================
// VALIDATION HELPERS
// ============================================================================

/**
 * Validate that a number is within the valid outcomes range
 */
export function validateOutcomes(numOutcomes: number): void {
  if (numOutcomes < 2 || numOutcomes > 6) {
    throw new Error(
      `Invalid number of outcomes: ${numOutcomes}. Must be between 2 and 6.`
    );
  }
}

/**
 * Validate that an outcome index is valid for a market
 */
export function validateOutcomeIndex(
  outcomeIndex: number,
  numOutcomes: number
): void {
  if (outcomeIndex < 0 || outcomeIndex >= numOutcomes) {
    throw new Error(
      `Invalid outcome index: ${outcomeIndex}. Must be between 0 and ${numOutcomes - 1}.`
    );
  }
}

/**
 * Validate that a buffer is exactly 32 bytes
 */
export function validate32Bytes(buffer: Buffer, name: string): void {
  if (buffer.length !== 32) {
    throw new Error(`${name} must be exactly 32 bytes, got ${buffer.length}`);
  }
}
