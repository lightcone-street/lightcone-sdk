import assert from "node:assert/strict";
import { describe, it } from "node:test";
import {
  AuthMethod,
  ChainType,
  displayName,
  type SessionResponse,
  type RegisterPrivyRequest,
  type User,
  type UserIdentity,
  type UserPrivyData,
  walletDisplayName,
} from "../src/auth";
import { Auth, classifyRegisterPrivyConflict } from "../src/auth/client";
import { SdkError } from "../src/error";
import type { LightconeHttp } from "../src/http";
import { RetryPolicy } from "../src/http";
import { ApiRejectedDetails } from "../src/shared";

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

function authWithHttp(
  http: LightconeHttp,
  setCredentials: (credentials: import("../src/auth").AuthCredentials | undefined) => void =
    () => {},
): Auth {
  return new Auth({
    http,
    authState: {
      getCredentials: () => undefined,
      setCredentials,
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
    assert.equal(
      walletDisplayName(wallet, AuthMethod.Lightcone),
      "1111...1111",
    );
    assert.equal(walletDisplayName(wallet, AuthMethod.Privy), "Toke...Q5DA");
    assert.equal(
      walletDisplayName(walletNoPrivy, AuthMethod.Privy),
      "1111...1111",
    );
  });
});

describe("email auth contract", () => {
  it("uses Email primary and linked identity shapes", () => {
    const email = user({
      type: "email",
      account: { email: "verified@example.com" },
      privy: privy("FRGkJho6fY7XivWsEBjousTaZBT6eUBkkrDyCN4nWcPR"),
    });
    email.linked_identities = [
      { type: "email", account: { email: "verified@example.com" } },
      { type: "google", account: { email: "verified@example.com" } },
    ];
    assert.equal(walletDisplayName(email, AuthMethod.Privy), "FRGk...WcPR");
  });

  it("limits Email display names to twenty characters", () => {
    const email = user({
      type: "email",
      account: { email: "lightconewebtesting@gmail.com" },
      privy: privy("FRGkJho6fY7XivWsEBjousTaZBT6eUBkkrDyCN4nWcPR"),
    });

    assert.equal(displayName(email), "lightcon...gmail.com");
    assert.equal([...displayName(email)].length, 20);
  });

  it("returns the synchronized session and installs refreshed credentials", async () => {
    const calls: unknown[][] = [];
    const registeredSession = session("5.50");
    registeredSession.auth_method = AuthMethod.Privy;
    const http = {
      baseUrl: () => "https://api.example.test",
      post: async (...args: unknown[]) => {
        calls.push(args);
        return registeredSession;
      },
    } as unknown as LightconeHttp;
    const request: RegisterPrivyRequest = {
      attempted_identity: { type: "email", email: "verified@example.com" },
    };
    let credentials: import("../src/auth").AuthCredentials | undefined;

    const result = await authWithHttp(http, (value) => {
      credentials = value;
    }).registerPrivy(request);

    assert.deepEqual(result, registeredSession);
    assert.equal(credentials?.user_id, registeredSession.user.user_id);
    assert.equal(
      credentials?.wallet_address,
      "11111111111111111111111111111111",
    );
    assert.deepEqual(calls, [
      [
        "https://api.example.test/api/auth/register-privy",
        request,
        RetryPolicy.Idempotent,
      ],
    ]);
  });
});

describe("max slippage preference", () => {
  it("uses the standard authenticated update contract", async () => {
    const calls: unknown[][] = [];
    const http = {
      baseUrl: () => "https://api.example.test",
      post: async (...args: unknown[]) => {
        calls.push(args);
        return { max_slippage_preference: "5.50" };
      },
    } as unknown as LightconeHttp;
    const auth = authWithHttp(http);

    const persisted = await auth.updateMaxSlippagePreference("5.50");

    assert.equal(persisted, "5.50");
    assert.deepEqual(calls, [
      [
        "https://api.example.test/api/auth/max_slippage_preference",
        { max_slippage_preference: "5.50" },
        RetryPolicy.Idempotent,
      ],
    ]);
  });

  it("accepts exact nullable session values", async () => {
    for (const preference of [null, "5.50"] as const) {
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
      (await authWithHttp(missingHttp).checkSession()).user
        .max_slippage_preference,
      null,
    );

    const malformed = {
      ...validSession,
      user: { ...validSession.user, max_slippage_preference: 10 },
    } as unknown as SessionResponse;
    const malformedHttp = {
      baseUrl: () => "https://api.example.test",
      get: async () => malformed,
    } as unknown as LightconeHttp;
    await assert.rejects(
      authWithHttp(malformedHttp).checkSession(),
      (error: unknown) => {
        return error instanceof SdkError && error.variant === "Serde";
      },
    );
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
        authWithHttp(http).updateMaxSlippagePreference("5.50"),
        (error: unknown) =>
          error instanceof SdkError && error.variant === "Serde",
      );
    }
  });

  it("classifies only exact register-privy conflict codes", () => {
    const conflict = SdkError.apiRejected(
      new ApiRejectedDetails({
        reason: "Identity belongs to another account",
        errorCode: "IDENTITY_OWNED_BY_ANOTHER_ACCOUNT",
        existingMethod: "google",
        httpStatus: 409,
      }),
    );
    assert.deepEqual(classifyRegisterPrivyConflict(conflict), {
      code: "IDENTITY_OWNED_BY_ANOTHER_ACCOUNT",
      existingMethod: "google",
    });

    const unrelated = SdkError.apiRejected(
      new ApiRejectedDetails({
        reason: "Conflict",
        errorCode: "RESOURCE_CONFLICT",
        existingMethod: "email",
        httpStatus: 409,
      }),
    );
    assert.equal(classifyRegisterPrivyConflict(unrelated), undefined);
  });
});
