import assert from "node:assert/strict";
import { describe, it } from "node:test";
import {
  AuthMethod,
  ChainType,
  type SessionResponse,
  type User,
  type UserIdentity,
  type UserPrivyData,
  walletDisplayName,
} from "../src/auth";
import { Auth } from "../src/auth/client";
import { SdkError } from "../src/error";
import type { LightconeHttp } from "../src/http";
import { RetryPolicy } from "../src/http";

function privy(address: string): UserPrivyData {
  return {
    id: "did:privy:test",
    wallet: {
      privy_id: "wallet:test",
      chain: ChainType.Solana,
      address,
    },
  };
}

function user(identity: UserIdentity): User {
  return {
    user_id: "user:test",
    identity,
    max_slippage_preference: null,
  };
}

function session(maxSlippagePreference: string | null): SessionResponse {
  return {
    user: {
      ...user({
        type: "wallet",
        address: "11111111111111111111111111111111",
        chain: ChainType.Solana,
      }),
      max_slippage_preference: maxSlippagePreference,
    },
    expires_at: 2_000_000_000,
    auth_method: AuthMethod.Lightcone,
    is_beta: false,
  };
}

function authWithHttp(http: LightconeHttp): Auth {
  return new Auth({
    http,
    authState: {
      getCredentials: () => undefined,
      setCredentials: () => {},
      clearCaches: async () => {},
    },
  });
}

describe("walletDisplayName", () => {
  it("uses the session trading wallet", () => {
    const googleWallet = "FRGkJho6fY7XivWsEBjousTaZBT6eUBkkrDyCN4nWcPR";
    const xWallet = "So11111111111111111111111111111111111111112";
    const signInWallet = "11111111111111111111111111111111";
    const embeddedWallet = "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA";

    const google = user({
      type: "google",
      account: {
        email: "user@example.com",
        name: "Google User",
      },
      privy: privy(googleWallet),
    });
    const x = user({
      type: "x",
      account: {
        user_id: "123",
        username: "x_user",
        display_name: "X User",
      },
      privy: privy(xWallet),
    });
    const wallet = user({
      type: "wallet",
      address: signInWallet,
      chain: ChainType.Solana,
      privy: privy(embeddedWallet),
    });
    const walletNoPrivy = user({
      type: "wallet",
      address: signInWallet,
      chain: ChainType.Solana,
    });

    assert.equal(walletDisplayName(google, AuthMethod.Privy), "FRGk...WcPR");
    assert.equal(walletDisplayName(x, AuthMethod.Privy), "So11...1112");
    assert.equal(walletDisplayName(wallet, AuthMethod.Lightcone), "1111...1111");
    assert.equal(walletDisplayName(wallet, AuthMethod.Privy), "Toke...Q5DA");
    assert.equal(walletDisplayName(walletNoPrivy, AuthMethod.Privy), "1111...1111");
  });
});

describe("max slippage preference", () => {
  it("uses the standard authenticated update contract", async () => {
    const calls: unknown[][] = [];
    const http = {
      baseUrl: () => "https://api.example.test",
      post: async (...args: unknown[]) => {
        calls.push(args);
        return { max_slippage_preference: "12.50" };
      },
    } as unknown as LightconeHttp;
    const auth = authWithHttp(http);

    const persisted = await auth.updateMaxSlippagePreference("12.50");

    assert.equal(persisted, "12.50");
    assert.deepEqual(calls, [
      [
        "https://api.example.test/api/auth/max_slippage_preference",
        { max_slippage_preference: "12.50" },
        RetryPolicy.Idempotent,
      ],
    ]);
  });

  it("accepts exact nullable session values", async () => {
    for (const preference of [null, "10.00"] as const) {
      const expected = session(preference);
      const http = {
        baseUrl: () => "https://api.example.test",
        get: async () => expected,
      } as unknown as LightconeHttp;

      assert.deepEqual(await authWithHttp(http).checkSession(), expected);
    }
  });

  it("normalizes missing session preference and rejects non-string values", async () => {
    const validSession = session(null);
    const missing = {
      ...validSession,
      user: {
        user_id: validSession.user.user_id,
        identity: validSession.user.identity,
      },
    } as unknown as SessionResponse;
    const missingHttp = {
      baseUrl: () => "https://api.example.test",
      get: async () => missing,
    } as unknown as LightconeHttp;
    assert.equal(
      (await authWithHttp(missingHttp).checkSession()).user.max_slippage_preference,
      null
    );

    const malformed = {
      ...validSession,
      user: { ...validSession.user, max_slippage_preference: 10 },
    } as unknown as SessionResponse;
    const malformedHttp = {
      baseUrl: () => "https://api.example.test",
      get: async () => malformed,
    } as unknown as LightconeHttp;
    await assert.rejects(authWithHttp(malformedHttp).checkSession(), (error: unknown) => {
      return error instanceof SdkError && error.variant === "Serde";
    });
  });

  it("rejects missing, null, or non-string update values", async () => {
    const malformedResponses = [
      {},
      { max_slippage_preference: null },
      { max_slippage_preference: 12.5 },
    ];
    for (const response of malformedResponses) {
      const http = {
        baseUrl: () => "https://api.example.test",
        post: async () => response,
      } as unknown as LightconeHttp;

      await assert.rejects(
        authWithHttp(http).updateMaxSlippagePreference("12.50"),
        (error: unknown) => error instanceof SdkError && error.variant === "Serde"
      );
    }
  });
});
