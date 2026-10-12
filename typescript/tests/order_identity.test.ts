import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { Keypair, PublicKey } from "@solana/web3.js";
import bs58 from "bs58";
import { sign } from "tweetnacl";

import {
  DepositSource,
  I64_MAX,
  ScalingError,
  TimeInForce,
  U64_MAX,
  scalePriceSize,
  stringifyJsonExact,
  validateRawAmounts,
  validateSignedFields,
  type OrderbookRules,
  type SubmitOrderRequest,
} from "../src/shared";
import { orderbookRulesFromWire } from "../src/domain/orderbook/wire";
import type { OrderBookPair } from "../src/domain/orderbook";
import { ORDER_SIZE } from "../src/program/constants";
import { OrderBuilder } from "../src/program/builder";
import { LimitOrderEnvelope } from "../src/program/envelope";
import { ProgramSdkError } from "../src/program/error";
import { buildCloseOrderStatusIx } from "../src/program/instructions";
import {
  calculateTakerFill,
  cancelAllMessage,
  deserializeOrder,
  deserializeSignedOrder,
  generateSalt,
  hashOrderHex,
  isOrderExpired,
  isSigned,
  orderPreimage,
  orderToSigned,
  serializeOrder,
  serializeSignedOrder,
  signCancelAll,
  signOrderFull,
  signedOrderToOrder,
  toSubmitRequest,
  verifyOrderSignature,
} from "../src/program/orders";
import { V1Transaction } from "../src/program/transaction";
import { OrderSide, type SignedOrder } from "../src/program/types";

/** Unit rules: whole atoms, six price decimals, every positive ratio on the grid. */
function unitRules(orderbookId = "unit"): OrderbookRules {
  return orderbookRulesFromWire({
    orderbook_id: orderbookId,
    base_decimals: 0,
    quote_decimals: 0,
    price_decimals: 6,
    trading_rules: {
      base_size_decimals: 0,
      max_price_decimals: 6,
      max_price_significant_figures: 5,
      integer_prices_always_allowed: true,
      price_quantum: "0.000001",
      price_quantum_raw: "1",
      base_size_quantum: "1",
      base_size_quantum_raw: "1",
    },
  });
}

// Backend signing vectors (fixtures/program-contract/v1/signing.json),
// generated independently of the SDKs with Python struct, Keccak and Ed25519,
// and pinned by the Rust SDK too. Every vector is signed by seed 00..1f, whose
// public key is the maker, on market [0x11; 32], base mint [0x22; 32] and
// quote mint [0x33; 32]. Amounts are raw atoms; expiration is Unix seconds.
const SIGNING_SEED_HEX = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
const SIGNING_MAKER_HEX = "03a107bff3ce10be1d70dd18e74bc09967e4d6309ba50d5f1ddc8664125531b8";
const PREIMAGE_ACCOUNTS =
  SIGNING_MAKER_HEX + "11".repeat(32) + "22".repeat(32) + "33".repeat(32);

const SIGNING_VECTORS = [
  {
    name: "bid",
    salt: 72_623_859_790_382_856n,
    side: OrderSide.BID,
    makerAmount: 60_000_000n,
    takerAmount: 100_000_000n,
    expiration: 0n,
    preimageHex:
      "0807060504030201" + PREIMAGE_ACCOUNTS + "00008793030000000000e1f505000000000000000000000000",
    orderIdHex: "497bbc21bdf8d3fb1187f0eda8f7a0f124dc97511e1f5068bbb7180b6b4a9bdc",
    signatureHex:
      "28ee8645e9a4f1cd3b28f910531d2d7b29b36038f873f5e1517058c02cf4cc4b" +
      "ec8400596d0e87766c3a527e4f2d04b4d0b592be656b91160e48b3b5c6a7e800",
    compactHex: "080706050403020100008793030000000000e1f505000000000000000000000000",
  },
  {
    name: "ask",
    salt: 9_833_440_827_789_222_417n,
    side: OrderSide.ASK,
    makerAmount: 100_000_000n,
    takerAmount: 60_000_000n,
    expiration: 1_700_000_000n,
    preimageHex:
      "1122334455667788" + PREIMAGE_ACCOUNTS + "0100e1f50500000000008793030000000000f1536500000000",
    orderIdHex: "ee414532884ed99be69c82c9a99d4916ccf13bb17c955d2e979c43df6697fdfc",
    signatureHex:
      "f7df8ad52408140f9418b0a9c3a5925ffe41d58f68899001769748eaa84a4f55" +
      "f5afc47c8a2d241f38b63141012f8970fecc159dfad855c624c4def66cd2c80a",
    compactHex: "11223344556677880100e1f50500000000008793030000000000f1536500000000",
  },
  {
    name: "zero_salt",
    salt: 0n,
    side: OrderSide.BID,
    makerAmount: 1n,
    takerAmount: 1n,
    expiration: 0n,
    preimageHex:
      "0000000000000000" + PREIMAGE_ACCOUNTS + "00010000000000000001000000000000000000000000000000",
    orderIdHex: "bddbbc63b3eca9b0c8ab96920b44d44239042684fa523f4d9e3e35d23228a77c",
    signatureHex:
      "2a0e4bea081930c109b666e29b979bdcfcaddb1b169a2bed1f1df60ac9522fcd" +
      "fee68652f00541151af5cc2ca9c288a79e6dab71904ee3f35efd4fb4caf43506",
    compactHex: "000000000000000000010000000000000001000000000000000000000000000000",
  },
  {
    name: "max_salt",
    salt: U64_MAX,
    side: OrderSide.ASK,
    makerAmount: U64_MAX,
    takerAmount: U64_MAX,
    expiration: I64_MAX,
    preimageHex:
      "ffffffffffffffff" + PREIMAGE_ACCOUNTS + "01ffffffffffffffffffffffffffffffffffffffffffffff7f",
    orderIdHex: "f51ef152121aeb7380e5fd0e0f5574382fc9e1bda5cecff7b98d8ff938ad0dc2",
    signatureHex:
      "a03310e35a4dbae73b111de3ffb233ffdf6b358fb2023ee5cd88989a8f8e92c7" +
      "6eee9d37042eb3028e31cd6f17878c927b8a3da5598f2736e8ddd69a84c77a01",
    compactHex: "ffffffffffffffff01ffffffffffffffffffffffffffffffffffffffffffffff7f",
  },
] as const;

function unsignedOrder(overrides: Partial<SignedOrder> = {}): Omit<SignedOrder, "signature"> {
  return {
    salt: 1n,
    maker: new PublicKey(Buffer.alloc(32, 7)),
    market: new PublicKey(Buffer.alloc(32, 8)),
    baseMint: new PublicKey(Buffer.alloc(32, 9)),
    quoteMint: new PublicKey(Buffer.alloc(32, 10)),
    side: OrderSide.BID,
    amountIn: 100n,
    amountOut: 50n,
    expiration: 0n,
    ...overrides,
  };
}

function testOrderbook(orderbookId = "unit"): OrderBookPair {
  return {
    orderbookId,
    marketPubkey: new PublicKey(Buffer.alloc(32, 1)).toBase58(),
    base: { pubkey: new PublicKey(Buffer.alloc(32, 2)).toBase58() },
    quote: { pubkey: new PublicKey(Buffer.alloc(32, 3)).toBase58() },
  } as unknown as OrderBookPair;
}

describe("nonce-free order identity", () => {
  it("matches the backend signing vectors", () => {
    const keypair = Keypair.fromSeed(Buffer.from(SIGNING_SEED_HEX, "hex"));
    assert.equal(keypair.publicKey.toBuffer().toString("hex"), SIGNING_MAKER_HEX);

    for (const vector of SIGNING_VECTORS) {
      const unsigned = {
        salt: vector.salt,
        maker: keypair.publicKey,
        market: new PublicKey(Buffer.alloc(32, 0x11)),
        baseMint: new PublicKey(Buffer.alloc(32, 0x22)),
        quoteMint: new PublicKey(Buffer.alloc(32, 0x33)),
        side: vector.side,
        amountIn: vector.makerAmount,
        amountOut: vector.takerAmount,
        expiration: vector.expiration,
      };
      const name = vector.name;

      assert.equal(orderPreimage(unsigned).toString("hex"), vector.preimageHex, `${name}: preimage`);
      // Makers sign the order id's 64 lowercase hex ASCII bytes.
      assert.equal(hashOrderHex(unsigned), vector.orderIdHex, `${name}: order id`);

      const signed: SignedOrder = { ...unsigned, signature: Buffer.from(vector.signatureHex, "hex") };
      assert.equal(
        serializeOrder(signedOrderToOrder(signed)).toString("hex"),
        vector.compactHex,
        `${name}: compact order`,
      );
      assert.ok(verifyOrderSignature(signed), `${name}: signature verifies`);

      // The 225-byte signed order is the preimage followed by the signature.
      const signedBytes = serializeSignedOrder(signed);
      assert.equal(signedBytes.toString("hex"), vector.preimageHex + vector.signatureHex);
      assert.deepEqual(serializeSignedOrder(deserializeSignedOrder(signedBytes)), signedBytes);
      const expanded = orderToSigned(
        deserializeOrder(Buffer.from(vector.compactHex, "hex")),
        signed.maker,
        signed.market,
        signed.baseMint,
        signed.quoteMint,
        signed.signature,
      );
      assert.deepEqual(serializeSignedOrder(expanded), signedBytes, `${name}: compact decode`);

      // Ed25519 is deterministic, so re-signing reproduces the vector.
      const resigned = signOrderFull(unsigned, keypair, unitRules());
      assert.equal(resigned.signature.toString("hex"), vector.signatureHex, `${name}: signature`);
    }
  });

  it("decodes serialized orders only at their exact lengths", () => {
    assert.equal(ORDER_SIZE.PREIMAGE, 161);
    assert.equal(ORDER_SIZE.SIGNED_ORDER, 225);
    assert.equal(ORDER_SIZE.ORDER, 33);
    for (const length of [224, 226]) {
      assert.throws(
        () => deserializeSignedOrder(Buffer.alloc(length)),
        (error) => error instanceof ProgramSdkError && error.variant === "InvalidDataLength",
      );
    }
    for (const length of [32, 34]) {
      assert.throws(
        () => deserializeOrder(Buffer.alloc(length)),
        (error) => error instanceof ProgramSdkError && error.variant === "InvalidDataLength",
      );
    }
    const badSide = Buffer.alloc(ORDER_SIZE.ORDER);
    badSide[8] = 2;
    assert.throws(
      () => deserializeOrder(badSide),
      (error) => error instanceof ProgramSdkError && error.variant === "InvalidSide",
    );
  });

  it("encodes and decodes only bid or ask sides", () => {
    const badSide = serializeSignedOrder({ ...unsignedOrder(), signature: Buffer.alloc(64, 1) });
    badSide[136] = 2;
    assert.throws(
      () => deserializeSignedOrder(badSide),
      (error) => error instanceof ProgramSdkError && error.variant === "InvalidSide",
    );
    const wrapping = unsignedOrder({ side: 256 as OrderSide });
    assert.throws(
      () => orderPreimage(wrapping),
      (error) => error instanceof ProgramSdkError && error.variant === "InvalidSide",
    );
    assert.throws(
      () => serializeOrder(signedOrderToOrder({ ...wrapping, signature: Buffer.alloc(64) })),
      (error) => error instanceof ProgramSdkError && error.variant === "InvalidSide",
    );
  });

  it("preflights signed amounts against rules before signing", () => {
    const keypair = Keypair.generate();
    const order = unsignedOrder({ maker: keypair.publicKey, salt: U64_MAX, amountIn: 1n, amountOut: 3_000n });
    assert.throws(
      () => signOrderFull(order, keypair, unitRules()),
      (error) => error instanceof ScalingError && error.code === "PRICE_NOT_EXACTLY_REPRESENTABLE",
    );
    assert.throws(
      () => signOrderFull({ ...order, amountIn: 0n }, keypair, unitRules()),
      (error) =>
        error instanceof ScalingError &&
        error.code === "ORDER_FIELD_OUT_OF_RANGE" &&
        /amount_in/.test(error.message),
    );
  });

  it("rejects values the encoding would otherwise wrap", () => {
    assert.throws(() => orderPreimage(unsignedOrder({ salt: U64_MAX + 1n })), /u64 value out of range/);
    assert.throws(() => orderPreimage(unsignedOrder({ salt: -1n })), /u64 value out of range/);
    assert.throws(() => orderPreimage(unsignedOrder({ expiration: I64_MAX + 1n })), /i64 value out of range/);
  });

  it("draws salts from the full u64 range", () => {
    let aboveI64 = false;
    for (let index = 0; index < 256; index += 1) {
      const salt = generateSalt();
      assert.ok(salt >= 0n && salt <= U64_MAX);
      aboveI64 ||= salt > I64_MAX;
    }
    assert.ok(aboveI64, "256 draws never exceeded i64::MAX");
  });

  it("rounds the minimum taker fill up", () => {
    const maker = { ...unsignedOrder({ side: OrderSide.ASK, amountIn: 100n, amountOut: 200n }), signature: Buffer.alloc(64) };
    assert.equal(calculateTakerFill(maker, 50n), 100n);
    // ceil(1 * 200 / 100) = 2 and ceil(3 * 200 / 101) = ceil(5.94) = 6.
    assert.equal(calculateTakerFill(maker, 1n), 2n);
    assert.equal(calculateTakerFill({ ...maker, amountIn: 101n }, 3n), 6n);
    assert.throws(
      () => calculateTakerFill({ ...maker, amountIn: 1n, amountOut: U64_MAX }, 2n),
      (error) => error instanceof ProgramSdkError && error.variant === "Overflow",
    );
    for (const fill of [-5n, U64_MAX + 1n]) {
      assert.throws(
        () => calculateTakerFill(maker, fill),
        (error) => error instanceof ProgramSdkError && error.variant === "Overflow",
      );
    }
    // A zero maker amount_in is Overflow, as the Rust SDK reports it.
    assert.throws(
      () => calculateTakerFill({ ...maker, amountIn: 0n }, 1n),
      (error) => error instanceof ProgramSdkError && error.variant === "Overflow",
    );
  });

  it("expires an order only after its expiration second", () => {
    const order = signedOrderToOrder({ ...unsignedOrder(), signature: Buffer.alloc(64) });
    assert.equal(isOrderExpired(order, I64_MAX), false);
    const expiring = { ...order, expiration: 1_700_000_000n };
    assert.equal(isOrderExpired(expiring, 1_699_999_999n), false);
    assert.equal(isOrderExpired(expiring, 1_700_000_000n), false);
    assert.equal(isOrderExpired(expiring, 1_700_000_001n), true);
  });
});

describe("signed amount ranges", () => {
  it("accepts any nonzero u64 amount", () => {
    assert.doesNotThrow(() => validateSignedFields(U64_MAX, U64_MAX));
    assert.doesNotThrow(() => validateSignedFields(1n, 1n));
    assert.throws(() => validateSignedFields(0n, 1n), /ORDER_FIELD_OUT_OF_RANGE: amount_in/);
    assert.throws(() => validateSignedFields(1n, 0n), /ORDER_FIELD_OUT_OF_RANGE: amount_out/);
    assert.throws(() => validateSignedFields(U64_MAX + 1n, 1n), /ORDER_FIELD_OUT_OF_RANGE: amount_in/);
  });

  it("admits amounts above i64::MAX", () => {
    const raw = validateRawAmounts(U64_MAX, U64_MAX, OrderSide.ASK, unitRules());
    assert.equal(raw.baseAtoms, U64_MAX);
    assert.equal(raw.priceRaw, 1_000_000n);

    const aboveI64 = (I64_MAX + 1n).toString();
    const scaled = scalePriceSize("1", aboveI64, OrderSide.BID, unitRules());
    assert.equal(scaled.amountIn, 1n << 63n);
    assert.equal(scaled.amountOut, 1n << 63n);
    assert.throws(
      () => scalePriceSize("2", aboveI64, OrderSide.BID, unitRules()),
      /ORDER_FIELD_OUT_OF_RANGE: quote amount/,
    );
  });
});

describe("submit request", () => {
  function sampleRequest(): SubmitOrderRequest {
    return {
      maker: "maker",
      salt: U64_MAX,
      market_pubkey: "market",
      base_token: "base",
      quote_token: "quote",
      side: 1,
      amount_in: 100n,
      amount_out: 50n,
      expiration: 1_700_000_000n,
      signature: "sig",
      orderbook_id: "ob",
    };
  }

  it("serializes only the backend's fields", () => {
    const json = stringifyJsonExact(sampleRequest());
    assert.equal(
      json,
      '{"maker":"maker","salt":18446744073709551615,"market_pubkey":"market",' +
        '"base_token":"base","quote_token":"quote","side":1,"amount_in":100,' +
        '"amount_out":50,"expiration":1700000000,"signature":"sig","orderbook_id":"ob"}',
    );
  });

  it("serializes time-in-force and the conditional deposit source", () => {
    const json = JSON.parse(
      stringifyJsonExact({ ...sampleRequest(), tif: TimeInForce.Ioc, deposit_source: DepositSource.Market }),
    ) as Record<string, unknown>;
    assert.equal(json.tif, "IOC");
    assert.equal(json.deposit_source, "conditional");
    assert.equal(Object.keys(json).length, 13);
  });

  it("encodes a signed order without a nonce and rejects an unsigned one", () => {
    const keypair = Keypair.generate();
    const signed = signOrderFull(unsignedOrder({ maker: keypair.publicKey }), keypair, unitRules());
    const request = toSubmitRequest(signed, "unit", { timeInForce: TimeInForce.Fok });
    assert.equal(request.salt, signed.salt);
    assert.equal(request.signature, signed.signature.toString("hex"));
    assert.ok(!("nonce" in request));
    assert.throws(
      () => toSubmitRequest({ ...signed, signature: Buffer.alloc(64) }, "unit"),
      (error) => error instanceof ProgramSdkError && error.variant === "UnsignedOrder",
    );
  });
});

describe("order envelope", () => {
  function envelope(maker: PublicKey): LimitOrderEnvelope {
    return LimitOrderEnvelope.new()
      .maker(maker)
      .market(new PublicKey(Buffer.alloc(32, 4)))
      .baseMint(new PublicKey(Buffer.alloc(32, 5)))
      .quoteMint(new PublicKey(Buffer.alloc(32, 6)))
      .bid()
      .amountIn(1_000_000n)
      .amountOut(500_000n);
  }

  it("draws an unset salt once and reuses it", () => {
    const order = envelope(Keypair.generate().publicKey);
    assert.equal(order.getSalt(), undefined);
    const first = order.payload();
    assert.equal(order.getSalt(), first.salt);
    const second = order.payload();
    assert.equal(second.salt, first.salt);
    assert.equal(hashOrderHex(second), hashOrderHex(first));
    assert.equal(order.salt(7n).payload().salt, 7n);
  });

  it("submits the hash a wallet adapter signed", () => {
    const keypair = Keypair.generate();
    const order = envelope(keypair.publicKey);
    // An external wallet signs the payload's hash before finalize().
    const payload = order.payload();
    const signature = Buffer.from(
      sign.detached(Buffer.from(hashOrderHex(payload), "ascii"), keypair.secretKey),
    );
    const request = order.finalize(bs58.encode(signature), testOrderbook(), unitRules());
    assert.equal(request.salt, payload.salt);
    assert.equal(request.signature, signature.toString("hex"));
    assert.ok(verifyOrderSignature({ ...payload, signature }));
  });

  it("rejects a malformed external signature", () => {
    const keypair = Keypair.generate();
    for (const signature of ["1111", "0x" + "ab".repeat(64)]) {
      assert.throws(
        () => envelope(keypair.publicKey).finalize(signature, testOrderbook(), unitRules()),
        (error) => error instanceof ProgramSdkError && error.variant === "InvalidSignature",
      );
    }
  });

  it("rejects an unencodable salt or expiration before submission", () => {
    const signature = bs58.encode(Buffer.alloc(64, 1));
    const maker = Keypair.generate().publicKey;
    assert.throws(
      () => envelope(maker).salt(-1n).finalize(signature, testOrderbook(), unitRules()),
      /u64 value out of range/,
    );
    assert.throws(
      () => envelope(maker).expiration(I64_MAX + 1n).finalize(signature, testOrderbook(), unitRules()),
      /i64 value out of range/,
    );
  });

  it("reports zero amounts as missing, as the Rust SDK does", () => {
    const maker = Keypair.generate().publicKey;
    assert.throws(
      () => envelope(maker).amountIn(0n).amountOut(5n).payload(),
      (error) =>
        error instanceof ProgramSdkError &&
        error.variant === "MissingField" &&
        /amount_in must be greater than 0/.test(error.message),
    );
    assert.throws(
      () => envelope(maker).amountIn(5n).amountOut(0n).payload(),
      (error) => error instanceof ProgramSdkError && /amount_out must be greater than 0/.test(error.message),
    );
  });

  it("derives price and size again on every build", () => {
    const keypair = Keypair.generate();
    const order = LimitOrderEnvelope.new().maker(keypair.publicKey).bid().price("0.5").size("10");
    const first = order.sign(keypair, testOrderbook(), unitRules());
    assert.deepEqual([first.amount_in, first.amount_out], [5n, 10n]);
    // Signing again unchanged resubmits the identical order.
    assert.deepEqual(order.sign(keypair, testOrderbook(), unitRules()), first);
    const repriced = order.price("0.6").sign(keypair, testOrderbook(), unitRules());
    assert.deepEqual([repriced.amount_in, repriced.amount_out], [6n, 10n]);
    const ask = order.ask().sign(keypair, testOrderbook(), unitRules());
    assert.deepEqual([ask.amount_in, ask.amount_out], [10n, 6n]);
    // Raw amounts replace derived ones and are kept as supplied.
    const raw = order.amountIn(7n).amountOut(14n).sign(keypair, testOrderbook(), unitRules());
    assert.deepEqual([raw.amount_in, raw.amount_out], [7n, 14n]);
    assert.equal(raw.salt, first.salt);
  });

  it("clears derived amounts on a price, size, or side change", () => {
    const keypair = Keypair.generate();
    const isMissingAmountIn = (error: unknown) =>
      error instanceof ProgramSdkError &&
      error.variant === "MissingField" &&
      /amount_in/.test(error.message);
    for (const change of [
      (order: LimitOrderEnvelope) => order.price("0.6"),
      (order: LimitOrderEnvelope) => order.size("20"),
      (order: LimitOrderEnvelope) => order.ask(),
      (order: LimitOrderEnvelope) => order.bid(),
      (order: LimitOrderEnvelope) => order.side(OrderSide.ASK),
    ]) {
      const order = LimitOrderEnvelope.new().maker(keypair.publicKey).bid().price("0.5").size("10");
      order.sign(keypair, testOrderbook(), unitRules());
      assert.deepEqual([order.getAmountIn(), order.getAmountOut()], [5n, 10n]);
      change(order);
      assert.equal(order.getAmountIn(), undefined);
      assert.throws(() => order.payload(), isMissingAmountIn);
    }
    // Raw amounts are kept across price, size, and side changes.
    const raw = LimitOrderEnvelope.new().maker(keypair.publicKey).bid().amountIn(7n).amountOut(14n);
    raw.price("0.6").size("20").ask();
    assert.deepEqual([raw.getAmountIn(), raw.getAmountOut()], [7n, 14n]);
  });

  it("reports missing amounts before identities, as Rust's payload does", () => {
    const isMissing = (field: string) => (error: unknown) =>
      error instanceof ProgramSdkError && error.variant === "MissingField" && error.message.endsWith(field);
    assert.throws(() => LimitOrderEnvelope.new().payload(), isMissing("amount_in"));
    assert.throws(() => LimitOrderEnvelope.new().amountIn(5n).amountOut(5n).payload(), isMissing("maker"));
  });

  it("reports a missing side with the Rust SDK's message", () => {
    const keypair = Keypair.generate();
    assert.throws(
      () => LimitOrderEnvelope.new().maker(keypair.publicKey).price("0.5").size("10").sign(keypair, testOrderbook(), unitRules()),
      (error) =>
        error instanceof ProgramSdkError &&
        error.variant === "MissingField" &&
        error.message.includes("side (call .bid() or .ask())"),
    );
  });

  it("serializes the conditional deposit source", () => {
    const keypair = Keypair.generate();
    const request = envelope(keypair.publicKey)
      .salt(0n)
      .depositSource(DepositSource.Market)
      .timeInForce(TimeInForce.Fok)
      .sign(keypair, testOrderbook(), unitRules());
    const json = JSON.parse(stringifyJsonExact(request)) as Record<string, unknown>;
    assert.equal(json.deposit_source, "conditional");
    assert.equal(json.tif, "FOK");
    assert.equal(json.nonce, undefined);
    assert.ok(isSigned({ ...envelope(keypair.publicKey).salt(0n).payload(), signature: Buffer.from(request.signature, "hex") }));
  });
});

describe("OrderBuilder", () => {
  function builder(maker: PublicKey): OrderBuilder {
    return new OrderBuilder()
      .maker(maker)
      .market(new PublicKey(Buffer.alloc(32, 4)))
      .baseMint(new PublicKey(Buffer.alloc(32, 5)))
      .quoteMint(new PublicKey(Buffer.alloc(32, 6)));
  }

  it("prices with the side set at build time", () => {
    const maker = Keypair.generate().publicKey;
    const priceFirst = builder(maker).salt(1n).price("0.5", "10", unitRules()).ask().build();
    const sideFirst = builder(maker).salt(1n).ask().price("0.5", "10", unitRules()).build();
    assert.deepEqual([priceFirst.amountIn, priceFirst.amountOut], [10n, 5n]);
    assert.equal(hashOrderHex(priceFirst), hashOrderHex(sideFirst));
  });

  it("fixes its salt on the first build", () => {
    const keypair = Keypair.generate();
    const order = builder(keypair.publicKey).bid().price("0.5", "10", unitRules());
    const unsigned = order.build();
    const signed = order.buildAndSign(keypair);
    assert.equal(signed.salt, unsigned.salt);
    assert.equal(hashOrderHex(signed), hashOrderHex(unsigned));
    assert.ok(verifyOrderSignature(signed));
  });

  it("lets raw amounts replace price and size", () => {
    const order = builder(Keypair.generate().publicKey)
      .price("0.5", "10", unitRules())
      .amountIn(6n)
      .amountOut(10n)
      .build();
    assert.deepEqual([order.amountIn, order.amountOut], [6n, 10n]);
  });
});

describe("cancel-all signing", () => {
  it("signs the UTF-8 message the backend verifies", () => {
    const keypair = Keypair.generate();
    const wallet = keypair.publicKey.toBase58();
    for (const salt of ["0b9f6c1e-1d3a-4c55-9a7e-2f4d8e6b1c3a", "salt-\u00e9", "\u65e5\u672c"]) {
      const signature = signCancelAll(wallet, "ob", 1_700_000_000, salt, keypair);
      const message = new TextEncoder().encode(cancelAllMessage(wallet, "ob", 1_700_000_000, salt));
      assert.ok(sign.detached.verify(message, Buffer.from(signature, "hex"), keypair.publicKey.toBytes()));
    }
  });
});

describe("v1 transaction known answer", () => {
  it("matches the program client's CloseOrderStatus bytes", () => {
    const programId = new PublicKey("Hobw7Fi6SN6YaCA4Bwp5RcbCR3YBXQ9PpGSSw5muEzai");
    const operator = Keypair.fromSeed(Buffer.alloc(32, 1));
    const instruction = buildCloseOrderStatusIx(
      { operator: operator.publicKey, orderHash: Buffer.alloc(32, 9) },
      programId,
    );
    const context = {
      blockhash: new PublicKey(Buffer.alloc(32, 7)).toBase58(),
      // Expiry is not encoded in the message; any nonzero height works.
      lastValidBlockHeight: 1,
      resources: {
        computeUnitLimit: 300_000,
        loadedAccountsDataSizeLimit: 256 * 1024,
        priorityFeeLamports: 5_000n,
        heapSize: 64 * 1024,
      },
    };

    // Generated by lightcone-client at lightcone-pinnochio f1092ae
    // (`transaction::compile`, `sign` and `wire_bytes`) for the same
    // CloseOrderStatus instruction, payer, blockhash and resources.
    const message = [
      // v1 prefix; header: 1 signer, 0 read-only signed, 3 read-only unsigned.
      "81",
      "010003",
      // Config mask: priority fee, compute, loaded data and heap follow.
      "1f000000",
      // Blockhash, then 1 instruction and 5 addresses.
      "0707070707070707070707070707070707070707070707070707070707070707",
      "01",
      "05",
      // Addresses: operator, order status, exchange, event authority, program.
      "8a88e3dd7409f195fd52db2d3cba5d72ca6709bf1d94121bf3748801b40f6f5c",
      "f4387fb4008f7b7e99b20780f8352e1ab6a078f2623ea3c09b26e16a118f93dc",
      "23d1b92b4d95970923f386214511e526cdae2f4d5f810e88aba3891216092bfe",
      "6f6489f61b2a8012f5d8c93bc98a295e39bce21fceb2586a1e8bab3f4c7eaaad",
      "f9ab34f009082920ea9265cf08b5627de9959ad8ea5c3a277159532374ce4e6f",
      // Config values: 5,000 lamports priority fee, 300,000 compute units,
      // 262,144 loaded account bytes and a 65,536-byte heap.
      "8813000000000000",
      "e0930400",
      "00000400",
      "00000100",
      // Instruction header: program index 4, 5 accounts, 33 data bytes.
      "04",
      "05",
      "2100",
      // Account indices, then CloseOrderStatus data (discriminator, order hash).
      "0002010304",
      "180909090909090909090909090909090909090909090909090909090909090909",
    ].join("");
    const signature =
      "1180f79af6fdc0997fa3d9049c2969aeb14fbc1d47d49afbb026fc74e22d918d" +
      "631a16f2a43260718588959f02a08c82d5af7460afc86c99c285b0cd6d67df06";

    const unsigned = V1Transaction.compile([instruction], operator.publicKey, context);
    assert.equal(Buffer.from(unsigned.messageBytes()).toString("hex"), message);
    const signed = unsigned.sign([operator]);
    // v1 wire bytes are the message followed by its signatures.
    assert.equal(Buffer.from(signed.toWireBytes()).toString("hex"), message + signature);
  });
});
