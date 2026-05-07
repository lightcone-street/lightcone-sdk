import { describe, it } from "node:test";
import assert from "node:assert/strict";
import {
  MarketResolutionKind,
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
    oracle: "oracle",
    question_id: "question",
    condition_id: "condition",
    market_status: "Resolved",
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
