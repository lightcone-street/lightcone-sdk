import { describe, it } from "node:test";
import assert from "node:assert/strict";
import { Keypair, PublicKey } from "@solana/web3.js";
import { Positions } from "../src/domain/position/client";
import { InitPositionTokensBuilder } from "../src/domain/position/builders";
import { RpcFailoverState } from "../src/rpcFailover";
import { DepositSource } from "../src/shared";
import type { ClientContext } from "../src/context";

const client = {
  programId: new PublicKey("11111111111111111111111111111111"),
  depositSource: DepositSource.Global,
  rpcFailoverState: new RpcFailoverState(),
} as ClientContext;

function builder() {
  return new Positions(client)
    .withdrawFromPosition()
    .user(PublicKey.unique())
    .market(PublicKey.unique())
    .depositMint(PublicKey.unique())
    .amount(1n)
    .outcomeIndex(2);
}

describe("WithdrawFromPositionBuilder", () => {
  it("requires num_outcomes", () => {
    assert.throws(() => builder().buildIx(), /num_outcomes is required/);
  });

  it("validates against num_outcomes", () => {
    assert.throws(() => builder().numOutcomes(2).buildIx(), /Invalid outcome index/i);
  });
});

describe("InitPositionTokensBuilder", () => {
  it("prepares a beneficiary without a slot or lookup table", () => {
    const payer = PublicKey.unique();

    const ix = new InitPositionTokensBuilder(client)
      .payer(payer)
      .user(Keypair.fromSeed(Buffer.alloc(32, 2)).publicKey)
      .market(PublicKey.unique())
      .depositMints([PublicKey.unique()])
      .numOutcomes(2)
      .buildIx();

    assert.equal(ix.keys[0]!.pubkey.toBase58(), payer.toBase58());
    assert.equal(ix.keys[0]!.isSigner, true);
  });

  it("requires payer", () => {
    assert.throws(
      () => new InitPositionTokensBuilder(client).buildIx(),
      /payer is required/
    );
  });
});
