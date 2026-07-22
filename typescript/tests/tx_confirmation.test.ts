/**
 * Tests for transaction confirmation — Rpc.confirmSignature.
 *
 * Drives the poll loop with a stubbed Connection: the stub returns a scripted
 * sequence of getSignatureStatuses results (repeating the last entry), so each
 * terminal outcome — confirmed, failed on-chain, expired, unknown — is
 * exercised without a network.
 */
import { describe, it } from "node:test";
import assert from "node:assert/strict";
import type { SignatureStatus } from "@solana/web3.js";

import { Rpc } from "../src/rpc";
import { SdkError } from "../src/error";
import { RpcFailoverState } from "../src/rpcFailover";
import type { ClientContext } from "../src/context";

const SIGNATURE = "TestSignature1111111111111111111111111111111111";

function status(
  confirmationStatus: SignatureStatus["confirmationStatus"],
  err: SignatureStatus["err"] = null
): SignatureStatus {
  return { slot: 1, confirmations: 1, err, confirmationStatus };
}

/**
 * Build a stub Connection whose getSignatureStatuses walks `sequence`
 * (repeating the final entry), throwing entries that are Errors.
 */
function stubRpc(
  sequence: Array<(SignatureStatus | null)[] | Error>,
  blockHeight = 0
): { rpc: Rpc; statusCalls: () => number } {
  let calls = 0;
  const connection = {
    async getSignatureStatuses(_signatures: string[]) {
      const next = sequence[Math.min(calls, sequence.length - 1)];
      calls += 1;
      if (next instanceof Error) throw next;
      return { context: { slot: 1 }, value: next };
    },
    async getBlockHeight(_commitment?: string) {
      return blockHeight;
    },
  };
  const ctx = {
    primaryConnection: connection,
    rpcFailoverState: new RpcFailoverState(),
  } as unknown as ClientContext;
  return { rpc: new Rpc(ctx), statusCalls: () => calls };
}

describe("Rpc.confirmSignature", () => {
  it("resolves once the signature reaches confirmed", async () => {
    const { rpc, statusCalls } = stubRpc([
      [status("processed")],
      [status("confirmed")],
    ]);
    await rpc.confirmSignature(SIGNATURE, 100);
    assert.equal(statusCalls(), 2);
  });

  it("throws TransactionFailed when the transaction landed with an error", async () => {
    const { rpc } = stubRpc([
      [status("confirmed", { InstructionError: [0, { Custom: 42 }] })],
    ]);
    await assert.rejects(
      () => rpc.confirmSignature(SIGNATURE, 100),
      (error: unknown) => {
        assert(error instanceof SdkError);
        assert.equal(error.variant, "TransactionFailed");
        assert.equal(error.signature, SIGNATURE);
        assert.match(error.message, /Custom/);
        return true;
      }
    );
  });

  it("throws TransactionExpired once the block height passes and the signature stays unseen", async () => {
    const { rpc } = stubRpc([[null]], 101);
    await assert.rejects(
      () => rpc.confirmSignature(SIGNATURE, 100),
      (error: unknown) => {
        assert(error instanceof SdkError);
        assert.equal(error.variant, "TransactionExpired");
        assert.equal(error.signature, SIGNATURE);
        return true;
      }
    );
  });

  it("still resolves when the signature confirms on the poll after expiry was observed", async () => {
    const { rpc, statusCalls } = stubRpc([[null], [status("confirmed")]], 101);
    await rpc.confirmSignature(SIGNATURE, 100);
    assert.equal(statusCalls(), 2);
  });

  it("throws ConfirmationTimeout after persistent status-poll failures", async () => {
    const { rpc, statusCalls } = stubRpc([new Error("boom")]);
    await assert.rejects(
      () => rpc.confirmSignature(SIGNATURE, 100),
      (error: unknown) => {
        assert(error instanceof SdkError);
        assert.equal(error.variant, "ConfirmationTimeout");
        assert.equal(error.signature, SIGNATURE);
        return true;
      }
    );
    assert.equal(statusCalls(), 3);
  });
});
