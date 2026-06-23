import { describe, it } from "node:test";
import assert from "node:assert/strict";
import {
  UserMarketBalanceIndex,
  conditionalDeltaIsZero,
  conditionalDeltaToOutcomeBalance,
  conditionalDeltaToTokenBalance,
  conditionalDeltaTotal,
} from "../src/domain/position";
import type { ConditionalBalanceDelta } from "../src/domain/position";
import { userOutcomeBalanceIsZero } from "../src/domain/order/wire";
import type { UserMarketBalance, UserOutcomeBalance } from "../src/domain/order/wire";

const delta = (overrides: Partial<ConditionalBalanceDelta> = {}): ConditionalBalanceDelta => ({
  marketPubkey: "market-1",
  orderbookId: "trump-usdc",
  outcomeIndex: 0,
  conditionalToken: "trump-usdc-mint",
  idle: "40.000000",
  onBook: "85.000000",
  ...overrides,
});

const outcome = (
  conditionalToken: string,
  balanceIdle: string,
  balanceOnBook: string,
  outcomeIndex = 0
): UserOutcomeBalance => ({
  outcome_index: outcomeIndex,
  conditional_token: conditionalToken,
  balance: "0",
  balance_idle: balanceIdle,
  balance_on_book: balanceOnBook,
});

const marketBalance = (marketPubkey: string): UserMarketBalance => ({
  market_pubkey: marketPubkey,
  deposit_assets: [
    {
      deposit_asset: "usdc-mint",
      outcomes: [
        outcome("trump-usdc-mint", "40.000000", "85.000000", 0),
        outcome("kamala-usdc-mint", "0", "0", 1), // zero → dropped
      ],
    },
    {
      deposit_asset: "empty-asset",
      outcomes: [outcome("zzz-mint", "0", "0", 0)], // all-zero → asset dropped
    },
  ],
});

describe("userOutcomeBalanceIsZero", () => {
  it("is true only when idle and on-book are both empty", () => {
    assert.equal(userOutcomeBalanceIsZero(outcome("m", "0", "0")), true);
    assert.equal(userOutcomeBalanceIsZero(outcome("m", "0.000001", "0")), false);
    assert.equal(userOutcomeBalanceIsZero(outcome("m", "0", "1")), false);
  });
});

describe("conditional balance delta", () => {
  it("computes full-precision total without truncation", () => {
    assert.equal(conditionalDeltaTotal(delta({ idle: "40.000001", onBook: "85" })), "125.000001");
  });

  it("reports zero only when idle and on-book are both empty", () => {
    assert.equal(conditionalDeltaIsZero(delta({ idle: "0", onBook: "0" })), true);
    assert.equal(conditionalDeltaIsZero(delta({ idle: "0", onBook: "0.000001" })), false);
    assert.equal(conditionalDeltaIsZero(delta()), false);
  });

  it("converts to a conditional TokenBalance", () => {
    const tokenBalance = conditionalDeltaToTokenBalance(delta());
    assert.equal(tokenBalance.mint, "trump-usdc-mint");
    assert.equal(tokenBalance.idle, "40.000000");
    assert.equal(tokenBalance.onBook, "85.000000");
    assert.deepEqual(tokenBalance.tokenType, {
      kind: "ConditionalToken",
      orderbookId: "trump-usdc",
      marketPubkey: "market-1",
      outcomeIndex: 0,
    });
  });

  it("defaults a missing orderbook id to empty string in the TokenBalance", () => {
    const tokenBalance = conditionalDeltaToTokenBalance(delta({ orderbookId: undefined }));
    if (tokenBalance.tokenType.kind !== "ConditionalToken") {
      throw new Error("expected ConditionalToken token type");
    }
    assert.equal(tokenBalance.tokenType.orderbookId, "");
  });

  it("converts to a UserOutcomeBalance with the summed balance", () => {
    const converted = conditionalDeltaToOutcomeBalance(delta({ idle: "40.000001", onBook: "85" }));
    assert.equal(converted.conditional_token, "trump-usdc-mint");
    assert.equal(converted.balance, "125.000001");
    assert.equal(converted.balance_idle, "40.000001");
    assert.equal(converted.balance_on_book, "85");
  });
});

describe("UserMarketBalanceIndex", () => {
  it("builds a nested index, skipping zero outcomes and empty deposit assets", () => {
    const index = UserMarketBalanceIndex.fromUserMarketBalances([marketBalance("market-1")]);
    const market = index.get("market-1");
    assert.equal(market?.size, 1); // empty-asset dropped
    assert.equal(market?.get("usdc-mint")?.size, 1); // zero outcome dropped
    assert.equal(market?.get("usdc-mint")?.has("trump-usdc-mint"), true);
    assert.equal(market?.get("usdc-mint")?.has("kamala-usdc-mint"), false);
  });

  it("returns undefined for a wholly-empty market", () => {
    const empty: UserMarketBalance = {
      market_pubkey: "market-empty",
      deposit_assets: [{ deposit_asset: "usdc-mint", outcomes: [outcome("zzz", "0", "0")] }],
    };
    assert.equal(UserMarketBalanceIndex.fromUserMarketBalance(empty), undefined);
  });

  it("returns market pubkeys sorted regardless of insertion order", () => {
    const index = UserMarketBalanceIndex.fromUserMarketBalances([
      marketBalance("market-c"),
      marketBalance("market-a"),
      marketBalance("market-b"),
    ]);
    assert.deepEqual(index.marketPubkeys(), ["market-a", "market-b", "market-c"]);
  });

  it("treats a fully-empty input as an empty index", () => {
    const index = UserMarketBalanceIndex.fromUserMarketBalances([]);
    assert.equal(index.isEmpty(), true);
    assert.deepEqual(index.marketPubkeys(), []);
  });
});
