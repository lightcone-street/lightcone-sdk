import { PublicKey } from "@solana/web3.js";
import { keccak_256 } from "js-sha3";
import { ProgramSdkError } from "./error";
import {
  MAX_OUTCOMES,
  MIN_OUTCOMES,
  TOKEN_PROGRAM_ID,
  TOKEN_2022_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "./constants";
import type { ScalarResolutionParams } from "./types";

const U32_MAX = 0xffffffffn;

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
 * Convert a number to u32 little-endian buffer
 */
export function toU32Le(value: number): Buffer {
  const buffer = Buffer.alloc(4);
  buffer.writeUInt32LE(value, 0);
  return buffer;
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
  if (!Number.isInteger(numOutcomes) || numOutcomes < MIN_OUTCOMES || numOutcomes > MAX_OUTCOMES) {
    throw ProgramSdkError.invalidOutcomeCount(numOutcomes);
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
    throw ProgramSdkError.invalidOutcomeIndex(outcomeIndex, numOutcomes - 1);
  }
}

/**
 * Validate that a buffer is exactly 32 bytes
 */
export function validate32Bytes(buffer: Buffer, name: string): void {
  if (buffer.length !== 32) {
    throw ProgramSdkError.invalidDataLength(name, 32, buffer.length);
  }
}

/**
 * Build a winner-takes-all payout vector for a binary or multi-outcome market.
 */
export function winnerTakesAllPayoutNumerators(
  winningOutcome: number,
  numOutcomes: number
): number[] {
  validateOutcomes(numOutcomes);
  validateOutcomeIndex(winningOutcome, numOutcomes);

  const payoutNumerators = Array<number>(numOutcomes).fill(0);
  payoutNumerators[winningOutcome] = 1;
  return payoutNumerators;
}

/**
 * Convert a two-sided scalar resolution into program payout numerators.
 *
 * All values are integer fixed-point BigInts. The resolved value is clamped to
 * [minValue, maxValue], reduced by GCD, and checked against the program's u32
 * payout representation.
 */
export function scalarToPayoutNumerators(
  params: ScalarResolutionParams
): number[] {
  validateOutcomes(params.numOutcomes);
  validateOutcomeIndex(params.lowerOutcomeIndex, params.numOutcomes);
  validateOutcomeIndex(params.upperOutcomeIndex, params.numOutcomes);

  if (params.lowerOutcomeIndex === params.upperOutcomeIndex) {
    throw ProgramSdkError.duplicateScalarOutcomes();
  }

  const range = params.maxValue - params.minValue;
  if (range <= 0n) {
    throw ProgramSdkError.invalidScalarRange();
  }

  const clamped =
    params.resolvedValue < params.minValue
      ? params.minValue
      : params.resolvedValue > params.maxValue
        ? params.maxValue
        : params.resolvedValue;

  const numerators = Array<bigint>(params.numOutcomes).fill(0n);
  numerators[params.lowerOutcomeIndex] = params.maxValue - clamped;
  numerators[params.upperOutcomeIndex] = clamped - params.minValue;

  return reduceAndFitPayoutNumerators(numerators);
}

function reduceAndFitPayoutNumerators(numerators: bigint[]): number[] {
  const nonZero = numerators.filter((n) => n > 0n);
  if (nonZero.length === 0) {
    throw ProgramSdkError.invalidPayoutNumerators();
  }

  const gcd = nonZero.reduce((acc, value) => gcdBigInt(acc, value));
  let sum = 0n;
  const reduced = numerators.map((numerator) => {
    const value = numerator === 0n ? 0n : numerator / gcd;
    if (value > U32_MAX) {
      throw ProgramSdkError.payoutVectorExceedsU32();
    }
    sum += value;
    return Number(value);
  });

  if (sum === 0n) {
    throw ProgramSdkError.invalidPayoutNumerators();
  }
  if (sum > U32_MAX) {
    throw ProgramSdkError.payoutVectorExceedsU32();
  }

  return reduced;
}

function gcdBigInt(a: bigint, b: bigint): bigint {
  let left = a;
  let right = b;
  while (right !== 0n) {
    const remainder = left % right;
    left = right;
    right = remainder;
  }
  return left;
}
