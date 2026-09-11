import { ed25519 } from "@noble/curves/ed25519";
import { sha512 } from "@noble/hashes/sha512";
import nacl from "tweetnacl";
import assert from "node:assert/strict";
import { describe, it } from "node:test";
import {
  Keypair,
  PublicKey,
  SystemProgram,
  TransactionInstruction,
  ComputeBudgetProgram,
  Transaction,
  TransactionMessage,
  VersionedTransaction,
} from "@solana/web3.js";
import {
  V1Transaction,
  validateV1Resources,
} from "../src/program/transaction";
import { TEST_CONTEXT, transferTransaction } from "./v1_helpers";

describe("v1 validation and immutable signing", () => {
  const payer = Keypair.fromSeed(Buffer.alloc(32, 1));
  const tx = transferTransaction(payer);
  it("rejects legacy/v0 imports, malformed headers, bad indexes, masks, and trailing bytes", () => {
    const legacy = new Transaction({
      feePayer: payer.publicKey,
      recentBlockhash: TEST_CONTEXT.blockhash,
    }).add(...tx.instructions);
    const v0 = new VersionedTransaction(
      new TransactionMessage({
        payerKey: payer.publicKey,
        recentBlockhash: TEST_CONTEXT.blockhash,
        instructions: tx.instructions,
      }).compileToV0Message(),
    );
    for (const wire of [
      legacy.serialize({ requireAllSignatures: false }),
      v0.serialize(),
      Buffer.concat([tx.toWireBytes(), Buffer.from([0])]),
      tx.toWireBytes().slice(0, -1),
    ])
      assert.throws(() => V1Transaction.fromWireBytes(wire, TEST_CONTEXT));
    for (const [offset, byte] of [
      [1, 13],
      [2, 1],
      [4, 1],
      [7, 128],
      [41, 65],
    ]) {
      const wire = tx.toWireBytes();
      wire[offset] = byte;
      assert.throws(() => V1Transaction.fromWireBytes(wire, TEST_CONTEXT));
    }
  });
  it("validates explicit resource limits and exact bigint priority fees", () => {
    for (const override of [
      { computeUnitLimit: 0 },
      { computeUnitLimit: 1_400_001 },
      { loadedAccountsDataSizeLimit: 0 },
      { loadedAccountsDataSizeLimit: 67_108_865 },
      { heapSize: 32_769 },
      { heapSize: 263_168 },
      { priorityFeeLamports: -1n },
      { priorityFeeLamports: 1n << 64n },
      { priorityFeeLamports: 1 as unknown as bigint },
    ])
      assert.throws(() =>
        validateV1Resources({ ...TEST_CONTEXT.resources, ...override }),
      );
    assert.throws(
      () =>
        V1Transaction.compile(
          [ComputeBudgetProgram.setComputeUnitLimit({ units: 1 })],
          payer.publicKey,
          TEST_CONTEXT,
        ),
      /ComputeBudget/,
    );
    assert.throws(() =>
      V1Transaction.compile([], payer.publicKey, TEST_CONTEXT),
    );
    assert.throws(() =>
      V1Transaction.compile(tx.instructions, payer.publicKey, {
        ...TEST_CONTEXT,
        lastValidBlockHeight: 0,
      }),
    );
    assert.throws(
      () =>
        V1Transaction.compile(tx.instructions, payer.publicKey, {
          ...TEST_CONTEXT,
          blockhash: "invalid",
        }),
      (error) => error instanceof Error && error.name === "SdkError",
    );
  });
  it("includes every signature slot when enforcing the 4096-byte maximum", () => {
    const second = Keypair.fromSeed(Buffer.alloc(32, 3));
    const instruction = new TransactionInstruction({
      programId: SystemProgram.programId,
      keys: [{ pubkey: second.publicKey, isSigner: true, isWritable: false }],
      data: Buffer.alloc(1),
    });
    const overhead =
      V1Transaction.compile(
        [instruction],
        payer.publicKey,
        TEST_CONTEXT,
      ).toWireBytes().length - 1;
    instruction.data = Buffer.alloc(4096 - overhead);
    const atLimit = V1Transaction.compile(
      [instruction],
      payer.publicKey,
      TEST_CONTEXT,
    );
    assert.equal(atLimit.toWireBytes().length, 4096);
    assert.equal(
      atLimit.sign([second, payer, payer]).toWireBytes().length,
      4096,
    );
    instruction.data = Buffer.alloc(4097 - overhead);
    assert.throws(
      () => V1Transaction.compile([instruction], payer.publicKey, TEST_CONTEXT),
      /4096/,
    );
  });
  it("rejects a 65th distinct address and more than twelve signers", () => {
    const keys = Array.from({ length: 63 }, (_, i) => ({
      pubkey: new PublicKey(Buffer.alloc(32, i + 10)),
      isSigner: false,
      isWritable: false,
    }));
    const instruction = new TransactionInstruction({
      programId: SystemProgram.programId,
      keys,
      data: Buffer.alloc(0),
    });
    assert.throws(
      () => V1Transaction.compile([instruction], payer.publicKey, TEST_CONTEXT),
      /64/,
    );
    instruction.keys = keys
      .slice(0, 12)
      .map((key) => ({ ...key, isSigner: true }));
    assert.throws(
      () => V1Transaction.compile([instruction], payer.publicKey, TEST_CONTEXT),
      /12/,
    );
  });
  it("copies inputs and output bytes so mutation cannot change signed authority", () => {
    const resources = { ...TEST_CONTEXT.resources };
    const instructions = tx.instructions;
    const local = V1Transaction.compile(instructions, payer.publicKey, {
      ...TEST_CONTEXT,
      resources,
    });
    const before = local.messageBytes();
    resources.priorityFeeLamports = 9n;
    instructions[0].data.fill(255);
    local.instructions[0].data.fill(255);
    local.toWireBytes().fill(255);
    assert.deepEqual(local.messageBytes(), before);
    Reflect.set(local.context.resources, "priorityFeeLamports", 4n);
    assert.equal(
      local.context.resources.priorityFeeLamports,
      TEST_CONTEXT.resources.priorityFeeLamports,
    );
  });
  it("rejects noncanonical S, weak public keys, and weak R accepted by permissive verifiers", () => {
    const scalar = (bytes: Uint8Array): bigint =>
      BigInt(`0x${Buffer.from(bytes).reverse().toString("hex")}`);
    const encodeScalar = (value: bigint): Buffer =>
      Buffer.from(value.toString(16).padStart(64, "0"), "hex").reverse();
    const signed = tx.sign([payer]).toWireBytes();
    const noncanonical = Uint8Array.from(signed);
    noncanonical.set(
      encodeScalar(scalar(signed.slice(-32)) + ed25519.CURVE.n),
      signed.length - 32,
    );
    assert.equal(
      nacl.sign.detached.verify(
        tx.messageBytes(),
        noncanonical.slice(-64),
        payer.publicKey.toBytes(),
      ),
      true,
    );
    assert.throws(() => tx.acceptSignedBytes(noncanonical), /signature/);

    const identity = Buffer.alloc(32);
    identity[0] = 1;
    const weakPayer = new PublicKey(identity);
    const weak = V1Transaction.compile(
      [
        SystemProgram.transfer({
          fromPubkey: weakPayer,
          toPubkey: payer.publicKey,
          lamports: 1n,
        }),
      ],
      weakPayer,
      TEST_CONTEXT,
    );
    const weakSignature = Buffer.concat([identity, Buffer.alloc(32)]);
    assert.equal(
      nacl.sign.detached.verify(weak.messageBytes(), weakSignature, identity),
      true,
    );
    assert.throws(
      () =>
        weak.acceptSignedBytes(
          Buffer.concat([weak.messageBytes(), weakSignature]),
        ),
      /signature/,
    );

    // A known test key can produce a valid equation with nonce zero (R=identity).
    // Dalek rejects that weak R despite its otherwise valid message/signature equation.
    const extended = ed25519.utils.getExtendedPublicKey(
      payer.secretKey.slice(0, 32),
    );
    const challenge =
      scalar(
        sha512(
          Buffer.concat([
            identity,
            payer.publicKey.toBuffer(),
            tx.messageBytes(),
          ]),
        ),
      ) % ed25519.CURVE.n;
    const weakRSignature = Buffer.concat([
      identity,
      encodeScalar((challenge * extended.scalar) % ed25519.CURVE.n),
    ]);
    assert.equal(
      nacl.sign.detached.verify(
        tx.messageBytes(),
        weakRSignature,
        payer.publicKey.toBytes(),
      ),
      true,
    );
    assert.equal(
      ed25519.verify(
        weakRSignature,
        tx.messageBytes(),
        payer.publicKey.toBytes(),
        { zip215: false },
      ),
      true,
    );
    assert.throws(
      () =>
        tx.acceptSignedBytes(
          Buffer.concat([tx.messageBytes(), weakRSignature]),
        ),
      /signature/,
    );
  });

  it("rejects message changes even with valid replacement signatures", () => {
    for (const context of [
      { ...TEST_CONTEXT, blockhash: Keypair.generate().publicKey.toBase58() },
      {
        ...TEST_CONTEXT,
        resources: { ...TEST_CONTEXT.resources, priorityFeeLamports: 8n },
      },
      {
        ...TEST_CONTEXT,
        resources: { ...TEST_CONTEXT.resources, computeUnitLimit: 200_001 },
      },
    ]) {
      assert.throws(() =>
        tx.acceptSignedBytes(
          transferTransaction(payer, context).sign([payer]).toWireBytes(),
        ),
      );
    }
    const ix = tx.instructions[0];
    ix.data[4] = 7;
    assert.throws(
      () =>
        tx.acceptSignedBytes(
          V1Transaction.compile([ix], payer.publicKey, TEST_CONTEXT)
            .sign([payer])
            .toWireBytes(),
        ),
      /wallet changed/,
    );
    assert.throws(() => tx.acceptSignedBytes(tx.toWireBytes()), /signature/);
    const corrupt = tx.sign([payer]).toWireBytes();
    corrupt[corrupt.length - 1] ^= 1;
    assert.throws(() => tx.acceptSignedBytes(corrupt), /signature/);
    assert.throws(() => tx.sign([]), /missing required signer/);
  });
});
