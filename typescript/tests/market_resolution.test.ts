import { describe, it } from "node:test";
import assert from "node:assert/strict";
import {
  MarketResolutionKind,
  OutcomeValidationError,
  hasSingleWinningOutcome,
  isMarketResolved,
  marketFromWire,
  singleWinningOutcome,
  type MarketResolutionResponse,
  type MarketResponse,
} from "../src/domain/market";
import type { Notification } from "../src/domain/notification";
import { asPubkeyStr } from "../src/shared";

const NOW = "2026-05-06T13:00:00Z";

function scalarResolution(): MarketResolutionResponse {
  return {
    kind: MarketResolutionKind.Scalar,
    payout_denominator: 10,
    payouts: [
      { outcome_index: 0, payout_numerator: 7 },
      { outcome_index: 1, payout_numerator: 3 },
    ],
    single_winning_outcome: null,
  };
}

function singleWinnerResolution(): MarketResolutionResponse {
  return {
    kind: MarketResolutionKind.SingleWinner,
    payout_denominator: 1,
    payouts: [
      { outcome_index: 0, payout_numerator: 0 },
      { outcome_index: 1, payout_numerator: 1 },
    ],
    single_winning_outcome: 1,
  };
}

function marketResponse(
  resolution?: MarketResolutionResponse
): MarketResponse {
  return {
    market_name: "Test Market",
    slug: "test-market",
    description: "Description",
    definition: "Definition",
    outcomes: [
      {
        index: 0,
        name: "Yes",
        icon_url_low: "https://example.com/yes-low.png",
      },
      {
        index: 1,
        name: "No",
        icon_url_low: "https://example.com/no-low.png",
      },
    ],
    banner_image_url_low: "https://example.com/banner-low.png",
    icon_url_low: "https://example.com/icon-low.png",
    market_pubkey: "market_1",
    market_id: 1,
    num_outcomes: 2,
    oracle: "oracle",
    question_id: "question",
    condition_id: "condition",
    market_status: "Resolved",
    resolution_by: null,
    resolution,
    created_at: NOW,
    settled_at: NOW,
    deposit_assets: [
      {
        display_name: "USD Coin",
        symbol: "USDC",
        deposit_asset: "USDC",
        id: 1,
        market_pubkey: "market_1",
        vault: "vault",
        num_outcomes: 2,
        icon_url_low: "https://example.com/usdc-low.png",
        decimals: 6,
        min_order_size: "1.000000",
        conditional_mints: [
          {
            id: 10,
            outcome_index: 0,
            token_address: "yes_mint",
            outcome: "Yes",
            short_symbol: "YES",
            decimals: 6,
            created_at: NOW,
          },
          {
            id: 11,
            outcome_index: 1,
            token_address: "no_mint",
            outcome: "No",
            short_symbol: "NO",
            decimals: 6,
            created_at: NOW,
          },
        ],
        created_at: NOW,
      },
    ],
    orderbooks: [
      {
        id: 1,
        market_pubkey: "market_1",
        orderbook_id: "ob_yes_no",
        base_token: "yes_mint",
        quote_token: "no_mint",
        outcome_index: 0,
        tick_size: 1,
        total_bids: 0,
        total_asks: 0,
        active: true,
        created_at: NOW,
        updated_at: NOW,
      },
    ],
  };
}

describe("market metadata", () => {
  it("keeps outcomes valid when artwork is absent, null, or blank", () => {
    const response = marketResponse();
    response.outcomes[0] = {
      index: 0,
      name: "Yes",
      icon_url_low: null,
      icon_url_medium: null,
      icon_url_high: null,
    };
    response.outcomes[1] = {
      index: 1,
      name: "No",
      icon_url_low: " ",
      icon_url_medium: "https://example.com/no.png",
      icon_url_high: "",
    };

    const market = marketFromWire(response);
    assert.deepEqual(market.outcomes[0], {
      index: 0,
      name: "Yes",
      nameLong: undefined,
      iconUrlLow: undefined,
      iconUrlMedium: undefined,
      iconUrlHigh: undefined,
    });
    assert.equal(market.outcomes[1]?.iconUrlLow, "https://example.com/no.png");
    assert.equal(market.outcomes[1]?.iconUrlMedium, "https://example.com/no.png");
    assert.equal(market.outcomes[1]?.iconUrlHigh, "https://example.com/no.png");

    const preserved = marketFromWire(marketResponse()).outcomes[0];
    assert.equal(preserved?.iconUrlLow, "https://example.com/yes-low.png");
    assert.equal(preserved?.iconUrlMedium, "https://example.com/yes-low.png");
    assert.equal(preserved?.iconUrlHigh, "https://example.com/yes-low.png");

    const retainedError = new OutcomeValidationError("Yes", ["legacy"]);
    assert.equal(retainedError.name, "OutcomeValidationError");
    assert.deepEqual(retainedError.details, ["legacy"]);
  });

  it("converts markets without description, banners, subcategory, or tags", () => {
    const response = marketResponse();
    delete response.description;
    delete response.banner_image_url_low;

    const market = marketFromWire(response);
    assert.equal(market.description, undefined);
    assert.equal(market.definition, "Definition");
    assert.equal(market.bannerImageUrlLow, undefined);
    assert.equal(market.bannerImageUrlMedium, undefined);
    assert.equal(market.bannerImageUrlHigh, undefined);
    assert.equal(market.subcategory, undefined);
    assert.deepEqual(market.tags, []);
  });

  it("passes optional metadata through when present", () => {
    const response = marketResponse();
    response.subcategory = "Bitcoin";
    response.tags = ["btc"];
    response.resolution_by = 1_760_000_000_000;

    const market = marketFromWire(response);
    assert.equal(market.description, "Description");
    assert.equal(market.definition, "Definition");
    assert.equal(market.subcategory, "Bitcoin");
    assert.deepEqual(market.tags, ["btc"]);
    assert.equal(market.resolutionBy, 1_760_000_000_000);
  });

  it("maps a null resolution deadline to an absent domain value", () => {
    const market = marketFromWire(marketResponse());

    assert.equal(market.resolutionBy, undefined);
  });

  it("takes the authoritative outcome count from the market response", () => {
    const response = marketResponse();
    response.outcomes = [];

    const market = marketFromWire(response);

    assert.equal(market.numOutcomes, 2);
    assert.deepEqual(market.outcomes, []);
  });

  it("falls back to the deposit asset outcome count", () => {
    const response = marketResponse();
    delete response.num_outcomes;

    const market = marketFromWire(response);

    assert.equal(market.numOutcomes, 2);
  });

  it("rejects inconsistent deposit asset outcome counts", () => {
    const response = marketResponse();
    response.deposit_assets[0]!.num_outcomes = 3;

    assert.throws(() => marketFromWire(response), /do not match market/);
  });

  it("rejects markets without a non-empty string definition", () => {
    for (const definition of [undefined, "", 1]) {
      const response = marketResponse();
      (response as { definition?: unknown }).definition = definition;

      assert.throws(() => marketFromWire(response), /Missing definition/);
    }
  });

  it("cross-falls-back banner URLs when partially set", () => {
    const response = marketResponse();
    delete response.banner_image_url_low;
    response.banner_image_url_high = "https://example.com/banner-high.png";

    const market = marketFromWire(response);
    assert.equal(market.bannerImageUrlLow, "https://example.com/banner-high.png");
    assert.equal(market.bannerImageUrlMedium, "https://example.com/banner-high.png");
    assert.equal(market.bannerImageUrlHigh, "https://example.com/banner-high.png");
  });
});

describe("market resolution", () => {
  it("treats scalar resolution as resolved without a single winner", () => {
    const market = marketFromWire(marketResponse(scalarResolution()));

    assert.equal(isMarketResolved(market), true);
    assert.equal(singleWinningOutcome(market), undefined);
    assert.equal(hasSingleWinningOutcome(market), false);
    assert.equal(market.resolution?.kind, MarketResolutionKind.Scalar);
    assert.equal(market.resolution?.payout_denominator, 10);
    assert.deepEqual(
      market.resolution?.payouts.map((payout) => payout.payout_numerator),
      [7, 3],
    );
    assert.equal(market.depositAssets[0]?.minOrderSize?.toString(), "1");
  });

  it("derives the single winner from single-winner resolution", () => {
    const market = marketFromWire(marketResponse(singleWinnerResolution()));

    assert.equal(isMarketResolved(market), true);
    assert.equal(singleWinningOutcome(market), 1);
    assert.equal(hasSingleWinningOutcome(market), true);
    assert.equal(market.resolution?.kind, MarketResolutionKind.SingleWinner);
  });

  it("leaves unresolved markets distinct from scalar markets", () => {
    const market = marketFromWire(marketResponse());

    assert.equal(isMarketResolved(market), false);
    assert.equal(singleWinningOutcome(market), undefined);
    assert.equal(hasSingleWinningOutcome(market), false);
  });

  it("models market_resolved notifications with resolution payloads", () => {
    const notification: Notification = {
      id: "notif_1",
      notification_type: "market_resolved",
      data: {
        market_pubkey: asPubkeyStr("market_1"),
        market_slug: "test-market",
        market_name: "Test Market",
        resolution: scalarResolution(),
      },
      title: "Market resolved",
      message: "The market has resolved.",
      created_at: NOW,
    };

    assert.equal(notification.data.resolution?.kind, MarketResolutionKind.Scalar);
    assert.equal(notification.data.resolution?.single_winning_outcome, null);
  });
});
