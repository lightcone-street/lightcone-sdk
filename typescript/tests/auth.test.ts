import assert from "node:assert/strict";
import { describe, it } from "node:test";
import {
  AuthMethod,
  ChainType,
  type User,
  type UserIdentity,
  type UserPrivyData,
  walletDisplayName,
} from "../src/auth";

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
  };
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
