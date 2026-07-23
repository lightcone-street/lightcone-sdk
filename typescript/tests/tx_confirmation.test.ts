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
 * (repeating the final entry), throwing entries that are Errors. `history`
 * scripts responses to history-searching status calls; when omitted, those
 * calls fall through to `sequence`. `blockHeight` may be a single value or a
 * per-call sequence (repeating the last entry, throwing entries that are
 * Errors).
 */
function stubRpc(
  sequence: Array<(SignatureStatus | null)[] | Error>,
  blockHeight: number | Array<number | Error> = 0,
  history?: Array<(SignatureStatus | null)[]>
): {
  rpc: Rpc;
  statusCalls: () => number;
  historyCalls: () => number;
  heightCalls: () => number;
} {
  let calls = 0;
  let historyLookups = 0;
  let heightLookups = 0;
  const blockHeights = Array.isArray(blockHeight) ? blockHeight : [blockHeight];
  const connection = {
    async getSignatureStatuses(
      _signatures: string[],
      config?: { searchTransactionHistory?: boolean }
    ) {
      if (config?.searchTransactionHistory && history) {
        const next = history[Math.min(historyLookups, history.length - 1)];
        historyLookups += 1;
        return { context: { slot: 1 }, value: next };
      }
      const next = sequence[Math.min(calls, sequence.length - 1)];
      calls += 1;
      if (next instanceof Error) throw next;
      return { context: { slot: 1 }, value: next };
    },
    async getBlockHeight(_commitment?: string) {
      const next =
        blockHeights[Math.min(heightLookups, blockHeights.length - 1)];
      heightLookups += 1;
      if (next instanceof Error) throw next;
      return next;
    },
  };
  const ctx = {
    primaryConnection: connection,
    rpcFailoverState: new RpcFailoverState(),
  } as unknown as ClientContext;
  return {
    rpc: new Rpc(ctx),
    statusCalls: () => calls,
    historyCalls: () => historyLookups,
    heightCalls: () => heightLookups,
  };
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
    const { rpc, historyCalls } = stubRpc([[null]], 101, [[null]]);
    await assert.rejects(
      () => rpc.confirmSignature(SIGNATURE, 100),
      (error: unknown) => {
        assert(error instanceof SdkError);
        assert.equal(error.variant, "TransactionExpired");
        assert.equal(error.signature, SIGNATURE);
        return true;
      }
    );
    // Expiry is only declared after a history-searching check comes back empty.
    assert.equal(historyCalls(), 1);
  });

  it("still resolves when the signature confirms on the poll after expiry was observed", async () => {
    const { rpc, statusCalls } = stubRpc([[null], [status("confirmed")]], 101);
    await rpc.confirmSignature(SIGNATURE, 100);
    assert.equal(statusCalls(), 2);
  });

  it("resolves via the history check when a landed transaction left the status cache", async () => {
    const { rpc, historyCalls } = stubRpc([[null]], 101, [
      [status("confirmed")],
    ]);
    await rpc.confirmSignature(SIGNATURE, 100);
    assert.equal(historyCalls(), 1);
  });

  it("does not expire on a single skewed height sample", async () => {
    const { rpc, historyCalls, heightCalls } = stubRpc(
      [[null], [null], [status("confirmed")]],
      [101, 99]
    );
    await rpc.confirmSignature(SIGNATURE, 100);
    // One over-bound sample followed by an under-bound one resets the
    // streak, so no expiry (and no history lookup) happens.
    assert.equal(historyCalls(), 0);
    assert.equal(heightCalls(), 2);
  });

  it("restarts expiry evidence after a failed height poll", async () => {
    const { rpc, historyCalls, heightCalls } = stubRpc(
      [[null], [null], [null], [status("confirmed")]],
      [101, new Error("boom"), 101],
      [[null]]
    );
    await rpc.confirmSignature(SIGNATURE, 100);
    // The failed height poll broke the streak, so the over-bound readings on
    // either side of it never combined into an expiry declaration.
    assert.equal(historyCalls(), 0);
    assert.equal(heightCalls(), 3);
  });

  it("restarts expiry evidence after a processed sighting", async () => {
    const { rpc, statusCalls, historyCalls } = stubRpc(
      [[null], [status("processed")], [null], [status("confirmed")]],
      101,
      [[null]]
    );
    await rpc.confirmSignature(SIGNATURE, 100);
    // The processed sighting reset the over-bound streak, so the lone unseen
    // poll after it never reached the history/expiry stage.
    assert.equal(historyCalls(), 0);
    assert.equal(statusCalls(), 4);
  });

  it("restarts expiry evidence after a history sighting", async () => {
    const { rpc, historyCalls } = stubRpc(
      [[null], [null], [null], [status("confirmed")]],
      101,
      [[status("processed")]]
    );
    await rpc.confirmSignature(SIGNATURE, 100);
    // The history sighting reset the streak, so only one lookup happened
    // before the transaction confirmed.
    assert.equal(historyCalls(), 1);
  });

  it("never reports expiry when the bound is unknown", async () => {
    const { rpc, statusCalls, historyCalls } = stubRpc(
      [[null], [status("confirmed")]],
      101
    );
    await rpc.confirmSignature(SIGNATURE, null);
    // Without a bound the loop only polls statuses — no expiry machinery runs.
    assert.equal(statusCalls(), 2);
    assert.equal(historyCalls(), 0);
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
