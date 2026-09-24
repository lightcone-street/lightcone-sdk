import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { Jurisdiction } from "../src/domain/jurisdiction";
import type { ClientContext } from "../src/context";
import type { JurisdictionClient, JurisdictionResponse } from "../src/prelude";

const capabilities = {
  mode: "full",
  can_authenticate: true,
  can_mutate_account: true,
  can_submit_orders: true,
  can_cancel_orders: true,
};

function wire() {
  return {
    country: "US",
    geoblocked: false,
    tier: "tier_3",
    policy_version: "test",
    frontend: capabilities,
    api: capabilities,
    relayed: false,
  };
}

describe("direct jurisdiction", () => {
  it("uses the API transport and maps every capability", async () => {
    let requested = "";
    const context = {
      http: {
        baseUrl: () => "https://api.example.test",
        get: async (url: string) => {
          requested = url;
          return wire();
        },
      },
    } as unknown as ClientContext;
    const client: JurisdictionClient = new Jurisdiction(context);
    const result: JurisdictionResponse = await client.geoblock();
    assert.equal(requested, "https://api.example.test/api/geoblock");
    assert.equal(result.country, "US");
    assert.equal(result.policyVersion, "test");
    assert.equal(result.frontend.canSubmitOrders, true);
    assert.equal(result.api.canCancelOrders, true);
    assert.equal(result.relayed, false);
  });

  it("rejects an absent relay stamp", async () => {
    const { relayed: _removed, ...withoutStamp } = wire();
    const context = {
      http: {
        baseUrl: () => "https://api.example.test",
        get: async () => withoutStamp,
      },
    } as unknown as ClientContext;
    await assert.rejects(() => new Jurisdiction(context).geoblock(), /relay stamp/);
  });
});
