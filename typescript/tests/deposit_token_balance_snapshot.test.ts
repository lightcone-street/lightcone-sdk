import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { PublicKey } from "@solana/web3.js";
import type { DepositTokenBalancesSnapshot } from "../src";
import type { ClientContext } from "../src/context";
import { Positions } from "../src/domain/position/client";
import type { RetryPolicy } from "../src/http";
import { RpcFailoverState } from "../src/rpcFailover";
import { DepositSource } from "../src/shared";

const snapshot: DepositTokenBalancesSnapshot = {
  context_slot: 1234,
  balances: {},
};

describe("Positions.depositTokenBalances", () => {
  it("forwards the minimum slot and per-call cookie", async () => {
    const requests: Array<{ url: string; cookie?: string }> = [];
    const http = {
      baseUrl: () => "https://api.example.test",
      get: async <T>(url: string, _retry: RetryPolicy): Promise<T> => {
        requests.push({ url });
        return snapshot as T;
      },
      getWithCookies: async <T>(
        url: string,
        _retry: RetryPolicy,
        cookie: string
      ): Promise<T> => {
        requests.push({ url, cookie });
        return snapshot as T;
      },
    };
    const client = {
      http,
      programId: new PublicKey("11111111111111111111111111111111"),
      depositSource: DepositSource.Global,
      rpcFailoverState: new RpcFailoverState(),
    } as unknown as ClientContext;
    const positions = new Positions(client);

    assert.deepEqual(await positions.depositTokenBalances(1234), snapshot);
    assert.deepEqual(
      await positions.depositTokenBalancesWithCookies(
        undefined,
        "lightcone-token=test"
      ),
      snapshot
    );
    assert.deepEqual(requests, [
      {
        url: "https://api.example.test/api/users/deposit-token-balances?min_context_slot=1234",
      },
      {
        url: "https://api.example.test/api/users/deposit-token-balances",
        cookie: "lightcone-token=test",
      },
    ]);
  });
});
