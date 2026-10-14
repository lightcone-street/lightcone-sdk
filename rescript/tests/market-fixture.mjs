// Staging TypeScript fixture, shared as a wire oracle.
const NOW="2026-05-06T13:00:00Z";
export function marketFixture(){
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
