/** Prepared-message submission tests for the external-wallet security boundary. */
import assert from "node:assert/strict";
import { describe, it } from "node:test";
import {
  Keypair,
  SystemProgram,
  Transaction,
  type Connection,
} from "@solana/web3.js";

import {
  signAndSubmitPreparedTxConfirmedWithSlot,
  type ClientContext,
} from "../src/context";
import { RpcFailoverState } from "../src/rpcFailover";
import { DepositSource } from "../src/shared";
import type { ExternalSigner } from "../src/shared/signing";

/** Deterministic signature returned by the fake RPC transport. */
const SIGNATURE = "prepared-signature";

/** Build an unsigned prepared transfer carrying the supplied fee-estimated blockhash. */
function preparedTransaction(blockhash: string): Transaction {
  const payer = Keypair.generate().publicKey;
  return new Transaction({ feePayer: payer, recentBlockhash: blockhash }).add(
    SystemProgram.transfer({
      fromPubkey: payer,
      toPubkey: Keypair.generate().publicKey,
      lamports: 1,
    })
  );
}

/** Build a client context that records the exact messages submitted to RPC. */
function contextFor(signer: ExternalSigner): {
  context: ClientContext;
  submittedMessages: Uint8Array[];
} {
  const submittedMessages: Uint8Array[] = [];
  const connection = {
    /** Record the submitted message while allowing signatures to vary. */
    async sendRawTransaction(bytes: Uint8Array) {
      submittedMessages.push(Transaction.from(bytes).serializeMessage());
      return SIGNATURE;
    },
    /** Return one deterministic confirmed processing slot. */
    async getSignatureStatuses() {
      return {
        context: { slot: 7 },
        value: [
          {
            slot: 7,
            confirmations: 1,
            err: null,
            confirmationStatus: "confirmed",
          },
        ],
      };
    },
  } as unknown as Connection;
  return {
    context: {
      primaryConnection: connection,
      rpcFailoverState: new RpcFailoverState(),
      signingStrategy: { type: "walletAdapter", signer },
      depositSource: DepositSource.Global,
    } as unknown as ClientContext,
    submittedMessages,
  };
}

/** Wallet adapter that adds no mutation, isolating the prepared-message path. */
const echoSigner: ExternalSigner = {
  /** Echo login bytes; login signing is outside these tests. */
  async signMessage(message) {
    return message;
  },
  /** Return transaction bytes unchanged to preserve the prepared message. */
  async signTransaction(transaction) {
    return transaction;
  },
};

describe("prepared transaction submission", () => {
  it("submits the exact fee-estimated message", async () => {
    const blockhash = Keypair.generate().publicKey.toBase58();
    const transaction = preparedTransaction(blockhash);
    const expectedMessage = transaction.serializeMessage();
    echoSigner.walletAddress = transaction.feePayer!.toBase58();
    const { context, submittedMessages } = contextFor(echoSigner);

    const confirmed = await signAndSubmitPreparedTxConfirmedWithSlot(
      context,
      transaction
    );

    assert.deepEqual(submittedMessages, [expectedMessage]);
    assert.equal(transaction.recentBlockhash, blockhash);
    assert.deepEqual(confirmed, { signature: SIGNATURE, slot: 7 });
  });

  it("rejects a wallet that replaces the prepared blockhash", async () => {
    const transaction = preparedTransaction(
      Keypair.generate().publicKey.toBase58()
    );
    /** Wallet adapter that preserves login bytes but replaces the transaction blockhash. */
    const rehashSigner: ExternalSigner = {
      walletAddress: transaction.feePayer!.toBase58(),
      /** Echo login bytes; login signing is outside this mutation test. */
      async signMessage(message) {
        return message;
      },
      /** Replace the prepared blockhash to exercise fail-before-submit validation. */
      async signTransaction(bytes) {
        const changed = Transaction.from(bytes);
        changed.recentBlockhash = Keypair.generate().publicKey.toBase58();
        return changed.serialize({
          requireAllSignatures: false,
          verifySignatures: false,
        });
      },
    };
    const { context, submittedMessages } = contextFor(rehashSigner);

    await assert.rejects(
      () => signAndSubmitPreparedTxConfirmedWithSlot(context, transaction),
      /changed the fee-prepared transaction message/
    );
    assert.equal(submittedMessages.length, 0);
  });

  it("rejects a mismatched signing wallet before signing or submission", async () => {
    const transaction = preparedTransaction(
      Keypair.generate().publicKey.toBase58()
    );
    let signingCalls = 0;
    const signer: ExternalSigner = {
      walletAddress: Keypair.generate().publicKey.toBase58(),
      async signMessage(message) {
        return message;
      },
      async signTransaction(bytes) {
        signingCalls += 1;
        return bytes;
      },
    };
    const { context, submittedMessages } = contextFor(signer);

    await assert.rejects(
      () => signAndSubmitPreparedTxConfirmedWithSlot(context, transaction),
      /does not control prepared transaction fee payer/
    );
    assert.equal(signingCalls, 0);
    assert.equal(submittedMessages.length, 0);
  });
});
