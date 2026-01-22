/**
 * Unit Tests for Lightcone Pinocchio SDK
 *
 * Tests PDA derivation, serialization, order management, and utilities
 * without requiring a running Solana cluster.
 */

import { Keypair, PublicKey } from "@solana/web3.js";
import {
  // Constants
  PROGRAM_ID,
  DISCRIMINATOR,
  ACCOUNT_SIZE,
  ORDER_SIZE,
  MAX_OUTCOMES,
  MIN_OUTCOMES,
  MAX_MAKERS,
  SEEDS,
  INSTRUCTION,
  // PDAs
  getExchangePda,
  getMarketPda,
  getVaultPda,
  getMintAuthorityPda,
  getConditionalMintPda,
  getAllConditionalMintPdas,
  getOrderStatusPda,
  getUserNoncePda,
  getPositionPda,
  // Types
  MarketStatus,
  OrderSide,
  // Utilities
  toLeBytes,
  fromLeBytes,
  toBeBytes,
  fromBeBytes,
  toU8,
  toU64Le,
  toI64Le,
  fromI64Le,
  keccak256,
  deriveConditionId,
  validateOutcomes,
  validateOutcomeIndex,
  validate32Bytes,
  serializeString,
  deserializeString,
  // Orders
  hashOrder,
  signOrder,
  signOrderFull,
  verifyOrderSignature,
  serializeFullOrder,
  deserializeFullOrder,
  serializeCompactOrder,
  deserializeCompactOrder,
  createBidOrder,
  createAskOrder,
  isOrderExpired,
  ordersCanCross,
  calculateTakerFill,
  // Account Deserialization
  deserializeExchange,
  deserializeMarket,
  deserializePosition,
  deserializeOrderStatus,
  deserializeUserNonce,
  isExchangeAccount,
  isMarketAccount,
  // Ed25519
  createEd25519VerifyInstruction,
  createBatchEd25519VerifyInstruction,
  // Types for creating orders
  FullOrder,
} from "../src";

// Color output helpers
const colors = {
  reset: "\x1b[0m",
  bright: "\x1b[1m",
  green: "\x1b[32m",
  red: "\x1b[31m",
  yellow: "\x1b[33m",
  cyan: "\x1b[36m",
};

function log(message: string) {
  console.log(`   ${message}`);
}

function pass(testName: string) {
  console.log(`   ${colors.green}✓${colors.reset} ${testName}`);
}

function fail(testName: string, error?: string) {
  console.log(`   ${colors.red}✗${colors.reset} ${testName}`);
  if (error) {
    console.log(`     ${colors.red}Error: ${error}${colors.reset}`);
  }
}

function section(name: string) {
  console.log(`\n${colors.bright}${colors.cyan}▶ ${name}${colors.reset}`);
}

let passCount = 0;
let failCount = 0;

function test(name: string, fn: () => void) {
  try {
    fn();
    pass(name);
    passCount++;
  } catch (err) {
    fail(name, err instanceof Error ? err.message : String(err));
    failCount++;
  }
}

function assertEqual<T>(actual: T, expected: T, message?: string) {
  if (actual !== expected) {
    throw new Error(
      message || `Expected ${expected}, got ${actual}`
    );
  }
}

function assertBufferEqual(actual: Buffer, expected: Buffer, message?: string) {
  if (!actual.equals(expected)) {
    throw new Error(
      message ||
        `Buffers not equal. Expected ${expected.toString("hex")}, got ${actual.toString("hex")}`
    );
  }
}

function assertTrue(value: boolean, message?: string) {
  if (!value) {
    throw new Error(message || "Expected true, got false");
  }
}

function assertFalse(value: boolean, message?: string) {
  if (value) {
    throw new Error(message || "Expected false, got true");
  }
}

function assertThrows(fn: () => void, message?: string) {
  let threw = false;
  try {
    fn();
  } catch {
    threw = true;
  }
  if (!threw) {
    throw new Error(message || "Expected function to throw");
  }
}

async function main() {
  console.log("\n" + "=".repeat(70));
  console.log(`${colors.bright}🧪 LIGHTCONE PINOCCHIO SDK - UNIT TESTS${colors.reset}`);
  console.log("=".repeat(70));

  // ============================================================================
  // Constants Tests
  // ============================================================================
  section("Constants");

  test("PROGRAM_ID is valid PublicKey", () => {
    assertTrue(PROGRAM_ID instanceof PublicKey);
    assertEqual(PROGRAM_ID.toBase58().length, 44); // Base58 encoded public key length
  });

  test("MAX_OUTCOMES is 6", () => {
    assertEqual(MAX_OUTCOMES, 6);
  });

  test("MIN_OUTCOMES is 2", () => {
    assertEqual(MIN_OUTCOMES, 2);
  });

  test("MAX_MAKERS is 5", () => {
    assertEqual(MAX_MAKERS, 5);
  });

  test("DISCRIMINATOR lengths are 8 bytes", () => {
    assertEqual(DISCRIMINATOR.EXCHANGE.length, 8);
    assertEqual(DISCRIMINATOR.MARKET.length, 8);
    assertEqual(DISCRIMINATOR.ORDER_STATUS.length, 8);
    assertEqual(DISCRIMINATOR.USER_NONCE.length, 8);
    assertEqual(DISCRIMINATOR.POSITION.length, 8);
  });

  test("INSTRUCTION discriminators are sequential 0-13", () => {
    assertEqual(INSTRUCTION.INITIALIZE, 0);
    assertEqual(INSTRUCTION.MATCH_ORDERS_MULTI, 13);
  });

  test("ACCOUNT_SIZE values are correct", () => {
    assertEqual(ACCOUNT_SIZE.EXCHANGE, 88);
    assertEqual(ACCOUNT_SIZE.MARKET, 120);
    assertEqual(ACCOUNT_SIZE.ORDER_STATUS, 24);
    assertEqual(ACCOUNT_SIZE.USER_NONCE, 16);
    assertEqual(ACCOUNT_SIZE.POSITION, 80);
  });

  test("ORDER_SIZE values are correct", () => {
    assertEqual(ORDER_SIZE.FULL, 225);
    assertEqual(ORDER_SIZE.COMPACT, 65);
    assertEqual(ORDER_SIZE.SIGNATURE, 64);
  });

  // ============================================================================
  // Buffer Utilities Tests
  // ============================================================================
  section("Buffer Utilities");

  test("toLeBytes converts bigint to little-endian", () => {
    const buf = toLeBytes(0x0102030405060708n, 8);
    assertEqual(buf[0], 0x08);
    assertEqual(buf[7], 0x01);
  });

  test("fromLeBytes converts little-endian to bigint", () => {
    const buf = Buffer.from([0x08, 0x07, 0x06, 0x05, 0x04, 0x03, 0x02, 0x01]);
    const value = fromLeBytes(buf);
    assertEqual(value, 0x0102030405060708n);
  });

  test("toLeBytes and fromLeBytes are inverse operations", () => {
    const original = 123456789012345678n;
    const buf = toLeBytes(original, 8);
    const result = fromLeBytes(buf);
    assertEqual(result, original);
  });

  test("toBeBytes converts bigint to big-endian", () => {
    const buf = toBeBytes(0x0102030405060708n, 8);
    assertEqual(buf[0], 0x01);
    assertEqual(buf[7], 0x08);
  });

  test("fromBeBytes converts big-endian to bigint", () => {
    const buf = Buffer.from([0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]);
    const value = fromBeBytes(buf);
    assertEqual(value, 0x0102030405060708n);
  });

  test("toU8 creates 1-byte buffer", () => {
    const buf = toU8(255);
    assertEqual(buf.length, 1);
    assertEqual(buf[0], 255);
  });

  test("toU64Le creates 8-byte buffer", () => {
    const buf = toU64Le(1000000n);
    assertEqual(buf.length, 8);
  });

  test("toI64Le handles positive values", () => {
    const buf = toI64Le(1000n);
    const result = fromI64Le(buf);
    assertEqual(result, 1000n);
  });

  test("toI64Le handles negative values", () => {
    const buf = toI64Le(-1000n);
    const result = fromI64Le(buf);
    assertEqual(result, -1000n);
  });

  test("toI64Le handles zero", () => {
    const buf = toI64Le(0n);
    const result = fromI64Le(buf);
    assertEqual(result, 0n);
  });

  // ============================================================================
  // String Serialization Tests
  // ============================================================================
  section("String Serialization");

  test("serializeString creates length-prefixed buffer", () => {
    const str = "Hello";
    const buf = serializeString(str);
    assertEqual(buf.length, 2 + 5); // 2 bytes length + 5 chars
    assertEqual(buf.readUInt16LE(0), 5);
  });

  test("deserializeString recovers original string", () => {
    const original = "Test String 123";
    const buf = serializeString(original);
    const [result, bytesConsumed] = deserializeString(buf, 0);
    assertEqual(result, original);
    assertEqual(bytesConsumed, 2 + original.length);
  });

  test("serializeString handles empty string", () => {
    const buf = serializeString("");
    assertEqual(buf.length, 2);
    assertEqual(buf.readUInt16LE(0), 0);
  });

  test("serializeString handles unicode", () => {
    const str = "Hello 世界 🌍";
    const buf = serializeString(str);
    const [result] = deserializeString(buf, 0);
    assertEqual(result, str);
  });

  // ============================================================================
  // Keccak256 Tests
  // ============================================================================
  section("Keccak256 Hashing");

  test("keccak256 produces 32-byte hash", () => {
    const hash = keccak256(Buffer.from("test"));
    assertEqual(hash.length, 32);
  });

  test("keccak256 is deterministic", () => {
    const input = Buffer.from("test data");
    const hash1 = keccak256(input);
    const hash2 = keccak256(input);
    assertBufferEqual(hash1, hash2);
  });

  test("keccak256 produces different hashes for different inputs", () => {
    const hash1 = keccak256(Buffer.from("test1"));
    const hash2 = keccak256(Buffer.from("test2"));
    assertFalse(hash1.equals(hash2));
  });

  test("deriveConditionId produces correct format", () => {
    const oracle = Keypair.generate().publicKey;
    const questionId = Buffer.alloc(32, 1);
    const conditionId = deriveConditionId(oracle, questionId, 2);
    assertEqual(conditionId.length, 32);
  });

  // ============================================================================
  // Validation Tests
  // ============================================================================
  section("Validation Helpers");

  test("validateOutcomes accepts valid range", () => {
    validateOutcomes(2);
    validateOutcomes(3);
    validateOutcomes(6);
  });

  test("validateOutcomes rejects 1 outcome", () => {
    assertThrows(() => validateOutcomes(1));
  });

  test("validateOutcomes rejects 7 outcomes", () => {
    assertThrows(() => validateOutcomes(7));
  });

  test("validateOutcomeIndex accepts valid index", () => {
    validateOutcomeIndex(0, 3);
    validateOutcomeIndex(2, 3);
  });

  test("validateOutcomeIndex rejects negative index", () => {
    assertThrows(() => validateOutcomeIndex(-1, 3));
  });

  test("validateOutcomeIndex rejects index >= numOutcomes", () => {
    assertThrows(() => validateOutcomeIndex(3, 3));
  });

  test("validate32Bytes accepts 32-byte buffer", () => {
    validate32Bytes(Buffer.alloc(32), "test");
  });

  test("validate32Bytes rejects wrong length", () => {
    assertThrows(() => validate32Bytes(Buffer.alloc(31), "test"));
    assertThrows(() => validate32Bytes(Buffer.alloc(33), "test"));
  });

  // ============================================================================
  // PDA Tests
  // ============================================================================
  section("PDA Derivation");

  test("getExchangePda is deterministic", () => {
    const [pda1, bump1] = getExchangePda();
    const [pda2, bump2] = getExchangePda();
    assertTrue(pda1.equals(pda2));
    assertEqual(bump1, bump2);
  });

  test("getExchangePda uses correct seed", () => {
    const [pda] = getExchangePda();
    // Verify it's a valid off-curve point
    assertFalse(PublicKey.isOnCurve(pda.toBytes()));
  });

  test("getMarketPda produces different addresses for different IDs", () => {
    const [pda1] = getMarketPda(0n);
    const [pda2] = getMarketPda(1n);
    assertFalse(pda1.equals(pda2));
  });

  test("getVaultPda includes both mint and market", () => {
    const depositMint = Keypair.generate().publicKey;
    const market = Keypair.generate().publicKey;
    const [pda1] = getVaultPda(depositMint, market);

    // Different mint = different PDA
    const otherMint = Keypair.generate().publicKey;
    const [pda2] = getVaultPda(otherMint, market);
    assertFalse(pda1.equals(pda2));

    // Different market = different PDA
    const otherMarket = Keypair.generate().publicKey;
    const [pda3] = getVaultPda(depositMint, otherMarket);
    assertFalse(pda1.equals(pda3));
  });

  test("getConditionalMintPda varies by outcome index", () => {
    const market = Keypair.generate().publicKey;
    const depositMint = Keypair.generate().publicKey;
    const [pda0] = getConditionalMintPda(market, depositMint, 0);
    const [pda1] = getConditionalMintPda(market, depositMint, 1);
    assertFalse(pda0.equals(pda1));
  });

  test("getAllConditionalMintPdas returns correct count", () => {
    const market = Keypair.generate().publicKey;
    const depositMint = Keypair.generate().publicKey;
    const pdas = getAllConditionalMintPdas(market, depositMint, 3);
    assertEqual(pdas.length, 3);
  });

  test("getOrderStatusPda requires 32-byte hash", () => {
    const validHash = Buffer.alloc(32);
    getOrderStatusPda(validHash); // Should not throw

    assertThrows(() => getOrderStatusPda(Buffer.alloc(31)));
  });

  test("getPositionPda does NOT include deposit mint", () => {
    const owner = Keypair.generate().publicKey;
    const market = Keypair.generate().publicKey;
    const [pda] = getPositionPda(owner, market);

    // Same owner + market = same PDA regardless of any other factor
    const [pda2] = getPositionPda(owner, market);
    assertTrue(pda.equals(pda2));
  });

  test("getUserNoncePda is unique per user", () => {
    const user1 = Keypair.generate().publicKey;
    const user2 = Keypair.generate().publicKey;
    const [pda1] = getUserNoncePda(user1);
    const [pda2] = getUserNoncePda(user2);
    assertFalse(pda1.equals(pda2));
  });

  // ============================================================================
  // Order Tests
  // ============================================================================
  section("Order Management");

  const createTestOrder = (): FullOrder => ({
    nonce: 0n,
    maker: Keypair.generate().publicKey,
    market: Keypair.generate().publicKey,
    baseMint: Keypair.generate().publicKey,
    quoteMint: Keypair.generate().publicKey,
    side: OrderSide.BID,
    makerAmount: 1000000n,
    takerAmount: 500000n,
    expiration: 0n,
    signature: Buffer.alloc(64),
  });

  test("hashOrder produces 32-byte hash", () => {
    const order = createTestOrder();
    const hash = hashOrder(order);
    assertEqual(hash.length, 32);
  });

  test("hashOrder is deterministic", () => {
    const signer = Keypair.generate();
    const order: FullOrder = {
      nonce: 123n,
      maker: signer.publicKey,
      market: Keypair.generate().publicKey,
      baseMint: Keypair.generate().publicKey,
      quoteMint: Keypair.generate().publicKey,
      side: OrderSide.ASK,
      makerAmount: 1000n,
      takerAmount: 2000n,
      expiration: 1234567890n,
      signature: Buffer.alloc(64),
    };
    const hash1 = hashOrder(order);
    const hash2 = hashOrder(order);
    assertBufferEqual(hash1, hash2);
  });

  test("signOrder produces 64-byte signature", () => {
    const signer = Keypair.generate();
    const order = createTestOrder();
    order.maker = signer.publicKey;
    const signature = signOrder(order, signer);
    assertEqual(signature.length, 64);
  });

  test("signOrderFull returns complete signed order", () => {
    const signer = Keypair.generate();
    const unsignedOrder = {
      nonce: 0n,
      maker: signer.publicKey,
      market: Keypair.generate().publicKey,
      baseMint: Keypair.generate().publicKey,
      quoteMint: Keypair.generate().publicKey,
      side: OrderSide.BID,
      makerAmount: 1000n,
      takerAmount: 500n,
      expiration: 0n,
    };
    const signedOrder = signOrderFull(unsignedOrder, signer);
    assertEqual(signedOrder.signature.length, 64);
    assertTrue(signedOrder.maker.equals(signer.publicKey));
  });

  test("verifyOrderSignature validates correct signature", () => {
    const signer = Keypair.generate();
    const unsignedOrder = {
      nonce: 0n,
      maker: signer.publicKey,
      market: Keypair.generate().publicKey,
      baseMint: Keypair.generate().publicKey,
      quoteMint: Keypair.generate().publicKey,
      side: OrderSide.BID,
      makerAmount: 1000n,
      takerAmount: 500n,
      expiration: 0n,
    };
    const signedOrder = signOrderFull(unsignedOrder, signer);
    assertTrue(verifyOrderSignature(signedOrder));
  });

  test("verifyOrderSignature rejects invalid signature", () => {
    const order = createTestOrder();
    order.signature = Buffer.alloc(64, 1); // Invalid signature
    assertFalse(verifyOrderSignature(order));
  });

  test("serializeFullOrder produces 225 bytes", () => {
    const order = createTestOrder();
    const serialized = serializeFullOrder(order);
    assertEqual(serialized.length, ORDER_SIZE.FULL);
  });

  test("deserializeFullOrder recovers original order", () => {
    const original = createTestOrder();
    const serialized = serializeFullOrder(original);
    const recovered = deserializeFullOrder(serialized);

    assertEqual(recovered.nonce, original.nonce);
    assertTrue(recovered.maker.equals(original.maker));
    assertTrue(recovered.market.equals(original.market));
    assertEqual(recovered.side, original.side);
    assertEqual(recovered.makerAmount, original.makerAmount);
    assertEqual(recovered.takerAmount, original.takerAmount);
    assertEqual(recovered.expiration, original.expiration);
  });

  test("serializeCompactOrder produces 65 bytes", () => {
    const order = {
      nonce: 0n,
      maker: Keypair.generate().publicKey,
      side: OrderSide.BID,
      makerAmount: 1000n,
      takerAmount: 500n,
      expiration: 0n,
    };
    const serialized = serializeCompactOrder(order);
    assertEqual(serialized.length, ORDER_SIZE.COMPACT);
  });

  test("deserializeCompactOrder recovers original", () => {
    const original = {
      nonce: 123n,
      maker: Keypair.generate().publicKey,
      side: OrderSide.ASK,
      makerAmount: 1000000n,
      takerAmount: 500000n,
      expiration: 1234567890n,
    };
    const serialized = serializeCompactOrder(original);
    const recovered = deserializeCompactOrder(serialized);

    assertEqual(recovered.nonce, original.nonce);
    assertTrue(recovered.maker.equals(original.maker));
    assertEqual(recovered.side, original.side);
    assertEqual(recovered.makerAmount, original.makerAmount);
    assertEqual(recovered.takerAmount, original.takerAmount);
    assertEqual(recovered.expiration, original.expiration);
  });

  test("createBidOrder sets side to BID", () => {
    const order = createBidOrder({
      nonce: 0n,
      maker: Keypair.generate().publicKey,
      market: Keypair.generate().publicKey,
      baseMint: Keypair.generate().publicKey,
      quoteMint: Keypair.generate().publicKey,
      makerAmount: 1000n,
      takerAmount: 500n,
    });
    assertEqual(order.side, OrderSide.BID);
  });

  test("createAskOrder sets side to ASK", () => {
    const order = createAskOrder({
      nonce: 0n,
      maker: Keypair.generate().publicKey,
      market: Keypair.generate().publicKey,
      baseMint: Keypair.generate().publicKey,
      quoteMint: Keypair.generate().publicKey,
      makerAmount: 1000n,
      takerAmount: 500n,
    });
    assertEqual(order.side, OrderSide.ASK);
  });

  test("isOrderExpired returns false for 0 expiration", () => {
    const order = createTestOrder();
    order.expiration = 0n;
    assertFalse(isOrderExpired(order));
  });

  test("isOrderExpired returns true for past timestamp", () => {
    const order = createTestOrder();
    order.expiration = 1n; // 1970
    assertTrue(isOrderExpired(order));
  });

  test("isOrderExpired returns false for future timestamp", () => {
    const order = createTestOrder();
    order.expiration = BigInt(Math.floor(Date.now() / 1000) + 3600); // 1 hour from now
    assertFalse(isOrderExpired(order));
  });

  test("ordersCanCross returns true for matching orders", () => {
    const market = Keypair.generate().publicKey;
    const baseMint = Keypair.generate().publicKey;
    const quoteMint = Keypair.generate().publicKey;

    const buyOrder: FullOrder = {
      ...createTestOrder(),
      market,
      baseMint,
      quoteMint,
      side: OrderSide.BID,
      makerAmount: 100n, // Willing to pay 100 quote
      takerAmount: 50n,  // For 50 base
    };

    const sellOrder: FullOrder = {
      ...createTestOrder(),
      market,
      baseMint,
      quoteMint,
      side: OrderSide.ASK,
      makerAmount: 50n,  // Selling 50 base
      takerAmount: 90n,  // For 90 quote (less than buyer offers)
    };

    assertTrue(ordersCanCross(buyOrder, sellOrder));
  });

  test("ordersCanCross returns false for non-crossing orders", () => {
    const buyOrder: FullOrder = {
      ...createTestOrder(),
      side: OrderSide.BID,
      makerAmount: 50n,   // Willing to pay 50 quote
      takerAmount: 100n,  // For 100 base
    };

    const sellOrder: FullOrder = {
      ...createTestOrder(),
      side: OrderSide.ASK,
      makerAmount: 100n,  // Selling 100 base
      takerAmount: 100n,  // For 100 quote (more than buyer offers)
    };

    assertFalse(ordersCanCross(buyOrder, sellOrder));
  });

  test("calculateTakerFill computes correct amount", () => {
    const makerOrder: FullOrder = {
      ...createTestOrder(),
      makerAmount: 100n,
      takerAmount: 50n,
    };
    const makerFill = 50n; // Maker gives 50
    const takerFill = calculateTakerFill(makerOrder, makerFill);
    assertEqual(takerFill, 25n); // 50 * 50 / 100 = 25
  });

  // ============================================================================
  // Account Deserialization Tests
  // ============================================================================
  section("Account Deserialization");

  test("isExchangeAccount validates discriminator", () => {
    const validData = Buffer.concat([
      DISCRIMINATOR.EXCHANGE,
      Buffer.alloc(ACCOUNT_SIZE.EXCHANGE - 8),
    ]);
    assertTrue(isExchangeAccount(validData));

    const invalidData = Buffer.alloc(ACCOUNT_SIZE.EXCHANGE);
    assertFalse(isExchangeAccount(invalidData));
  });

  test("isMarketAccount validates discriminator", () => {
    const validData = Buffer.concat([
      DISCRIMINATOR.MARKET,
      Buffer.alloc(ACCOUNT_SIZE.MARKET - 8),
    ]);
    assertTrue(isMarketAccount(validData));
  });

  test("deserializeExchange parses all fields", () => {
    const authority = Keypair.generate().publicKey;
    const operator = Keypair.generate().publicKey;

    const data = Buffer.alloc(ACCOUNT_SIZE.EXCHANGE);
    let offset = 0;

    // discriminator
    DISCRIMINATOR.EXCHANGE.copy(data, offset);
    offset += 8;

    // authority
    authority.toBuffer().copy(data, offset);
    offset += 32;

    // operator
    operator.toBuffer().copy(data, offset);
    offset += 32;

    // market_count (u64 LE)
    toU64Le(5n).copy(data, offset);
    offset += 8;

    // paused (u8)
    data[offset] = 0;
    offset += 1;

    // bump (u8)
    data[offset] = 255;

    const exchange = deserializeExchange(data);

    assertTrue(exchange.authority.equals(authority));
    assertTrue(exchange.operator.equals(operator));
    assertEqual(exchange.marketCount, 5n);
    assertFalse(exchange.paused);
    assertEqual(exchange.bump, 255);
  });

  test("deserializeMarket parses all fields", () => {
    const oracle = Keypair.generate().publicKey;
    const questionId = Buffer.alloc(32, 0xAB);
    const conditionId = Buffer.alloc(32, 0xCD);

    const data = Buffer.alloc(ACCOUNT_SIZE.MARKET);
    let offset = 0;

    // discriminator
    DISCRIMINATOR.MARKET.copy(data, offset);
    offset += 8;

    // market_id
    toU64Le(42n).copy(data, offset);
    offset += 8;

    // num_outcomes
    data[offset++] = 3;

    // status (Active = 1)
    data[offset++] = 1;

    // winning_outcome
    data[offset++] = 2;

    // has_winning_outcome
    data[offset++] = 1;

    // bump
    data[offset++] = 254;

    // padding (3 bytes)
    offset += 3;

    // oracle
    oracle.toBuffer().copy(data, offset);
    offset += 32;

    // question_id
    questionId.copy(data, offset);
    offset += 32;

    // condition_id
    conditionId.copy(data, offset);

    const market = deserializeMarket(data);

    assertEqual(market.marketId, 42n);
    assertEqual(market.numOutcomes, 3);
    assertEqual(market.status, MarketStatus.Active);
    assertEqual(market.winningOutcome, 2);
    assertTrue(market.hasWinningOutcome);
    assertEqual(market.bump, 254);
    assertTrue(market.oracle.equals(oracle));
    assertBufferEqual(market.questionId, questionId);
    assertBufferEqual(market.conditionId, conditionId);
  });

  test("deserializeOrderStatus parses all fields", () => {
    const data = Buffer.alloc(ACCOUNT_SIZE.ORDER_STATUS);

    DISCRIMINATOR.ORDER_STATUS.copy(data, 0);
    toU64Le(1000n).copy(data, 8);
    data[16] = 1; // is_cancelled

    const orderStatus = deserializeOrderStatus(data);

    assertEqual(orderStatus.remaining, 1000n);
    assertTrue(orderStatus.isCancelled);
  });

  test("deserializeUserNonce parses all fields", () => {
    const data = Buffer.alloc(ACCOUNT_SIZE.USER_NONCE);

    DISCRIMINATOR.USER_NONCE.copy(data, 0);
    toU64Le(42n).copy(data, 8);

    const userNonce = deserializeUserNonce(data);

    assertEqual(userNonce.nonce, 42n);
  });

  test("deserializePosition parses all fields", () => {
    const owner = Keypair.generate().publicKey;
    const market = Keypair.generate().publicKey;

    const data = Buffer.alloc(ACCOUNT_SIZE.POSITION);

    DISCRIMINATOR.POSITION.copy(data, 0);
    owner.toBuffer().copy(data, 8);
    market.toBuffer().copy(data, 40);
    data[72] = 253; // bump

    const position = deserializePosition(data);

    assertTrue(position.owner.equals(owner));
    assertTrue(position.market.equals(market));
    assertEqual(position.bump, 253);
  });

  test("deserialization rejects invalid discriminators", () => {
    const invalidData = Buffer.alloc(ACCOUNT_SIZE.EXCHANGE);
    assertThrows(() => deserializeExchange(invalidData));
  });

  test("deserialization rejects short data", () => {
    const shortData = Buffer.alloc(10);
    assertThrows(() => deserializeExchange(shortData));
    assertThrows(() => deserializeMarket(shortData));
  });

  // ============================================================================
  // Ed25519 Tests
  // ============================================================================
  section("Ed25519 Signature Helpers");

  test("createEd25519VerifyInstruction creates valid instruction", () => {
    const publicKey = Keypair.generate().publicKey;
    const message = Buffer.alloc(32, 0xAB);
    const signature = Buffer.alloc(64, 0xCD);

    const ix = createEd25519VerifyInstruction({
      publicKey,
      message,
      signature,
    });

    // Ed25519 instructions have no accounts
    assertEqual(ix.keys.length, 0);
    // Data should be 144 bytes (16 header + 64 sig + 32 pubkey + 32 message)
    assertEqual(ix.data.length, 144);
  });

  test("createEd25519VerifyInstruction rejects invalid signature length", () => {
    assertThrows(() =>
      createEd25519VerifyInstruction({
        publicKey: Keypair.generate().publicKey,
        message: Buffer.alloc(32),
        signature: Buffer.alloc(63), // Invalid
      })
    );
  });

  test("createEd25519VerifyInstruction rejects invalid message length", () => {
    assertThrows(() =>
      createEd25519VerifyInstruction({
        publicKey: Keypair.generate().publicKey,
        message: Buffer.alloc(31), // Invalid
        signature: Buffer.alloc(64),
      })
    );
  });

  test("createBatchEd25519VerifyInstruction handles multiple signatures", () => {
    const params = [
      {
        publicKey: Keypair.generate().publicKey,
        message: Buffer.alloc(32, 0x01),
        signature: Buffer.alloc(64, 0x01),
      },
      {
        publicKey: Keypair.generate().publicKey,
        message: Buffer.alloc(32, 0x02),
        signature: Buffer.alloc(64, 0x02),
      },
    ];

    const ix = createBatchEd25519VerifyInstruction(params);

    assertEqual(ix.keys.length, 0);
    // Header: 2 + 14*2 = 30 bytes
    // Data: 2 * (64 + 32 + 32) = 256 bytes
    // Total: 286 bytes
    assertEqual(ix.data.length, 30 + 256);
  });

  test("createBatchEd25519VerifyInstruction rejects empty array", () => {
    assertThrows(() => createBatchEd25519VerifyInstruction([]));
  });

  // ============================================================================
  // Enums Tests
  // ============================================================================
  section("Enums");

  test("MarketStatus has correct values", () => {
    assertEqual(MarketStatus.Pending, 0);
    assertEqual(MarketStatus.Active, 1);
    assertEqual(MarketStatus.Resolved, 2);
    assertEqual(MarketStatus.Cancelled, 3);
  });

  test("OrderSide has correct values", () => {
    assertEqual(OrderSide.BID, 0);
    assertEqual(OrderSide.ASK, 1);
  });

  // ============================================================================
  // Summary
  // ============================================================================
  console.log("\n" + "=".repeat(70));
  if (failCount === 0) {
    console.log(
      `${colors.bright}${colors.green}🎉 ALL ${passCount} TESTS PASSED!${colors.reset}`
    );
  } else {
    console.log(
      `${colors.bright}${colors.red}❌ ${failCount} TESTS FAILED${colors.reset} (${passCount} passed)`
    );
  }
  console.log("=".repeat(70) + "\n");

  process.exit(failCount > 0 ? 1 : 0);
}

main().catch((err) => {
  console.error(`${colors.red}Fatal error:${colors.reset}`, err);
  process.exit(1);
});
