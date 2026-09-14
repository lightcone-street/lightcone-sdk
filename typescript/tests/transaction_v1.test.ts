import { ed25519 } from "@noble/curves/ed25519";
import { sha512 } from "@noble/hashes/sha512";
import nacl from "tweetnacl";
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { getCompiledTransactionMessageDecoder, getCompiledTransactionMessageEncoder } from "@solana/kit";
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

describe("explicit zero priority fee imports", () => {
  it("preserves signed bytes while accepting equivalent zero-fee resources", () => {
    const payer = Keypair.fromSeed(Buffer.alloc(32, 1));
    const context = { ...TEST_CONTEXT, resources: { ...TEST_CONTEXT.resources, priorityFeeLamports: 0n } };
    const omitted = transferTransaction(payer, context);
    const decoded = getCompiledTransactionMessageDecoder().decode(omitted.messageBytes());
    assert.equal(decoded.version, 1);
    if (decoded.version !== 1) throw new Error("expected v1");
    const message = getCompiledTransactionMessageEncoder().encode({
      ...decoded, configMask: decoded.configMask | 3,
      configValues: [{ kind: "u64", value: 0n }, ...decoded.configValues],
    });
    const wire = Buffer.concat([message, nacl.sign.detached(message, payer.secretKey)]);
    const imported = V1Transaction.fromWireBytes(wire, context);
    imported.verifySignatures();
    assert.deepEqual(imported.toWireBytes(), Uint8Array.from(wire));
    assert.deepEqual(imported.acceptSignedBytes(wire).toWireBytes(), imported.toWireBytes());
    assert.throws(() => omitted.acceptSignedBytes(wire), /wallet changed/);
    assert.throws(() => V1Transaction.fromWireBytes(wire, {
      ...context, resources: { ...context.resources, priorityFeeLamports: 1n },
    }), /context/);
  });
});

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


it("pins cross-language compilation and signed encoding without fixture files", () => {
  const payer = Keypair.fromSeed(Buffer.alloc(32, 1));
  const second = Keypair.fromSeed(Buffer.alloc(32, 2));
  const key = (byte: number) => new PublicKey(Buffer.alloc(32, byte));
  const accounts = [
    { pubkey: second.publicKey, isSigner: true, isWritable: false },
    { pubkey: key(10), isSigner: false, isWritable: false },
    { pubkey: payer.publicKey, isSigner: true, isWritable: true },
    { pubkey: second.publicKey, isSigner: true, isWritable: false },
    { pubkey: key(8), isSigner: false, isWritable: true },
  ];
  const instruction = new TransactionInstruction({ programId: key(99), keys: accounts, data: Buffer.from([0, 1, 2, 127, 128, 255]) });
  const merged = new TransactionInstruction({ ...instruction, keys: [...accounts,
    { pubkey: key(10), isSigner: false, isWritable: true },
    { pubkey: second.publicKey, isSigner: true, isWritable: true },
  ] });
  // These digests are also asserted against upstream Rust and solders in their tests.
  const cases = [
    {
      resources: { computeUnitLimit: 1_400_000, loadedAccountsDataSizeLimit: 67_108_864, priorityFeeLamports: (1n << 64n) - 1n, heapSize: 262_144 },
      instructions: [instruction], signers: [payer, second],
      messageHash: "c5fdf05cc77574469ea3b7b08798722fda2bd225b1b8e35995d42f31e7011290",
      wireHash: "40156bfa4370bd072ca9000a3c42d1159be9c6c8b32cf598793a276b16f332c3",
    },
    {
      resources: { computeUnitLimit: 200_000, loadedAccountsDataSizeLimit: 1_048_576, priorityFeeLamports: 1001n, heapSize: 32_768 },
      instructions: [instruction, merged, SystemProgram.transfer({ fromPubkey: payer.publicKey, toPubkey: key(9), lamports: 42 })], signers: [payer, second],
      messageHash: "fdf9fd85a3963251ca654526c1fb6134f4e76ad361aa2b057bb3dfff379f41c4",
      wireHash: "368fb841224f6273bfc741ac09d88df014050640e8d3f4b31a38c05b1ef04809",
    },
    {
      resources: { computeUnitLimit: 200_000, loadedAccountsDataSizeLimit: 1_048_576, priorityFeeLamports: 0n },
      instructions: [
        new TransactionInstruction({ programId: key(99), keys: [
          { pubkey: key(99), isSigner: false, isWritable: true },
          { pubkey: key(98), isSigner: false, isWritable: true },
        ], data: Buffer.from([1, 2]) }),
        new TransactionInstruction({ programId: key(98), keys: [
          { pubkey: key(99), isSigner: false, isWritable: false },
          { pubkey: payer.publicKey, isSigner: true, isWritable: true },
        ], data: Buffer.from([3, 4]) }),
      ], signers: [payer],
      messageHash: "68e412b10a0ac0b912f9606cf2b62bc158e74ee25a2f5a7a5c20117584cfd7c8",
      wireHash: "e7629ba9b05db1fd502a408343395bb001aa0fdb48d659c59c1b06983f924307",
    },
  ];
  for (const { resources, instructions, signers, messageHash, wireHash } of cases) {
    const tx = V1Transaction.compile(instructions, payer.publicKey, { ...TEST_CONTEXT, blockhash: key(7).toBase58(), resources });
    const signed = tx.sign(signers);
    assert.equal(createHash("sha256").update(tx.messageBytes()).digest("hex"), messageHash);
    assert.equal(createHash("sha256").update(signed.toWireBytes()).digest("hex"), wireHash);
    signed.verifySignatures();
  }
});
