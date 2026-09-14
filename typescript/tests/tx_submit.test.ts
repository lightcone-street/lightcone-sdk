import assert from "node:assert/strict";
import { describe, it } from "node:test";
import { Keypair, type Connection } from "@solana/web3.js";
import {
  signAndSubmitTx,
  signAndSubmitInstructions,
  signAndSubmitPreparedTxConfirmedWithSlot,
  signAndSubmitTxConfirmedUsingStrategy,
  type ClientContext,
} from "../src/context";
import { LightconeClient } from "../src/client";
import { SdkError } from "../src/error";
import { Rpc } from "../src/rpc";
import { RpcFailoverState } from "../src/rpcFailover";
import { DepositSource } from "../src/shared";
import { Privy } from "../src/privy/client";
import type { ExternalSigner, SigningStrategy } from "../src/shared/signing";
import { V1Transaction } from "../src/program/transaction";
import { TEST_CONTEXT, transferTransaction } from "./v1_helpers";

const PAYER = Keypair.fromSeed(Buffer.alloc(32, 1));
const TRANSACTION = transferTransaction(PAYER);
const SIGNED = TRANSACTION.sign([PAYER]);
function harness(
  options: {
    fee?: number | null;
    balance?: number;
    feeError?: boolean;
    balanceError?: boolean;
    active?: boolean;
    simulationError?: unknown;
    sendError?: boolean;
    sendRpcError?: unknown;
    sendResult?: unknown;
    wrongSignature?: boolean;
    sponsored?: boolean;
    strategy?: SigningStrategy;
    onFee?: () => void;
  } = {},
) {
  const calls: Array<{ method: string; params: unknown[] }> = [];
  let signingCalls = 0;
  const signer: ExternalSigner = {
    walletAddress: PAYER.publicKey.toBase58(),
    async signMessage(bytes) {
      return bytes;
    },
    async signTransaction(bytes) {
      signingCalls++;
      return V1Transaction.fromWireBytes(bytes, TEST_CONTEXT)
        .sign([PAYER])
        .toWireBytes();
    },
  };
  const connection = {
    rpcEndpoint: "https://primary.example.invalid",
    async getBalance() {
      if (options.balanceError) throw Error("balance unavailable");
      return options.balance ?? 5_000;
    },
    async getLatestBlockhash() {
      throw Error("submission must retain its original lifetime");
    },
    async getBlockHeight() {
      return 99;
    },
    async getSignatureStatuses() {
      return {
        value: [
          {
            slot: 7,
            err: null,
            confirmations: 1,
            confirmationStatus: "confirmed",
          },
        ],
      };
    },
  } as unknown as Connection;
  const context = {
    primaryConnection: connection,
    backupConnection: {
      rpcEndpoint: "https://backup.example.invalid",
    } as Connection,
    rpcFailoverState: new RpcFailoverState(),
    signingStrategy: options.strategy ?? { type: "walletAdapter", signer },
    transactionSponsorshipEnabled: options.sponsored ?? false,
    transactionResources: TEST_CONTEXT.resources,
    depositSource: DepositSource.Global,
    rpcFetch: async (url: unknown, init: RequestInit) => {
      assert.equal(url, connection.rpcEndpoint);
      const request = JSON.parse(String(init.body));
      calls.push(request);
      let result: unknown;
      switch (request.method) {
        case "getFeeForMessage":
          options.onFee?.();
          if (options.feeError) throw Error("fee unavailable");
          result = { value: options.fee === undefined ? 5_000 : options.fee };
          break;
        case "getAccountInfo": {
          const data = Buffer.alloc(9);
          data[0] = options.active === false ? 0 : 1;
          data.writeBigUInt64LE(1n, 1);
          result = {
            context: { slot: 9 },
            value: {
              owner: "Feature111111111111111111111111111111111111",
              executable: false,
              data: [data.toString("base64"), "base64"],
            },
          };
          break;
        }
        case "simulateTransaction":
          result = {
            context: { slot: 9 },
            value: {
              err: options.simulationError ?? null,
              unitsConsumed: 123,
              loadedAccountsDataSize: 999,
              logs: ["ok"],
            },
          };
          break;
        case "sendTransaction":
          if (options.sendError) throw Error("connection reset: https://rpc.invalid/?api-key=do-not-expose");
          if (options.sendRpcError !== undefined)
            return Response.json({ error: options.sendRpcError, ...("sendResult" in options ? { result: options.sendResult } : {}) });
          result = options.wrongSignature ? "wrong" : SIGNED.signature;
          break;
        default:
          throw Error(`unexpected ${request.method}`);
      }
      return Response.json({ jsonrpc: "2.0", id: 1, result });
    },
  } as unknown as ClientContext;
  return { context, calls, signer, signingCalls: () => signingCalls };
}

describe("v1 transaction submission", () => {
  it("keeps sponsorship and resources independent in cloned clients", () => {
    const client = LightconeClient.builder()
      .transactionResources(TEST_CONTEXT.resources)
      .build();
    assert.equal(client.transactionSponsorshipEnabled, false);
    client.setTransactionSponsorshipEnabled(true);
    const clone = client.clone();
    client.setTransactionSponsorshipEnabled(false);
    assert.equal(clone.transactionSponsorshipEnabled, true);
    assert.deepEqual(clone.transactionResources, TEST_CONTEXT.resources);
  });

  it("estimates, signs, simulates, sends and confirms the exact immutable message", async () => {
    const { context, calls } = harness();
    assert.deepEqual(
      await signAndSubmitPreparedTxConfirmedWithSlot(context, TRANSACTION),
      { signature: SIGNED.signature, slot: 7 },
    );
    assert.equal(
      calls.find((c) => c.method === "getFeeForMessage")?.params[0],
      Buffer.from(TRANSACTION.messageBytes()).toString("base64"),
    );
    const simulation = calls.find((c) => c.method === "simulateTransaction");
    const send = calls.find((c) => c.method === "sendTransaction");
    assert.equal(
      simulation?.params[0],
      Buffer.from(SIGNED.toWireBytes()).toString("base64"),
    );
    assert.equal(send?.params[0], simulation?.params[0]);
    assert.deepEqual(simulation?.params[1], {
      encoding: "base64",
      commitment: "confirmed",
      sigVerify: true,
      replaceRecentBlockhash: false,
    });
    assert.deepEqual(send?.params[1], {
      encoding: "base64",
      skipPreflight: false,
      preflightCommitment: "confirmed",
      maxRetries: 0,
    });
    assert.equal(TRANSACTION.lastValidBlockHeight, 100);
    assert.throws(() => TRANSACTION.verifySignatures(), /missing or invalid/);
  });

  it("returns typed insufficiency before signing for ordinary and prepared submission", async () => {
    for (const submit of [
      signAndSubmitTx,
      signAndSubmitPreparedTxConfirmedWithSlot,
    ]) {
      const h = harness({ balance: 4_999 });
      await assert.rejects(
        submit(h.context, TRANSACTION),
        (error) =>
          error instanceof SdkError &&
          error.variant === "InsufficientSolForTransactionFees" &&
          error.availableLamports === 4_999n &&
          error.requiredLamports === 5_000n,
      );
      assert.equal(h.signingCalls(), 0);
      assert.deepEqual(
        h.calls.map((c) => c.method),
        ["getFeeForMessage"],
      );
    }
  });

  it("continues on unknown generic funding, including null fee estimates", async () => {
    for (const options of [
      { feeError: true },
      { balanceError: true },
      { fee: null },
      { balance: 6_000 },
    ]) {
      const h = harness(options);
      assert.equal(
        await signAndSubmitTx(h.context, TRANSACTION),
        SIGNED.signature,
      );
    }
  });

  it("captures sponsorship before asynchronous fee evidence", async () => {
    let h: ReturnType<typeof harness>;
    h = harness({
      balance: 1,
      onFee: () => {
        (
          h.context as { transactionSponsorshipEnabled: boolean }
        ).transactionSponsorshipEnabled = true;
      },
    });
    await assert.rejects(
      signAndSubmitTx(h.context, TRANSACTION),
      /Insufficient SOL/,
    );
    assert.equal(h.signingCalls(), 0);
  });

  it("permits an asserted external sponsor and rejects local sponsorship before RPC", async () => {
    const sponsored = harness({ sponsored: true });
    Object.assign(sponsored.signer, {
      walletAddress: Keypair.generate().publicKey.toBase58(),
    });
    assert.equal(
      await signAndSubmitTx(sponsored.context, TRANSACTION),
      SIGNED.signature,
    );
    assert.equal(
      sponsored.calls.some((c) => c.method === "getFeeForMessage"),
      false,
    );
    const native = harness({
      sponsored: true,
      strategy: { type: "native", keypair: PAYER },
    });
    await assert.rejects(
      signAndSubmitTx(native.context, TRANSACTION),
      /sponsorship is not supported/,
    );
    assert.equal(native.calls.length, 0);
  });

  it("rejects a mismatched known signer before funding evidence", async () => {
    const h = harness();
    Object.assign(h.signer, {
      walletAddress: Keypair.generate().publicKey.toBase58(),
    });
    await assert.rejects(
      signAndSubmitTx(h.context, TRANSACTION),
      /does not control/,
    );
    assert.equal(h.calls.length, 0);
  });

  it("rejects inactive clusters before prompting a wallet", async () => {
    const h = harness({ active: false });
    await assert.rejects(signAndSubmitTx(h.context, TRANSACTION), /inactive/);
    assert.equal(h.signingCalls(), 0);
  });

  it("rejects unsigned wallet responses and altered blockhashes before simulation or send", async () => {
    for (const mutated of [
      TRANSACTION,
      transferTransaction(PAYER, {
        ...TEST_CONTEXT,
        blockhash: Keypair.generate().publicKey.toBase58(),
      }).sign([PAYER]),
    ]) {
      const h = harness();
      h.signer.signTransaction = async () => mutated.toWireBytes();
      await assert.rejects(signAndSubmitTx(h.context, TRANSACTION));
      assert.equal(
        h.calls.some(
          (c) =>
            c.method === "sendTransaction" ||
            c.method === "simulateTransaction",
        ),
        false,
      );
    }
  });

  it("stops when signed simulation fails", async () => {
    const h = harness({
      simulationError: { InstructionError: [0, "InvalidArgument"] },
    });
    await assert.rejects(
      signAndSubmitTx(h.context, TRANSACTION),
      /simulation failed/,
    );
    assert.equal(
      h.calls.some((c) => c.method === "sendTransaction"),
      false,
    );
  });

  it("retains signature and expiry on an ambiguous send without retry or failover", async () => {
    for (const options of [{ sendError: true }, { wrongSignature: true }]) {
      const h = harness({
        ...options,
        strategy: { type: "native", keypair: PAYER },
      });
      await assert.rejects(
        signAndSubmitTx(h.context, TRANSACTION),
        (error) =>
          error instanceof SdkError &&
          error.variant === "SubmissionUnknown" &&
          error.signature === SIGNED.signature &&
          error.lastValidBlockHeight === 100 &&
          !String(error.stack).includes("do-not-expose") &&
          error.causeError === undefined,
      );
      assert.equal(
        h.calls.filter((c) => c.method === "sendTransaction").length,
        1,
      );
    }
  });

  it("distinguishes definite RPC rejections from uncertain outcomes without resending", async () => {
    for (const code of [-32700, -32600, -32601, -32602, -32002, -32003, -32005, -32006, -32013, -32015, -32016]) {
      const h = harness({ sendRpcError: { code, message: "request rejected", data: { err: "BlockhashNotFound" } } });
      await assert.rejects(new Rpc(h.context).submitSignedTransaction(SIGNED), error =>
        error instanceof SdkError && error.variant === "SubmissionRejected" &&
        error.rpcCode === code && error.signature === SIGNED.signature &&
        error.message.includes("request rejected"));
      assert.equal(h.calls.filter(call => call.method === "sendTransaction").length, 1);
    }
    for (const sendRpcError of [
      { code: -32002, message: "failed", data: { err: "AlreadyProcessed" } },
      { code: -32002, message: "This transaction has already been processed" },
      { code: -32603, message: "internal failure" },
      { code: -32099, message: "provider error" },
      { code: -32002 },
    ]) {
      const h = harness({ sendRpcError });
      await assert.rejects(new Rpc(h.context).submitSignedTransaction(SIGNED), error =>
        error instanceof SdkError && error.variant === "SubmissionUnknown" &&
        error.signature === SIGNED.signature && error.lastValidBlockHeight === 100);
      assert.equal(h.calls.filter(call => call.method === "sendTransaction").length, 1);
    }
  });

  it("fails over activation and simulation reads while preserving the signed bytes", async () => {
    for (const method of ["getAccountInfo", "simulateTransaction"]) {
      const h = harness();
      const calls: Array<{ url: string; method: string; params: unknown[] }> = [];
      const rpc = new Rpc({ ...h.context, rpcFetch: async (url, init) => {
        const request = JSON.parse(String(init?.body));
        calls.push({ url: String(url), ...request });
        if (request.method === method && String(url).includes("primary"))
          return new Response("unavailable", { status: 503 });
        return h.context.rpcFetch!(h.context.primaryConnection!.rpcEndpoint, init);
      } });
      assert.equal(await rpc.submitSignedTransaction(SIGNED), SIGNED.signature);
      assert.equal(calls.filter(call => call.method === method && call.url.includes("primary")).length, 2);
      assert.ok(calls.some(call => call.method === method && call.url.includes("backup")));
      const wire = Buffer.from(SIGNED.toWireBytes()).toString("base64");
      for (const call of calls.filter(call => ["simulateTransaction", "sendTransaction"].includes(call.method)))
        assert.equal(call.params[0], wire);
      assert.equal(calls.filter(call => call.method === "sendTransaction").length, 1);
    }
  });

  it("rejects shared and raw Privy submission before any network call", async () => {
    const h = harness({ strategy: { type: "privy", walletId: "wallet" } });
    await assert.rejects(
      signAndSubmitTx(h.context, TRANSACTION),
      /ExternalSigner/,
    );
    assert.equal(h.calls.length, 0);
    const privy = new Privy({
      http: {
        post() {
          throw Error("must not call HTTP");
        },
      } as never,
    });
    await assert.rejects(
      privy.signAndSendTx("wallet", "invalid"),
      /ExternalSigner/,
    );
  });

  it("preserves fee RPC integer tokens beyond Number.MAX_SAFE_INTEGER", async () => {
    const h = harness();
    const rpc = new Rpc({
      ...h.context,
      rpcFetch: async () =>
        new Response('{"result":{"value":18446744073709551615}}'),
    });
    assert.equal(
      await rpc.estimatePreparedTransactionFee(TRANSACTION),
      0xffff_ffff_ffff_ffffn,
    );
  });

  it("requires an unsponsored wallet identity before RPC", async () => {
    const h = harness();
    delete (h.signer as { walletAddress?: string }).walletAddress;
    await assert.rejects(
      signAndSubmitTx(h.context, TRANSACTION),
      /wallet identity is required/,
    );
    assert.equal(h.calls.length, 0);
  });

  it("rejects legacy objects at direct RPC transaction boundaries before network activity", async () => {
    const h = harness();
    const rpc = new Rpc(h.context);
    const legacy = {
      verifySignatures() {
        return false;
      },
    } as unknown as V1Transaction;
    await assert.rejects(
      rpc.estimatePreparedTransactionFee(legacy),
      /only validated Solana v1/,
    );
    await assert.rejects(
      rpc.simulateTransaction(legacy),
      /only validated Solana v1/,
    );
    await assert.rejects(
      rpc.submitSignedTransaction(legacy),
      /only validated Solana v1/,
    );
    assert.equal(h.calls.length, 0);
  });

  it("keeps read-only fee failover bound to the same canonical message", async () => {
    for (const networkFailure of [false, true]) {
      const h = harness(); const urls: string[] = []; const messages: string[] = [];
      const rpc = new Rpc({ ...h.context, rpcFetch: async (url, init) => {
        urls.push(String(url)); messages.push(JSON.parse(String(init?.body)).params[0]);
        if (urls.length < 3) {
          if (networkFailure) throw new TypeError("fetch failed");
          return new Response("unavailable", { status: 503 });
        }
        return Response.json({ result: { value: 8_000 } });
      } });
      assert.equal(await rpc.estimatePreparedTransactionFee(TRANSACTION), 8_000n);
      assert.deepEqual(urls, ["https://primary.example.invalid", "https://primary.example.invalid", "https://backup.example.invalid"]);
      assert.deepEqual(messages, Array(3).fill(Buffer.from(TRANSACTION.messageBytes()).toString("base64")));
    }
  });

  it("requires explicit resource configuration before blockhash RPC", async () => {
    const h = harness();
    delete (h.context as { transactionResources?: unknown })
      .transactionResources;
    await assert.rejects(
      new Rpc(h.context).transactionContext(),
      /resources are required/,
    );
  });
});


it("uses the configured RPC fetch for blockhashes on both managed endpoints", async () => {
  const urls: string[] = [];
  const client = LightconeClient.builder()
    .rpcUrl("https://primary.example.invalid")
    .backupRpcUrl("https://backup.example.invalid")
    .transactionResources(TEST_CONTEXT.resources)
    .rpcFetch(async (url, init) => {
      urls.push(String(url));
      if (String(url).includes("primary")) throw new TypeError("fetch failed");
      const request = JSON.parse(String(init?.body));
      assert.equal(request.method, "getLatestBlockhash");
      assert.deepEqual(request.params, [{ commitment: "confirmed" }]);
      return Response.json({ jsonrpc: "2.0", id: request.id,
        result: { context: { slot: 10 }, value: {
          blockhash: TEST_CONTEXT.blockhash, lastValidBlockHeight: 100,
        } },
      });
    }).build();
  assert.deepEqual(await client.transactionContext(), TEST_CONTEXT);
  assert.deepEqual(await client.clone().transactionContext(), TEST_CONTEXT);
  assert.deepEqual(urls, ["https://primary.example.invalid", "https://primary.example.invalid", "https://backup.example.invalid", "https://backup.example.invalid"]);
});

it("rejects malformed feature result shapes with an SDK error", async () => {
  for (const result of [null, false, 7, "invalid", [], {}, { value: null }, { context: null, value: {} }]) {
    const h = harness();
    const rpc = new Rpc({ ...h.context, rpcFetch: async () => Response.json({ result }) });
    await assert.rejects(rpc.ensureV1Supported(), error =>
      error instanceof SdkError && error.message.includes("feature is unavailable"));
  }
});


it("captures a builder signer before fetching its context", async () => {
  const h = harness({ strategy: { type: "native", keypair: PAYER } });
  h.context.primaryConnection!.getLatestBlockhash = async () => {
    Object.assign(h.context, { signingStrategy: { type: "native", keypair: Keypair.generate() } });
    return { blockhash: TEST_CONTEXT.blockhash, lastValidBlockHeight: TEST_CONTEXT.lastValidBlockHeight };
  };
  assert.equal(await signAndSubmitInstructions(h.context, TRANSACTION.instructions, PAYER.publicKey), SIGNED.signature);
  assert.equal(h.calls.filter(call => call.method === "sendTransaction").length, 1);
});


it("treats a present null result alongside an RPC error as an uncertain send", async () => {
  const h = harness({ sendRpcError: { code: -32002, message: "rejected" }, sendResult: null });
  await assert.rejects(new Rpc(h.context).submitSignedTransaction(SIGNED), error =>
    error instanceof SdkError && error.variant === "SubmissionUnknown" &&
    error.signature === SIGNED.signature && error.lastValidBlockHeight === 100);
  assert.equal(h.calls.filter(call => call.method === "sendTransaction").length, 1);
});


it("rejects malformed fee and simulation envelopes with SDK errors", async () => {
  for (const method of ["getFeeForMessage", "simulateTransaction"]) {
    for (const result of [null, false, 7, "invalid", [], {}]) {
      const h = harness();
      const rpc = new Rpc({ ...h.context, rpcFetch: async (url, init) => {
        const request = JSON.parse(String(init?.body));
        return request.method === method ? Response.json({ result }) : h.context.rpcFetch!(url, init);
      } });
      await assert.rejects(
        method === "getFeeForMessage" ? rpc.estimatePreparedTransactionFee(TRANSACTION) : rpc.simulateTransaction(SIGNED),
        error => error instanceof SdkError,
      );
    }
  }
});


it("keeps the original native keypair when the strategy is mutated during blockhash RPC", async () => {
  const strategy: SigningStrategy = { type: "native", keypair: PAYER };
  const replacement = Keypair.generate();
  const h = harness({ strategy });
  h.context.primaryConnection!.getLatestBlockhash = async () => {
    strategy.keypair = replacement;
    Object.assign(h.context, { transactionSponsorshipEnabled: true });
    return { blockhash: TEST_CONTEXT.blockhash, lastValidBlockHeight: TEST_CONTEXT.lastValidBlockHeight };
  };
  assert.equal(
    await signAndSubmitInstructions(h.context, TRANSACTION.instructions, PAYER.publicKey),
    SIGNED.signature,
  );
  assert.equal(strategy.keypair, replacement);
  assert.deepEqual(
    h.calls.filter(call => call.method === "sendTransaction").map(call => call.params[0]),
    [Buffer.from(SIGNED.toWireBytes()).toString("base64")],
  );
});

it("keeps the original native keypair during direct and confirmed submission RPC", async () => {
  for (const submit of [signAndSubmitTx, signAndSubmitPreparedTxConfirmedWithSlot, signAndSubmitTxConfirmedUsingStrategy]) {
    const strategy: SigningStrategy = { type: "native", keypair: PAYER };
    const replacement = Keypair.generate();
    const h = harness({ strategy, onFee: () => { strategy.keypair = replacement; } });
    const result = await submit(h.context, TRANSACTION, strategy);
    assert.equal(typeof result === "string" ? result : result.signature, SIGNED.signature);
    assert.equal(strategy.keypair, replacement);
    assert.deepEqual(
      h.calls.filter(call => call.method === "sendTransaction").map(call => call.params[0]),
      [Buffer.from(SIGNED.toWireBytes()).toString("base64")],
    );
  }
});

it("keeps the original external signer when its strategy is mutated during RPC", async () => {
  let strategy: Extract<SigningStrategy, { type: "walletAdapter" }>;
  const h = harness({ onFee: () => {
    strategy.signer = {
      walletAddress: PAYER.publicKey.toBase58(),
      async signMessage() { throw Error("replacement signer must not be called"); },
      async signTransaction() { throw Error("replacement signer must not be called"); },
    };
  } });
  assert.equal(h.context.signingStrategy?.type, "walletAdapter");
  strategy = h.context.signingStrategy as Extract<SigningStrategy, { type: "walletAdapter" }>;
  assert.equal(await signAndSubmitTx(h.context, TRANSACTION), SIGNED.signature);
  assert.equal(h.signingCalls(), 1);
  assert.deepEqual(
    h.calls.filter(call => call.method === "sendTransaction").map(call => call.params[0]),
    [Buffer.from(SIGNED.toWireBytes()).toString("base64")],
  );
});
