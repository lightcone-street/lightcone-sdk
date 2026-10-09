import { describe, it } from "node:test";
import assert from "node:assert/strict";

import { decimalTextToDecimal } from "../src/shared/decimal_text";
import { FundingSource } from "../src/shared/types";
import {
  fundingAccountFromWire,
  ObservationState,
  positionsResponseFromWire,
} from "../src/domain/position/wire";
import { LightconeClient } from "../src/client";

/** Shape emitted by `trading_wire::funding_account`, including the backend's u128 maximum aggregate fixture. */
function conditionalAccount(): Record<string, unknown> {
  return {
    account: "Acct1111111111111111111111111111111111111111",
    mint: "Cond1111111111111111111111111111111111111111",
    source: "conditional",
    market_pubkey: "A9Bxkkc4nah517EjgjwSafwspGnmU9s1Ei5PTo5ZkJd9",
    deposit_mint: "7SrxsoXjNR7Y8T3koJCt1yV4FrNUumoAUrJExDt6tQez",
    raw_observed: "0.000100",
    order_reserved: "340282366920938463463374607431768.211455",
    execution_reserved: "0.000040",
    signed_remaining_atoms: "-184467440737095516100",
    accepted_boundary: "boundary",
    observed_slot: "7",
    observed_blockhash: "hash",
    observation_state: "present",
  };
}

describe("positions wire", () => {
  it("decodes the backend positions page", () => {
    const response = positionsResponseFromWire({
      owner: "Wallet1111111111111111111111111111111111111",
      market_pubkey: null,
      funding_accounts: [
        conditionalAccount(),
        {
          account: "Glob1111111111111111111111111111111111111111",
          mint: "7SrxsoXjNR7Y8T3koJCt1yV4FrNUumoAUrJExDt6tQez",
          source: "global",
          market_pubkey: "",
          deposit_mint: "7SrxsoXjNR7Y8T3koJCt1yV4FrNUumoAUrJExDt6tQez",
          raw_observed: null,
          order_reserved: "0",
          execution_reserved: "0",
          signed_remaining_atoms: null,
          accepted_boundary: "",
          observed_slot: null,
          observed_blockhash: "",
          observation_state: "unobserved",
        },
      ],
      committed_revision: "275",
      projection_generation: "1",
      has_more: true,
      next_cursor: "Glob1111111111111111111111111111111111111111",
    });

    assert.equal(response.committed_revision, 275n);
    assert.equal(response.projection_generation, 1n);
    assert.equal(response.market_pubkey, undefined);
    assert.ok(response.has_more);
    const [conditional, global] = response.funding_accounts;
    assert.equal(conditional!.source, FundingSource.Conditional);
    assert.equal(conditional!.raw_observed, "0.000100");
    assert.equal(conditional!.order_reserved, "340282366920938463463374607431768.211455");
    assert.ok(decimalTextToDecimal(conditional!.execution_reserved).eq("0.00004"));
    assert.equal(conditional!.signed_remaining_atoms, -184_467_440_737_095_516_100n);
    assert.equal(conditional!.observed_slot, 7n);
    assert.equal(conditional!.observation_state, ObservationState.Present);

    assert.equal(global!.source, FundingSource.Global);
    assert.equal(global!.market_pubkey, undefined);
    assert.equal(global!.raw_observed, undefined);
    assert.equal(global!.signed_remaining_atoms, undefined);
    assert.equal(global!.observation_state, ObservationState.Unobserved);
  });

  it("keeps the market filter of a market-scoped page", () => {
    const response = positionsResponseFromWire({
      owner: "Wallet1111111111111111111111111111111111111",
      market_pubkey: "A9Bxkkc4nah517EjgjwSafwspGnmU9s1Ei5PTo5ZkJd9",
      funding_accounts: [],
      committed_revision: "9",
      projection_generation: "2",
      has_more: true,
      next_cursor: null,
    });
    assert.equal(response.market_pubkey, "A9Bxkkc4nah517EjgjwSafwspGnmU9s1Ei5PTo5ZkJd9");
    assert.equal(response.funding_accounts.length, 0);
    assert.ok(response.has_more);
  });

  it("decodes an unknown observation state without failing the page", () => {
    const account = fundingAccountFromWire({ ...conditionalAccount(), observation_state: "quarantined" });
    assert.equal(account.observation_state, ObservationState.Unknown);
  });

  it("names the field when an aggregate is not exact text", () => {
    assert.throws(
      () => fundingAccountFromWire({ ...conditionalAccount(), order_reserved: 12 }),
      /funding_account\.order_reserved/,
    );
  });
});

describe("positions client", () => {
  it("pages the authenticated routes and keeps the public routes query-free", async () => {
    const client = LightconeClient.builder().baseUrl("https://example.test").build();
    const urls: string[] = [];
    (client.http as unknown as { get: (url: string) => Promise<unknown> }).get = async (url) => {
      urls.push(url);
      return {
        owner: "Wallet1111111111111111111111111111111111111",
        funding_accounts: [],
        committed_revision: "1",
        projection_generation: "1",
        has_more: false,
      };
    };
    await client.positions().positionsPage(undefined, "next", 50);
    await client.positions().positionsPage("Market11111111111111111111111111111111111111");
    await client.positions().positionsForMarket("Market11111111111111111111111111111111111111");
    await client.positions().get("Wallet1111111111111111111111111111111111111");
    assert.deepEqual(urls, [
      "https://example.test/api/users/positions?cursor=next&limit=50",
      "https://example.test/api/users/markets/Market11111111111111111111111111111111111111/positions",
      "https://example.test/api/users/markets/Market11111111111111111111111111111111111111/positions",
      "https://example.test/api/users/Wallet1111111111111111111111111111111111111/positions",
    ]);
  });
});
