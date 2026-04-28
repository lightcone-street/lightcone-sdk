import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { deriveDepositAssetPairs } from "../src/domain/market/convert";
import {
  sortByDisplayPriority,
  tokenDisplayPriority,
  type ConditionalToken,
  type DepositAsset,
} from "../src/domain/market";
import type { OrderBookPair } from "../src/domain/orderbook";
import { asPubkeyStr, type OrderBookId } from "../src/shared";

function depositAsset(mint: string): DepositAsset {
  const pubkey = asPubkeyStr(mint);
  return {
    id: 1,
    marketPda: asPubkeyStr("market"),
    depositAsset: pubkey,
    numOutcomes: 2,
    pubkey,
    name: mint,
    symbol: mint,
    description: undefined,
    decimals: 6,
    iconUrl: "",
  };
}

function conditionalToken(
  mint: string,
  outcomeIndex: number,
  depositMint: string,
): ConditionalToken {
  return {
    id: outcomeIndex + 1,
    outcomeIndex,
    outcome: "Yes",
    depositAsset: asPubkeyStr(depositMint),
    depositSymbol: depositMint,
    pubkey: asPubkeyStr(mint),
    name: "Outcome",
    symbol: "YES",
    description: undefined,
    decimals: 6,
    iconUrl: "",
  };
}

function orderbookPair(
  baseMint: string,
  quoteMint: string,
  outcomeIndex: number,
): OrderBookPair {
  return {
    id: outcomeIndex,
    marketPubkey: asPubkeyStr("market"),
    orderbookId: `ob-${outcomeIndex}` as OrderBookId,
    base: conditionalToken(`cond-base-${outcomeIndex}`, outcomeIndex, baseMint),
    quote: conditionalToken(`cond-quote-${outcomeIndex}`, outcomeIndex, quoteMint),
    outcomeIndex,
    tickSize: 1,
    totalBids: 0,
    totalAsks: 0,
    lastTradePrice: undefined,
    lastTradeTime: undefined,
    active: true,
  };
}

describe("deriveDepositAssetPairs", () => {
  it("deduplicates across outcomes", () => {
    const base = depositAsset("USDC");
    const quote = depositAsset("USDT");
    const pairs = deriveDepositAssetPairs(
      [base, quote],
      [orderbookPair("USDC", "USDT", 0), orderbookPair("USDC", "USDT", 1)],
    );

    assert.equal(pairs.length, 1);
    assert.equal(pairs[0]!.id, "USDC-USDT");
    assert.deepEqual(pairs[0]!.base, base);
    assert.deepEqual(pairs[0]!.quote, quote);
  });

  it("skips orderbook pairs without a matching deposit asset", () => {
    const pairs = deriveDepositAssetPairs(
      [depositAsset("USDC")],
      [orderbookPair("USDC", "MISSING", 0)],
    );
    assert.deepEqual(pairs, []);
  });

  it("returns all distinct pairs", () => {
    const pairs = deriveDepositAssetPairs(
      [depositAsset("USDC"), depositAsset("USDT"), depositAsset("DAI")],
      [orderbookPair("USDC", "USDT", 0), orderbookPair("USDC", "DAI", 0)],
    ).sort((a, b) => a.id.localeCompare(b.id));

    assert.equal(pairs.length, 2);
    assert.equal(pairs[0]!.id, "USDC-DAI");
    assert.equal(pairs[1]!.id, "USDC-USDT");
  });
});

describe("tokenDisplayPriority / sortByDisplayPriority", () => {
  const cases: ReadonlyArray<[string, number]> = [
    ["BTC", 0],
    ["WBTC", 0],
    ["ETH", 1],
    ["WETH", 1],
    ["SOL", 2],
    ["USDC", 255],
    ["ZZZ", 255],
  ];

  for (const [symbol, priority] of cases) {
    it(`tokenDisplayPriority returns ${priority} for ${symbol}`, () => {
      assert.equal(tokenDisplayPriority({ symbol }), priority);
    });
  }

  it("sorts priority groups first then alphabetical", () => {
    const symbols = ["USDC", "SOL", "WETH", "AAA", "WBTC", "ETH", "BTC", "ZZZ"];
    const sorted = sortByDisplayPriority(symbols.map((symbol) => ({ symbol })));
    assert.deepEqual(
      sorted.map((token) => token.symbol),
      ["BTC", "WBTC", "ETH", "WETH", "SOL", "AAA", "USDC", "ZZZ"],
    );
  });

  it("sorts composite pair types via their base token", () => {
    // DepositAssetPair-shaped objects (no top-level symbol, base carries it).
    const pairs = ["USDC", "SOL", "BTC", "ETH"].map((symbol) => ({
      id: `${symbol}-DAI`,
      base: depositAsset(symbol),
      quote: depositAsset("DAI"),
    }));
    const sorted = sortByDisplayPriority(pairs);
    assert.deepEqual(
      sorted.map((pair) => pair.base.symbol),
      ["BTC", "ETH", "SOL", "USDC"],
    );
  });
});
