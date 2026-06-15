import { describe, it } from "node:test";
import assert from "node:assert/strict";
import Decimal from "decimal.js";
import {
  applyImpactProtection,
  convertDenomination,
  Denominator,
  receiveDenominator,
  Side,
  spendDenominator,
  toOrderSide,
} from "../src/shared/types";
import { OrderSide } from "../src/program/types";

describe("trading math", () => {
  it("maps sides to spend/receive denominations", () => {
    assert.equal(spendDenominator(Side.Bid), Denominator.Quote);
    assert.equal(receiveDenominator(Side.Bid), Denominator.Base);
    assert.equal(spendDenominator(Side.Ask), Denominator.Base);
    assert.equal(receiveDenominator(Side.Ask), Denominator.Quote);
  });

  it("treats same-denomination conversion as identity even without a price", () => {
    const amount = new Decimal("4.25");
    assert.equal(convertDenomination(Denominator.Base, Denominator.Base, amount, new Decimal(0)), amount);
    assert.equal(convertDenomination(Denominator.Quote, Denominator.Quote, amount, new Decimal(0)), amount);
  });

  it("crosses denominations at the price", () => {
    const basePriceInQuote = new Decimal("0.25");
    assert.ok(
      convertDenomination(Denominator.Base, Denominator.Quote, new Decimal(8), basePriceInQuote)?.eq(2)
    );
    assert.ok(
      convertDenomination(Denominator.Quote, Denominator.Base, new Decimal(2), basePriceInQuote)?.eq(8)
    );
  });

  it("requires a positive price to cross denominations", () => {
    const amount = new Decimal(10);
    assert.equal(convertDenomination(Denominator.Base, Denominator.Quote, amount, new Decimal(0)), null);
    assert.equal(convertDenomination(Denominator.Quote, Denominator.Base, amount, new Decimal(-1)), null);
  });

  it("round-trips a conversion", () => {
    const basePriceInQuote = new Decimal("3.7");
    const amount = new Decimal(12);
    const quoteAmount = convertDenomination(Denominator.Base, Denominator.Quote, amount, basePriceInQuote);
    assert.ok(quoteAmount);
    const baseAmount = convertDenomination(Denominator.Quote, Denominator.Base, quoteAmount, basePriceInQuote);
    assert.ok(baseAmount?.eq(amount));
  });

  it("pads the market price in the side's fill direction", () => {
    const worstFillPrice = new Decimal(100);
    const protectionPercent = new Decimal(10);
    // buying: willing to pay more
    assert.ok(applyImpactProtection(Side.Bid, worstFillPrice, protectionPercent)?.eq(110));
    // selling: willing to receive less
    assert.ok(applyImpactProtection(Side.Ask, worstFillPrice, protectionPercent)?.eq(90));
  });

  it("requires positive impact-protection inputs", () => {
    assert.equal(applyImpactProtection(Side.Bid, new Decimal(0), new Decimal(10)), null);
    assert.equal(applyImpactProtection(Side.Ask, new Decimal(100), new Decimal(0)), null);
  });

  it("maps Side onto OrderSide", () => {
    assert.equal(toOrderSide(Side.Bid), OrderSide.BID);
    assert.equal(toOrderSide(Side.Ask), OrderSide.ASK);
  });
});
