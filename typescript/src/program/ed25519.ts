import {
  PublicKey,
  TransactionInstruction,
  Transaction,
} from "@solana/web3.js";
import { ED25519_PROGRAM_ID, PROGRAM_ID } from "./constants";
import { FullOrder, MatchOrdersMultiParams } from "./types";
import { hashOrder } from "./orders";
import { buildMatchOrdersMultiIx } from "./instructions";

// ============================================================================
// ED25519 INSTRUCTION HELPERS
// ============================================================================

/**
 * Parameters for creating an Ed25519 verification instruction
 */
export interface Ed25519VerifyParams {
  /** The public key that signed the message */
  publicKey: PublicKey;
  /** The message that was signed (32-byte order hash) */
  message: Buffer;
  /** The Ed25519 signature (64 bytes) */
  signature: Buffer;
}

/**
 * Create an Ed25519 signature verification instruction
 *
 * This instruction must precede the matchOrdersMulti instruction in the transaction.
 * The Solana Ed25519 program will verify the signature and the matchOrdersMulti
 * instruction will read the verification result from the instructions sysvar.
 *
 * Ed25519 instruction data format:
 * - num_signatures (u8): 1
 * - padding (u8): 0
 * - signature_offset (u16): 16
 * - signature_instruction_index (u16): 0xFFFF (same instruction)
 * - public_key_offset (u16): 80
 * - public_key_instruction_index (u16): 0xFFFF
 * - message_data_offset (u16): 112
 * - message_data_size (u16): 32
 * - message_instruction_index (u16): 0xFFFF
 * - signature (64 bytes)
 * - public_key (32 bytes)
 * - message (32 bytes)
 */
export function createEd25519VerifyInstruction(
  params: Ed25519VerifyParams
): TransactionInstruction {
  if (params.signature.length !== 64) {
    throw new Error(
      `Invalid signature length: ${params.signature.length}, expected 64`
    );
  }
  if (params.message.length !== 32) {
    throw new Error(
      `Invalid message length: ${params.message.length}, expected 32`
    );
  }

  // Header: 2 bytes (num_signatures, padding) + 14 bytes per signature = 16 bytes total header
  const header = Buffer.alloc(16);
  let offset = 0;

  // num_signatures (u8)
  header[offset] = 1;
  offset += 1;

  // padding (u8)
  header[offset] = 0;
  offset += 1;

  // signature_offset (u16 LE) - starts at byte 16
  header.writeUInt16LE(16, offset);
  offset += 2;

  // signature_instruction_index (u16 LE) - 0xFFFF = same instruction
  header.writeUInt16LE(0xffff, offset);
  offset += 2;

  // public_key_offset (u16 LE) - starts at byte 80 (16 + 64)
  header.writeUInt16LE(80, offset);
  offset += 2;

  // public_key_instruction_index (u16 LE)
  header.writeUInt16LE(0xffff, offset);
  offset += 2;

  // message_data_offset (u16 LE) - starts at byte 112 (16 + 64 + 32)
  header.writeUInt16LE(112, offset);
  offset += 2;

  // message_data_size (u16 LE)
  header.writeUInt16LE(32, offset);
  offset += 2;

  // message_instruction_index (u16 LE)
  header.writeUInt16LE(0xffff, offset);
  offset += 2;

  // Combine: header (16) + signature (64) + pubkey (32) + message (32) = 144 bytes
  const data = Buffer.concat([
    header,
    params.signature,
    params.publicKey.toBuffer(),
    params.message,
  ]);

  return new TransactionInstruction({
    keys: [],
    programId: ED25519_PROGRAM_ID,
    data,
  });
}

/**
 * Create Ed25519 verification instructions for multiple signatures
 */
export function createEd25519VerifyInstructions(
  params: Ed25519VerifyParams[]
): TransactionInstruction[] {
  return params.map(createEd25519VerifyInstruction);
}

/**
 * Create an Ed25519 verification instruction for an order
 */
export function createOrderVerifyInstruction(
  order: FullOrder
): TransactionInstruction {
  const message = hashOrder(order);
  return createEd25519VerifyInstruction({
    publicKey: order.maker,
    message,
    signature: order.signature,
  });
}

// ============================================================================
// MATCH ORDERS TRANSACTION BUILDER
// ============================================================================

/**
 * Build a complete matchOrdersMulti transaction with Ed25519 pre-instructions
 *
 * The transaction will contain:
 * 1. Ed25519 verify instruction for taker order
 * 2. Ed25519 verify instructions for each maker order
 * 3. matchOrdersMulti instruction
 *
 * @param params - Match orders parameters
 * @param programId - Program ID (defaults to PROGRAM_ID)
 * @returns Transaction with all required instructions
 */
export function buildMatchOrdersTransaction(
  params: MatchOrdersMultiParams,
  programId: PublicKey = PROGRAM_ID
): Transaction {
  const transaction = new Transaction();

  // 1. Add taker Ed25519 verify instruction
  const takerVerifyIx = createOrderVerifyInstruction(params.takerOrder);
  transaction.add(takerVerifyIx);

  // 2. Add maker Ed25519 verify instructions
  for (const makerOrder of params.makerOrders) {
    const makerVerifyIx = createOrderVerifyInstruction(makerOrder);
    transaction.add(makerVerifyIx);
  }

  // 3. Add matchOrdersMulti instruction
  const matchIx = buildMatchOrdersMultiIx(params, programId);
  transaction.add(matchIx);

  return transaction;
}

/**
 * Helper to create verification params from an order
 */
export function orderToVerifyParams(order: FullOrder): Ed25519VerifyParams {
  return {
    publicKey: order.maker,
    message: hashOrder(order),
    signature: order.signature,
  };
}

// ============================================================================
// BATCH SIGNATURE VERIFICATION
// ============================================================================

/**
 * Create a single Ed25519 instruction that verifies multiple signatures
 * This is more efficient than multiple single-signature instructions
 *
 * Note: Each additional signature adds 14 bytes header + 64 sig + 32 pubkey + message_size
 * For order hashes (32 bytes), that's 14 + 64 + 32 + 32 = 142 bytes per signature
 */
export function createBatchEd25519VerifyInstruction(
  params: Ed25519VerifyParams[]
): TransactionInstruction {
  if (params.length === 0) {
    throw new Error("At least one signature is required");
  }

  // Check all messages are 32 bytes
  for (const p of params) {
    if (p.signature.length !== 64) {
      throw new Error(
        `Invalid signature length: ${p.signature.length}, expected 64`
      );
    }
    if (p.message.length !== 32) {
      throw new Error(
        `Invalid message length: ${p.message.length}, expected 32`
      );
    }
  }

  const numSignatures = params.length;

  // Calculate header size: 2 bytes base + 14 bytes per signature
  const headerSize = 2 + numSignatures * 14;

  // Calculate offsets for each signature's data
  // Data layout after header: [sig0, pubkey0, msg0, sig1, pubkey1, msg1, ...]
  // Each entry: 64 + 32 + 32 = 128 bytes
  const entrySize = 64 + 32 + 32;

  const header = Buffer.alloc(headerSize);
  let headerOffset = 0;

  // num_signatures (u8)
  header[headerOffset] = numSignatures;
  headerOffset += 1;

  // padding (u8)
  header[headerOffset] = 0;
  headerOffset += 1;

  // Per-signature header entries
  for (let i = 0; i < numSignatures; i++) {
    const dataStart = headerSize + i * entrySize;

    // signature_offset (u16 LE)
    header.writeUInt16LE(dataStart, headerOffset);
    headerOffset += 2;

    // signature_instruction_index (u16 LE)
    header.writeUInt16LE(0xffff, headerOffset);
    headerOffset += 2;

    // public_key_offset (u16 LE)
    header.writeUInt16LE(dataStart + 64, headerOffset);
    headerOffset += 2;

    // public_key_instruction_index (u16 LE)
    header.writeUInt16LE(0xffff, headerOffset);
    headerOffset += 2;

    // message_data_offset (u16 LE)
    header.writeUInt16LE(dataStart + 64 + 32, headerOffset);
    headerOffset += 2;

    // message_data_size (u16 LE)
    header.writeUInt16LE(32, headerOffset);
    headerOffset += 2;

    // message_instruction_index (u16 LE)
    header.writeUInt16LE(0xffff, headerOffset);
    headerOffset += 2;
  }

  // Build data section
  const dataBuffers: Buffer[] = [header];
  for (const p of params) {
    dataBuffers.push(p.signature);
    dataBuffers.push(p.publicKey.toBuffer());
    dataBuffers.push(p.message);
  }

  const data = Buffer.concat(dataBuffers);

  return new TransactionInstruction({
    keys: [],
    programId: ED25519_PROGRAM_ID,
    data,
  });
}

/**
 * Build a compact matchOrdersMulti transaction using batch Ed25519 verification
 * This uses a single Ed25519 instruction to verify all signatures at once
 *
 * @deprecated Use buildCrossRefMatchOrdersTransaction for smaller transactions
 */
export function buildCompactMatchOrdersTransaction(
  params: MatchOrdersMultiParams,
  programId: PublicKey = PROGRAM_ID
): Transaction {
  const transaction = new Transaction();

  // Collect all verify params
  const verifyParams: Ed25519VerifyParams[] = [
    orderToVerifyParams(params.takerOrder),
  ];
  for (const makerOrder of params.makerOrders) {
    verifyParams.push(orderToVerifyParams(makerOrder));
  }

  // 1. Add batch Ed25519 verify instruction
  const batchVerifyIx = createBatchEd25519VerifyInstruction(verifyParams);
  transaction.add(batchVerifyIx);

  // 2. Add matchOrdersMulti instruction
  const matchIx = buildMatchOrdersMultiIx(params, programId);
  transaction.add(matchIx);

  return transaction;
}

// ============================================================================
// CROSS-INSTRUCTION REFERENCE ED25519 VERIFICATION
// ============================================================================

/**
 * Match instruction data layout offsets (for cross-instruction references)
 *
 * Layout for single maker (332 bytes):
 * [0]:        discriminator (1 byte)
 * [1..33]:    taker_hash (32 bytes)        <- taker message
 * [33..98]:   taker_compact (65 bytes)     <- taker pubkey at offset 33+8=41
 * [98..162]:  taker_signature (64 bytes)   <- taker signature
 * [162]:      num_makers (1 byte)
 * [163..195]: maker_hash (32 bytes)        <- maker message
 * [195..260]: maker_compact (65 bytes)     <- maker pubkey at offset 195+8=203
 * [260..324]: maker_signature (64 bytes)   <- maker signature
 * [324..332]: maker_fill_amount (8 bytes)
 */
const MATCH_IX_OFFSETS = {
  TAKER_MESSAGE: 1,      // taker_hash starts at offset 1
  TAKER_PUBKEY: 41,      // taker_compact.maker at 33+8
  TAKER_SIGNATURE: 98,   // taker_signature at offset 98
  NUM_MAKERS: 162,       // num_makers at offset 162
  // Per-maker offsets are calculated dynamically
} as const;

/**
 * Calculate offsets for a maker order in the match instruction data
 * Each maker entry is: hash(32) + compact(65) + sig(64) + fill(8) = 169 bytes
 */
function getMakerOffsets(makerIndex: number): {
  message: number;
  pubkey: number;
  signature: number;
} {
  const makerBaseOffset = 163 + makerIndex * 169;
  return {
    message: makerBaseOffset,           // maker_hash
    pubkey: makerBaseOffset + 32 + 8,   // maker_compact.maker (32 for hash + 8 for nonce)
    signature: makerBaseOffset + 32 + 65, // maker_signature (32 for hash + 65 for compact)
  };
}

/**
 * Create Ed25519 verification instructions using cross-instruction references
 *
 * Instead of embedding signature/pubkey/message data in the Ed25519 instruction,
 * this points to offsets within the matchOrdersMulti instruction data.
 *
 * This saves ~256 bytes for 2 orders (128 bytes per order: 64 sig + 32 pubkey + 32 msg)
 *
 * @param numMakers - Number of maker orders
 * @param matchIxIndex - Index of the matchOrdersMulti instruction in the transaction
 * @returns Array of Ed25519 verification instructions (taker first, then makers)
 */
export function createCrossRefEd25519Instructions(
  numMakers: number,
  matchIxIndex: number
): TransactionInstruction[] {
  const instructions: TransactionInstruction[] = [];

  // Taker verification instruction
  const takerIx = createCrossRefEd25519Instruction({
    signatureOffset: MATCH_IX_OFFSETS.TAKER_SIGNATURE,
    signatureIxIndex: matchIxIndex,
    pubkeyOffset: MATCH_IX_OFFSETS.TAKER_PUBKEY,
    pubkeyIxIndex: matchIxIndex,
    messageOffset: MATCH_IX_OFFSETS.TAKER_MESSAGE,
    messageSize: 32,
    messageIxIndex: matchIxIndex,
  });
  instructions.push(takerIx);

  // Maker verification instructions
  for (let i = 0; i < numMakers; i++) {
    const offsets = getMakerOffsets(i);
    const makerIx = createCrossRefEd25519Instruction({
      signatureOffset: offsets.signature,
      signatureIxIndex: matchIxIndex,
      pubkeyOffset: offsets.pubkey,
      pubkeyIxIndex: matchIxIndex,
      messageOffset: offsets.message,
      messageSize: 32,
      messageIxIndex: matchIxIndex,
    });
    instructions.push(makerIx);
  }

  return instructions;
}

/**
 * Parameters for cross-instruction Ed25519 verification
 */
interface CrossRefEd25519Params {
  signatureOffset: number;
  signatureIxIndex: number;
  pubkeyOffset: number;
  pubkeyIxIndex: number;
  messageOffset: number;
  messageSize: number;
  messageIxIndex: number;
}

/**
 * Create a single Ed25519 instruction that references data in another instruction
 *
 * Ed25519 header format (16 bytes for 1 signature):
 * - num_signatures (u8): 1
 * - padding (u8): 0
 * - signature_offset (u16)
 * - signature_instruction_index (u16)
 * - public_key_offset (u16)
 * - public_key_instruction_index (u16)
 * - message_data_offset (u16)
 * - message_data_size (u16)
 * - message_instruction_index (u16)
 *
 * Total: 16 bytes (no signature/pubkey/message data embedded)
 */
function createCrossRefEd25519Instruction(
  params: CrossRefEd25519Params
): TransactionInstruction {
  const data = Buffer.alloc(16);
  let offset = 0;

  // num_signatures (u8)
  data[offset] = 1;
  offset += 1;

  // padding (u8)
  data[offset] = 0;
  offset += 1;

  // signature_offset (u16 LE)
  data.writeUInt16LE(params.signatureOffset, offset);
  offset += 2;

  // signature_instruction_index (u16 LE)
  data.writeUInt16LE(params.signatureIxIndex, offset);
  offset += 2;

  // public_key_offset (u16 LE)
  data.writeUInt16LE(params.pubkeyOffset, offset);
  offset += 2;

  // public_key_instruction_index (u16 LE)
  data.writeUInt16LE(params.pubkeyIxIndex, offset);
  offset += 2;

  // message_data_offset (u16 LE)
  data.writeUInt16LE(params.messageOffset, offset);
  offset += 2;

  // message_data_size (u16 LE)
  data.writeUInt16LE(params.messageSize, offset);
  offset += 2;

  // message_instruction_index (u16 LE)
  data.writeUInt16LE(params.messageIxIndex, offset);

  return new TransactionInstruction({
    keys: [],
    programId: ED25519_PROGRAM_ID,
    data,
  });
}

/**
 * Build a matchOrdersMulti transaction using cross-instruction Ed25519 references
 *
 * This is the most space-efficient approach:
 * - Ed25519 instructions are only 16 bytes each (just header with offsets)
 * - Signature/pubkey/message data is only stored once in the match instruction
 *
 * Transaction layout:
 * [0]: Ed25519 verify for taker (16 bytes, refs instruction 2)
 * [1]: Ed25519 verify for maker (16 bytes, refs instruction 2)
 * [2]: matchOrdersMulti (contains all order data)
 *
 * @param params - Match orders parameters
 * @param programId - Program ID (defaults to PROGRAM_ID)
 * @returns Transaction with all required instructions (~1,040 bytes for single maker)
 */
export function buildCrossRefMatchOrdersTransaction(
  params: MatchOrdersMultiParams,
  programId: PublicKey = PROGRAM_ID
): Transaction {
  const transaction = new Transaction();

  const numMakers = params.makerOrders.length;

  // The match instruction will be at index (1 + numMakers)
  // Taker verify at 0, maker verifies at 1..numMakers, match at numMakers+1
  const matchIxIndex = 1 + numMakers;

  // 1. Add Ed25519 verification instructions (cross-ref to match instruction)
  const verifyIxs = createCrossRefEd25519Instructions(numMakers, matchIxIndex);
  for (const ix of verifyIxs) {
    transaction.add(ix);
  }

  // 2. Add matchOrdersMulti instruction
  const matchIx = buildMatchOrdersMultiIx(params, programId);
  transaction.add(matchIx);

  return transaction;
}
