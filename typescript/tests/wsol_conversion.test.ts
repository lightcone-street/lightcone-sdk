/** Import-safe safety-guard coverage for the WSOL conversion example. */
import assert from "node:assert/strict";
import { describe, it } from "node:test";

import {
  requireNonProduction,
  submitPreparedOnce,
  validateCoveringSnapshotSlot,
} from "../examples/wsol_conversion";

/** Every endpoint override that can redirect a nominally safe environment. */
const ENDPOINT_OVERRIDES = [
  "SDK_API_URL",
  "SDK_WS_URL",
  "SDK_RPC_URL",
  "SDK_PROGRAM_ID",
] as const;

describe("WSOL conversion example safety", () => {
  it("rejects implicit and explicit production", () => {
    assert.throws(
      () => requireNonProduction({}),
      /disabled in production/
    );
    assert.throws(
      () => requireNonProduction({ LIGHTCONE_ENV: "prod" }),
      /disabled in production/
    );
    assert.throws(
      () =>
        requireNonProduction({
          CI: "true",
          LIGHTCONE_ENV: "prod",
          SDK_RPC_URL: "https://example.invalid",
        }),
      /disabled in production/
    );
  });

  it("rejects every endpoint override in a non-production environment", () => {
    for (const overrideName of ENDPOINT_OVERRIDES) {
      assert.throws(
        () =>
          requireNonProduction({
            LIGHTCONE_ENV: "staging",
            [overrideName]: "https://example.invalid",
          }),
        new RegExp(`unset ${overrideName}`)
      );
    }
  });

  it("accepts built-in local and staging configuration", () => {
    assert.doesNotThrow(() => requireNonProduction({ LIGHTCONE_ENV: "local" }));
    assert.doesNotThrow(() =>
      requireNonProduction({ LIGHTCONE_ENV: "staging" })
    );
  });

  it("accepts a paid local RPC but rejects other local overrides", () => {
    assert.doesNotThrow(() =>
      requireNonProduction({
        LIGHTCONE_ENV: "local",
        SDK_RPC_URL: "https://example.invalid",
      })
    );
    for (const overrideName of [
      "SDK_API_URL",
      "SDK_WS_URL",
      "SDK_PROGRAM_ID",
    ] as const) {
      assert.throws(
        () =>
          requireNonProduction({
            LIGHTCONE_ENV: "local",
            [overrideName]: "https://example.invalid",
          }),
        new RegExp(`unset ${overrideName}`)
      );
    }
  });

  it("accepts workflow endpoints but not a program override in staging CI", () => {
    assert.doesNotThrow(() =>
      requireNonProduction({
        CI: "true",
        LIGHTCONE_ENV: "staging",
        SDK_API_URL: "https://api.dev.lightcone.xyz",
        SDK_WS_URL: "wss://ws.dev.lightcone.xyz/ws",
        SDK_RPC_URL: "https://example.invalid",
      })
    );
    assert.throws(
      () =>
        requireNonProduction({
          CI: "true",
          LIGHTCONE_ENV: "staging",
          SDK_PROGRAM_ID: "unsafe-program",
        }),
      /unset SDK_PROGRAM_ID/
    );
  });

  it("does not retry an uncertain prepared submission", async () => {
    const transaction = { prepared: true };
    let attempts = 0;

    await assert.rejects(
      submitPreparedOnce(transaction, async (submitted) => {
        attempts += 1;
        assert.equal(submitted, transaction);
        throw new Error("uncertain confirmation");
      }),
      /uncertain confirmation/
    );
    assert.equal(attempts, 1);
  });

  it("requires the authoritative snapshot to cover confirmation", () => {
    assert.doesNotThrow(() => validateCoveringSnapshotSlot(10, 10));
    assert.doesNotThrow(() => validateCoveringSnapshotSlot(11, 10));
    assert.throws(
      () => validateCoveringSnapshotSlot(9, 10),
      /below confirmed slot 10/
    );
  });
});
