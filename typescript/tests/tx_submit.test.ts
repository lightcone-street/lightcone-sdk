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
  signAndSubmitTx,
  signAndSubmitPreparedTxConfirmedWithSlot,
  type ClientContext,
} from "../src/context";
import { LightconeClient } from "../src/client";
import { SdkError } from "../src/error";
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
function contextFor(
  signer: ExternalSigner,
  funding: {
    feeLamports?: number;
    balanceLamports?: number;
    feeError?: Error;
    balanceError?: Error;
    sponsored?: boolean;
  } = {}
): {
  context: ClientContext;
  submittedMessages: Uint8Array[];
  fundingCalls: { fee: number; balance: number };
} {
  const submittedMessages: Uint8Array[] = [];
  const fundingCalls = { fee: 0, balance: 0 };
  const connection = {
    async getLatestBlockhash() {
      return {
        blockhash: Keypair.generate().publicKey.toBase58(),
        lastValidBlockHeight: 100,
      };
    },
    async getFeeForMessage() {
      fundingCalls.fee += 1;
      if (funding.feeError) throw funding.feeError;
      return { context: { slot: 1 }, value: funding.feeLamports ?? 5_000 };
    },
    async getBalance() {
      fundingCalls.balance += 1;
      if (funding.balanceError) throw funding.balanceError;
      return funding.balanceLamports ?? 5_000;
    },
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
      transactionSponsorshipEnabled: funding.sponsored ?? false,
      depositSource: DepositSource.Global,
    } as unknown as ClientContext,
    submittedMessages,
    fundingCalls,
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
  it("exposes default-false capability and clone-by-value semantics", () => {
    const client = LightconeClient.builder().build();
    assert.equal(client.transactionSponsorshipEnabled, false);

    client.setTransactionSponsorshipEnabled(true);
    const clone = client.clone();
    client.setTransactionSponsorshipEnabled(false);

    assert.equal(client.transactionSponsorshipEnabled, false);
    assert.equal(clone.transactionSponsorshipEnabled, true);
  });

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

  it("returns the typed fee error before external signing or submission", async () => {
    const transaction = preparedTransaction(
      Keypair.generate().publicKey.toBase58()
    );
    let signingCalls = 0;
    const signer: ExternalSigner = {
      walletAddress: transaction.feePayer!.toBase58(),
      async signMessage(message) {
        return message;
      },
      async signTransaction(bytes) {
        signingCalls += 1;
        return bytes;
      },
    };
    const { context, submittedMessages } = contextFor(signer, {
      feeLamports: 5_000,
      balanceLamports: 4_999,
    });

    await assert.rejects(
      () => signAndSubmitPreparedTxConfirmedWithSlot(context, transaction),
      (error: unknown) => {
        assert.ok(error instanceof SdkError);
        assert.equal(error.variant, "InsufficientSolForTransactionFees");
        assert.equal(error.availableLamports, 4_999n);
        assert.equal(error.requiredLamports, 5_000n);
        assert.equal(
          error.message,
          "Insufficient SOL for transaction fees. Deposit SOL to your wallet and try again."
        );
        return true;
      }
    );
    assert.equal(signingCalls, 0);
    assert.equal(submittedMessages.length, 0);
  });

  it("applies the same typed funding guard to ordinary submission", async () => {
    const transaction = preparedTransaction(
      Keypair.generate().publicKey.toBase58()
    );
    transaction.recentBlockhash = undefined;
    let signingCalls = 0;
    const signer: ExternalSigner = {
      walletAddress: transaction.feePayer!.toBase58(),
      async signMessage(message) {
        return message;
      },
      async signTransaction(bytes) {
        signingCalls += 1;
        return bytes;
      },
    };
    const { context, submittedMessages } = contextFor(signer, {
      feeLamports: 5_000,
      balanceLamports: 4_999,
    });

    await assert.rejects(
      () => signAndSubmitTx(context, transaction),
      (error: unknown) =>
        error instanceof SdkError &&
        error.variant === "InsufficientSolForTransactionFees"
    );
    assert.equal(signingCalls, 0);
    assert.equal(submittedMessages.length, 0);
  });

  it("keeps the sponsorship value captured before blockhash lookup", async () => {
    const transaction = preparedTransaction(
      Keypair.generate().publicKey.toBase58()
    );
    transaction.recentBlockhash = undefined;
    const signer: ExternalSigner = {
      ...echoSigner,
      walletAddress: transaction.feePayer!.toBase58(),
    };
    const { context, submittedMessages } = contextFor(signer, {
      feeLamports: 5_000,
      balanceLamports: 4_999,
    });
    let releaseBlockhash!: () => void;
    const blockhashReleased = new Promise<void>((resolve) => {
      releaseBlockhash = resolve;
    });
    let markBlockhashStarted!: () => void;
    const blockhashStarted = new Promise<void>((resolve) => {
      markBlockhashStarted = resolve;
    });
    const connection = context.primaryConnection as Connection;
    connection.getLatestBlockhash = async () => {
      markBlockhashStarted();
      await blockhashReleased;
      return {
        blockhash: Keypair.generate().publicKey.toBase58(),
        lastValidBlockHeight: 100,
      };
    };

    const submission = signAndSubmitTx(context, transaction);
    await blockhashStarted;
    (
      context as ClientContext & { transactionSponsorshipEnabled: boolean }
    ).transactionSponsorshipEnabled = true;
    releaseBlockhash();

    await assert.rejects(
      submission,
      (error: unknown) =>
        error instanceof SdkError &&
        error.variant === "InsufficientSolForTransactionFees"
    );
    assert.equal(submittedMessages.length, 0);
  });

  it("preserves submission when either generic funding observation fails", async () => {
    for (const funding of [
      { feeError: new Error("fee unavailable") },
      { balanceError: new Error("balance unavailable") },
    ]) {
      const transaction = preparedTransaction(
        Keypair.generate().publicKey.toBase58()
      );
      const signer: ExternalSigner = {
        ...echoSigner,
        walletAddress: transaction.feePayer!.toBase58(),
      };
      const { context, submittedMessages } = contextFor(signer, funding);

      await signAndSubmitPreparedTxConfirmedWithSlot(context, transaction);

      assert.equal(submittedMessages.length, 1);
    }
  });

  it("continues when the fee-payer balance is strictly above the exact fee", async () => {
    const transaction = preparedTransaction(
      Keypair.generate().publicKey.toBase58()
    );
    const signer: ExternalSigner = {
      ...echoSigner,
      walletAddress: transaction.feePayer!.toBase58(),
    };
    const { context, submittedMessages } = contextFor(signer, {
      feeLamports: 5_000,
      balanceLamports: 5_001,
    });

    await signAndSubmitPreparedTxConfirmedWithSlot(context, transaction);

    assert.equal(submittedMessages.length, 1);
  });

  it("allows a sponsored external signer to differ from the prepared fee payer", async () => {
    const transaction = preparedTransaction(
      Keypair.generate().publicKey.toBase58()
    );
    const signer: ExternalSigner = {
      ...echoSigner,
      walletAddress: Keypair.generate().publicKey.toBase58(),
    };
    const { context, fundingCalls, submittedMessages } = contextFor(signer, {
      sponsored: true,
    });

    await signAndSubmitPreparedTxConfirmedWithSlot(context, transaction);

    assert.deepEqual(fundingCalls, { fee: 0, balance: 0 });
    assert.equal(submittedMessages.length, 1);
  });

  it("submits the prepared snapshot when the caller mutates after invocation", async () => {
    const transaction = preparedTransaction(
      Keypair.generate().publicKey.toBase58()
    );
    const expectedMessage = transaction.serializeMessage();
    const signer: ExternalSigner = {
      ...echoSigner,
      walletAddress: Keypair.generate().publicKey.toBase58(),
    };
    const { context, submittedMessages } = contextFor(signer, { sponsored: true });

    const submission = signAndSubmitPreparedTxConfirmedWithSlot(context, transaction);
    transaction.feePayer = Keypair.generate().publicKey;
    transaction.instructions[0].data = Buffer.from([255]);
    await submission;

    assert.deepEqual(submittedMessages, [expectedMessage]);
  });

  it("rejects sponsored prepared local signing before payer validation", async () => {
    const keypair = Keypair.generate();
    const transaction = preparedTransaction(
      Keypair.generate().publicKey.toBase58()
    );
    const { context } = contextFor(echoSigner, { sponsored: true });
    const nativeContext = {
      ...context,
      signingStrategy: { type: "native", keypair },
    } as ClientContext;

    await assert.rejects(
      () => signAndSubmitPreparedTxConfirmedWithSlot(nativeContext, transaction),
      /transaction sponsorship is not supported with local-keypair signing/
    );
    assert.equal(transaction.signatures.length, 0);
  });

  it("rejects sponsored local-keypair submission before signing", async () => {
    const keypair = Keypair.generate();
    const transaction = new Transaction({ feePayer: keypair.publicKey }).add(
      SystemProgram.transfer({
        fromPubkey: keypair.publicKey,
        toPubkey: Keypair.generate().publicKey,
        lamports: 1,
      })
    );
    const { context } = contextFor(echoSigner, { sponsored: true });
    const nativeContext = {
      ...context,
      signingStrategy: { type: "native", keypair },
    } as ClientContext;
    let blockhashCalls = 0;
    (nativeContext.primaryConnection as Connection).getLatestBlockhash = async () => {
      blockhashCalls += 1;
      throw new Error("blockhash lookup must not run");
    };

    await assert.rejects(
      () => signAndSubmitTx(nativeContext, transaction),
      /transaction sponsorship is not supported with local-keypair signing/
    );
    assert.equal(blockhashCalls, 0);
    assert.equal(transaction.recentBlockhash, undefined);
    assert.equal(transaction.signatures.length, 0);
  });

  it("rejects a known ordinary signer mismatch before funding RPC", async () => {
    const transaction = preparedTransaction(
      Keypair.generate().publicKey.toBase58()
    );
    transaction.recentBlockhash = undefined;
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
    const { context, fundingCalls, submittedMessages } = contextFor(signer);
    let blockhashCalls = 0;
    (context.primaryConnection as Connection).getLatestBlockhash = async () => {
      blockhashCalls += 1;
      throw new Error("blockhash lookup must not run");
    };

    await assert.rejects(
      () => signAndSubmitTx(context, transaction),
      /does not control transaction fee payer/
    );

    assert.equal(blockhashCalls, 0);
    assert.deepEqual(fundingCalls, { fee: 0, balance: 0 });
    assert.equal(signingCalls, 0);
    assert.equal(submittedMessages.length, 0);
  });

  it("submits the ordinary snapshot when the caller mutates during fee RPC", async () => {
    const transaction = preparedTransaction(
      Keypair.generate().publicKey.toBase58()
    );
    transaction.recentBlockhash = undefined;
    const originalPayer = transaction.feePayer!;
    const signer: ExternalSigner = {
      ...echoSigner,
      walletAddress: originalPayer.toBase58(),
    };
    const { context, submittedMessages } = contextFor(signer);
    let feeStartedResolve!: () => void;
    const feeStarted = new Promise<void>((resolve) => {
      feeStartedResolve = resolve;
    });
    let releaseFeeResolve!: () => void;
    const releaseFee = new Promise<void>((resolve) => {
      releaseFeeResolve = resolve;
    });
    let expectedMessage: Uint8Array | undefined;
    (context.primaryConnection as Connection).getFeeForMessage = async (message) => {
      expectedMessage = message.serialize();
      feeStartedResolve();
      await releaseFee;
      return { context: { slot: 1 }, value: 5_000 };
    };
    (context.primaryConnection as Connection).getBalance = async (feePayer) => {
      assert.equal(feePayer.toBase58(), originalPayer.toBase58());
      return 5_000;
    };

    const submission = signAndSubmitTx(context, transaction);
    await feeStarted;
    transaction.feePayer = Keypair.generate().publicKey;
    transaction.instructions[0].data = Buffer.from([255]);
    releaseFeeResolve();
    await submission;

    assert.ok(expectedMessage);
    assert.deepEqual(submittedMessages, [expectedMessage]);
  });

  it("rejects ordinary wallet mutation beyond a replacement blockhash", async () => {
    const transaction = preparedTransaction(
      Keypair.generate().publicKey.toBase58()
    );
    transaction.recentBlockhash = undefined;
    const signer: ExternalSigner = {
      walletAddress: transaction.feePayer!.toBase58(),
      async signMessage(message) {
        return message;
      },
      async signTransaction(bytes) {
        const changed = Transaction.from(bytes);
        changed.feePayer = Keypair.generate().publicKey;
        return changed.serialize({
          requireAllSignatures: false,
          verifySignatures: false,
        });
      },
    };
    const { context, submittedMessages } = contextFor(signer);

    await assert.rejects(
      () => signAndSubmitTx(context, transaction),
      /changed the transaction message beyond recent blockhash/
    );
    assert.equal(submittedMessages.length, 0);
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
      /does not control transaction fee payer/
    );
    assert.equal(signingCalls, 0);
    assert.equal(submittedMessages.length, 0);
  });

  it("does not retry or fail over an uncertain prepared submission", async () => {
    const transaction = preparedTransaction(
      Keypair.generate().publicKey.toBase58()
    );
    const signer: ExternalSigner = {
      ...echoSigner,
      walletAddress: transaction.feePayer!.toBase58(),
    };
    let primaryAttempts = 0;
    let backupAttempts = 0;
    const primaryConnection = {
      async sendRawTransaction() {
        primaryAttempts += 1;
        throw new TypeError("network response was lost");
      },
    } as unknown as Connection;
    const backupConnection = {
      async sendRawTransaction() {
        backupAttempts += 1;
        throw new TypeError("backup must not receive prepared bytes");
      },
    } as unknown as Connection;
    const context = {
      primaryConnection,
      backupConnection,
      rpcFailoverState: new RpcFailoverState(),
      signingStrategy: { type: "walletAdapter", signer },
      transactionSponsorshipEnabled: false,
      depositSource: DepositSource.Global,
    } as unknown as ClientContext;

    await assert.rejects(
      () => signAndSubmitPreparedTxConfirmedWithSlot(context, transaction),
      /network response was lost/
    );
    assert.equal(primaryAttempts, 1);
    assert.equal(backupAttempts, 0);
  });

  it("publishes the native signature before an uncertain prepared send", async () => {
    const keypair = Keypair.generate();
    const transaction = new Transaction({
      feePayer: keypair.publicKey,
      recentBlockhash: Keypair.generate().publicKey.toBase58(),
    }).add(
      SystemProgram.transfer({
        fromPubkey: keypair.publicKey,
        toPubkey: Keypair.generate().publicKey,
        lamports: 1,
      })
    );
    const { context } = contextFor(echoSigner);
    const nativeContext = {
      ...context,
      signingStrategy: { type: "native", keypair },
    } as ClientContext;
    let submittedSignature: Buffer | null = null;
    (nativeContext.primaryConnection as Connection).sendRawTransaction = async (
      bytes
    ) => {
      submittedSignature = Transaction.from(bytes).signatures[0].signature;
      throw new TypeError("network response was lost");
    };

    await assert.rejects(
      () => signAndSubmitPreparedTxConfirmedWithSlot(nativeContext, transaction),
      /network response was lost/
    );
    assert.ok(submittedSignature);
    assert.deepEqual(transaction.signatures[0].signature, submittedSignature);
  });
});
