import { describe, it } from "node:test";
import assert from "node:assert/strict";
import Decimal from "decimal.js";
import {
  conditionalDeltaIsZero,
  conditionalDeltaToTokenBalance,
  conditionalDeltaTotal,
} from "../src/domain/position";
import type { ConditionalBalanceDelta } from "../src/domain/position";

const delta = (overrides: Partial<ConditionalBalanceDelta> = {}): ConditionalBalanceDelta => ({
  marketPubkey: "market-1",
  orderbookId: "trump-usdc",
  outcomeIndex: 0,
  conditionalToken: "trump-usdc-mint",
  idle: "40.000000",
  onBook: "85.000000",
  ...overrides,
});

describe("conditional balance delta", () => {
  it("computes full-precision total without truncation", () => {
    assert.equal(conditionalDeltaTotal(delta({ idle: "40.000001", onBook: "85" })), "125.000001");
    assert.equal(conditionalDeltaTotal(delta()), "125.000000");
  });

  it("adds exactly, independent of Decimal.precision, and never uses exponents", () => {
    const previous = Decimal.precision;
    Decimal.set({ precision: 6 });
    try {
      assert.equal(
        conditionalDeltaTotal(delta({ idle: "12344.678912345", onBook: "0.000000001" })),
        "12344.678912346",
      );
    } finally {
      Decimal.set({ precision: previous });
    }
    assert.equal(conditionalDeltaTotal(delta({ idle: "0.0000001", onBook: "0" })), "0.0000001");
    assert.equal(
      conditionalDeltaTotal(delta({ idle: "79228162514264337593543950335", onBook: "1" })),
      "79228162514264337593543950336",
    );
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
});
