import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { Keypair, PublicKey } from "@solana/web3.js";
import {
  I64_MAX,
  ScalingError,
  parseJsonExact,
  scalePriceSize,
  stringifyJsonExact,
  validateRawAmounts,
  validateSignedFields,
  RejectionCode,
} from "../src/shared";
import { orderbookRulesFromWire, type DecimalsResponse } from "../src/domain/orderbook/wire";
import { ORDER_SIZE } from "../src/program/constants";
import { generateSalt, hashOrderHex, signOrderFull } from "../src/program/orders";
import { OrderSide } from "../src/program/types";
import { parseMessageIn } from "../src/ws";
import { LightconeClient } from "../src/client";
import { LimitOrderEnvelope } from "../src/program/envelope";
import { ProgramSdkError } from "../src/program/error";
import type { OrderBookPair } from "../src/domain/orderbook";

const rulesWire: DecimalsResponse = {
  orderbook_id: "11111111111111111111111111111111",
  base_decimals: 8,
  quote_decimals: 6,
  price_decimals: 4,
  trading_rules: {
    base_size_decimals: 5,
    max_price_decimals: 1,
    max_price_significant_figures: 5,
    integer_prices_always_allowed: true,
    price_quantum: "0.1000",
    price_quantum_raw: "1000",
    base_size_quantum: "0.00001000",
    base_size_quantum_raw: "1000",
  },
};
const rules = orderbookRulesFromWire(rulesWire);

const validOrders = [
  { side: "bid", price: "12.3", size: "1.23456", amountIn: 15_185_088n, amountOut: 123_456_000n },
  { side: "ask", price: "12.3", size: "1.23456", amountIn: 123_456_000n, amountOut: 15_185_088n },
  { side: "bid", price: "150250", size: "1", amountIn: 150_250_000_000n, amountOut: 100_000_000n },
] as const;

const invalidOrders = [
  { price: "12.34", size: "1.23456", code: "INVALID_PRICE_DECIMALS" },
  { price: "12.3", size: "1.234567", code: "INVALID_SIZE_DECIMALS" },
  { price: "150250.1", size: "1", code: "INVALID_PRICE_SIGNIFICANT_FIGURES" },
] as const;

const signingCase = {
  seedHex: "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f",
  maker: "FAe4sisG95oZ42w7buUn5qEE4TAnfTTFPiguZUHmhiF",
  market: "4vJ9JU1bJJE96FWSJKvHsmmFADCg4gpZQff4P3bkLKi",
  baseMint: "8qbHbw2BbbTHBW1sbeqakYXVKRQM8Ne7pLK7m6CVfeR",
  quoteMint: "CktRuQ2mttgRGkXJtyksdKHjUdc2C4TgDzyB98oEzy8",
  hashHex: "17228fe4bdf93c14714367454e948206bb4f001917d59e132e4aaad097819eac",
  signatureHex: "1e68fe672f919085ed34333c86facf9ad816ae30ab23d3d6ccef0aeb4c40b161f841c8f68355c9720252fc9ab4d859e64416d81ffa170e5a6cd17dfe41038808",
} as const;

describe("exact order precision", () => {
  it("matches the exact valid and invalid cases", () => {
    for (const value of validOrders) {
      const side = value.side === "bid" ? OrderSide.BID : OrderSide.ASK;
      const scaled = scalePriceSize(value.price, value.size, side, rules);
      assert.equal(scaled.amountIn, value.amountIn);
      assert.equal(scaled.amountOut, value.amountOut);
    }
    for (const value of invalidOrders) {
      assert.throws(
        () => scalePriceSize(value.price, value.size, OrderSide.BID, rules),
        (error) => error instanceof ScalingError && error.code === value.code
      );
    }
  });

  it("preflights exact ratios and signed field boundaries", () => {
    assert.throws(
      () => validateRawAmounts(1n, 3_000n, OrderSide.BID, rules),
      /PRICE_NOT_EXACTLY_REPRESENTABLE/
    );
    assert.doesNotThrow(() => validateSignedFields(I64_MAX, I64_MAX, I64_MAX, 2 ** 32 - 1));
    assert.throws(() => validateSignedFields(I64_MAX + 1n, 1n, 0n, 0));
    for (let index = 0; index < 10_000; index += 1) {
      const salt = generateSalt();
      assert.ok(salt >= 0n && salt <= I64_MAX);
    }
  });

  it("keeps the 169-byte signing contract and known hash/signature", () => {
    const keypair = Keypair.fromSeed(Buffer.from(signingCase.seedHex, "hex"));
    const order = signOrderFull(
      {
        nonce: 42,
        salt: 123n,
        maker: keypair.publicKey,
        market: new PublicKey(signingCase.market),
        baseMint: new PublicKey(signingCase.baseMint),
        quoteMint: new PublicKey(signingCase.quoteMint),
        side: OrderSide.BID,
        amountIn: 15_185_088n,
        amountOut: 123_456_000n,
        expiration: 0n,
      },
      keypair,
      rules
    );
    assert.equal(keypair.publicKey.toBase58(), signingCase.maker);
    assert.equal(ORDER_SIZE.SIGNED_ORDER - ORDER_SIZE.SIGNATURE, 169);
    assert.equal(hashOrderHex(order), signingCase.hashHex);
    assert.equal(order.signature.toString("hex"), signingCase.signatureHex);
    const { signature: _signature, ...unsigned } = order;
    assert.throws(
      () => signOrderFull({ ...unsigned, salt: I64_MAX + 1n }, keypair, rules),
      /ORDER_FIELD_OUT_OF_RANGE/
    );
    assert.throws(
      () => signOrderFull({ ...unsigned, amountIn: 1n, amountOut: 3_000n }, keypair, rules),
      /PRICE_NOT_EXACTLY_REPRESENTABLE/
    );
  });

  it("requires rules and validates the high-level signed-order path", () => {
    const keypair = Keypair.fromSeed(Buffer.alloc(32, 9));
    const pair = {
      orderbookId: rulesWire.orderbook_id,
      marketPubkey: new PublicKey(Buffer.alloc(32, 1)).toBase58(),
      base: { pubkey: new PublicKey(Buffer.alloc(32, 2)).toBase58() },
      quote: { pubkey: new PublicKey(Buffer.alloc(32, 3)).toBase58() },
    } as unknown as OrderBookPair;
    const request = LimitOrderEnvelope.new()
      .nonce(1)
      .salt(0n)
      .maker(keypair.publicKey)
      .bid()
      .price("12.3")
      .size("1.23456")
      .sign(keypair, pair, rules);
    assert.equal(request.amount_in, 15_185_088n);
    assert.equal(request.amount_out, 123_456_000n);

    assert.throws(
      () =>
        LimitOrderEnvelope.new()
          .nonce(1)
          .salt(0n)
          .maker(keypair.publicKey)
          .bid()
          .price("12.3")
          .size("1.23456")
          .sign(keypair, pair, { ...rules, orderbookId: "another-orderbook" }),
      /cannot be used/
    );

    assert.throws(() =>
      LimitOrderEnvelope.new()
        .nonce(1)
        .salt(0n)
        .maker(keypair.publicKey)
        .bid()
        .amountIn(1n)
        .amountOut(3_000n)
        .sign(keypair, pair, rules)
    , /PRICE_NOT_EXACTLY_REPRESENTABLE/);
  });

  it("parses and serializes unsafe integers without number loss", () => {
    const parsed = parseJsonExact<{ revision: bigint }>("{\"revision\":9223372036854775807}");
    assert.equal(parsed.revision, I64_MAX);
    assert.equal(stringifyJsonExact({ amount_in: I64_MAX }), "{\"amount_in\":9223372036854775807}");
    const ws = parseMessageIn(
      "{\"type\":\"book_update\",\"version\":0.1,\"data\":{\"orderbook_id\":\"ob\",\"seq\":9223372036854775807,\"bids\":[],\"asks\":[]}}"
    );
    assert.equal(ws.type, "book_update");
    if (ws.type === "book_update") assert.equal(ws.data.seq, I64_MAX);
    assert.throws(() =>
      orderbookRulesFromWire({
        ...rulesWire,
        trading_rules: {
          ...rulesWire.trading_rules,
          price_quantum_raw: 1000,
        },
      } as unknown as DecimalsResponse)
    );
  });

  it("deduplicates immutable rules discovery and normalizes depth metadata", async () => {
    const client = LightconeClient.builder().baseUrl("https://example.test").build();
    let decimalsCalls = 0;
    (client.http as unknown as { get: (url: string) => Promise<unknown> }).get = async (url) => {
      if (url.includes("/decimals")) {
        decimalsCalls += 1;
        return rulesWire;
      }
      return {
        orderbook_id: "ob",
        best_bid: null,
        best_ask: null,
        bids: [],
        asks: [],
        price_quantum: "0.1000",
        trading_rules: rulesWire.trading_rules,
        revision: 1842,
        captured_at_ms: 1785776400123,
        decimals: { price: 4, size: 8 },
      };
    };
    const [first, second] = await Promise.all([
      client.orderbooks().decimals("ob"),
      client.orderbooks().decimals("ob"),
    ]);
    assert.equal(decimalsCalls, 1);
    assert.equal(first, second);
    const depth = await client.orderbooks().get("ob");
    assert.equal(depth.revision, 1842n);
    assert.equal(depth.captured_at_ms, 1785776400123n);
    assert.equal(depth.trading_rules.priceQuantumRaw, 1000n);
    assert.equal(depth.bids_truncated, false);
    assert.equal(depth.asks_truncated, false);

    let submitCalls = 0;
    (client.http as unknown as { post: () => Promise<unknown> }).post = async () => {
      submitCalls += 1;
      return {};
    };
    await assert.rejects(
      () =>
        client.orders().submit({
          maker: "maker",
          nonce: 0,
          salt: 0n,
          market_pubkey: "market",
          base_token: "base",
          quote_token: "quote",
          side: OrderSide.BID,
          amount_in: 1n,
          amount_out: 3_000n,
          expiration: 0n,
          signature: "signature",
          orderbook_id: "ob",
        }),
      /PRICE_NOT_EXACTLY_REPRESENTABLE/
    );
    assert.equal(submitCalls, 0);
  });

  it("reports null projection metadata as a serialization error", async () => {
    for (const field of ["revision", "captured_at_ms"] as const) {
      const client = LightconeClient.builder().baseUrl("https://example.test").build();
      (client.http as unknown as { get: () => Promise<unknown> }).get = async () => ({
        orderbook_id: "ob",
        best_bid: null,
        best_ask: null,
        bids: [],
        asks: [],
        price_quantum: "0.1000",
        trading_rules: rulesWire.trading_rules,
        revision: 1842,
        captured_at_ms: 1785776400123,
        decimals: { price: 4, size: 8 },
        [field]: null,
      });

      await assert.rejects(
        () => client.orderbooks().get("ob"),
        (error) =>
          error instanceof ProgramSdkError && error.variant === "Serialization"
      );
    }
  });

  it("recognizes every exact-order rejection code", () => {
    for (const code of [
      "TRADING_RULES_UNAVAILABLE",
      "ORDER_FIELD_OUT_OF_RANGE",
      "PRICE_NOT_EXACTLY_REPRESENTABLE",
      "PRICE_OUT_OF_RANGE",
      "INVALID_PRICE_DECIMALS",
      "INVALID_PRICE_SIGNIFICANT_FIGURES",
      "INVALID_SIZE_DECIMALS",
      "TRIGGER_PRICE_OUT_OF_RANGE",
    ]) {
      assert.notEqual(RejectionCode.from(code).label(), code);
      assert.equal(RejectionCode.from(code).wireName(), code);
    }
  });
});
