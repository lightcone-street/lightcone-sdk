import {
  PublicKey,
  TransactionInstruction,
  type Signer,
} from "@solana/web3.js";
import {
  address,
  blockhash,
  decompileTransactionMessage,
  getCompiledTransactionMessageDecoder,
  getCompiledTransactionMessageEncoder,
  getTransactionDecoder,
  getTransactionEncoder,
  type Transaction as KitTransaction,
  type V1CompiledTransactionMessage,
  type CompiledTransactionMessageWithLifetime,
} from "@solana/kit";
import nacl from "tweetnacl";
import { ed25519 } from "@noble/curves/ed25519";
import bs58 from "bs58";
import { SdkError } from "../error";

/** Explicit resource limits signed into every v1 transaction. */
export interface V1ResourceConfig {
  /** Maximum compute units, from 1 through 1,400,000. */
  readonly computeUnitLimit: number;
  /** Maximum loaded account data bytes, from 1 through 64 MiB. */
  readonly loadedAccountsDataSizeLimit: number;
  /** Total transaction priority fee in lamports, as an unsigned 64-bit integer. */
  readonly priorityFeeLamports: bigint;
  /** Heap bytes, 32–256 KiB in 1-KiB increments; omission requests the runtime default. */
  readonly heapSize?: number;
}

/** A recent blockhash, its paired expiry height, and caller-selected resources. */
export interface V1TransactionContext {
  readonly blockhash: string;
  /** Exact nonzero block height returned alongside blockhash by getLatestBlockhash. */
  readonly lastValidBlockHeight: number;
  readonly resources: V1ResourceConfig;
}

type CompiledV1 = V1CompiledTransactionMessage &
  CompiledTransactionMessageWithLifetime;
const COMPUTE_BUDGET = "ComputeBudget111111111111111111111111111111";

/** Validate resource units and runtime bounds without supplying defaults. */
export function validateV1Resources(resources: V1ResourceConfig): void {
  if (
    !resources ||
    !Number.isInteger(resources.computeUnitLimit) ||
    resources.computeUnitLimit < 1 ||
    resources.computeUnitLimit > 1_400_000
  ) {
    throw SdkError.validation("compute limit must be 1..=1,400,000");
  }
  if (
    !Number.isInteger(resources.loadedAccountsDataSizeLimit) ||
    resources.loadedAccountsDataSizeLimit < 1 ||
    resources.loadedAccountsDataSizeLimit > 67_108_864
  ) {
    throw SdkError.validation(
      "loaded account data limit must be 1..=67,108,864 bytes",
    );
  }
  if (
    typeof resources.priorityFeeLamports !== "bigint" ||
    resources.priorityFeeLamports < 0n ||
    resources.priorityFeeLamports > 0xffff_ffff_ffff_ffffn
  ) {
    throw SdkError.validation(
      "priority fee must be unsigned 64-bit total lamports",
    );
  }
  if (
    resources.heapSize !== undefined &&
    (!Number.isInteger(resources.heapSize) ||
      resources.heapSize < 32_768 ||
      resources.heapSize > 262_144 ||
      resources.heapSize % 1024 !== 0)
  ) {
    throw SdkError.validation("heap must be 32..=256 KiB in 1-KiB increments");
  }
}

function snapshotContext(context: V1TransactionContext): V1TransactionContext {
  validateV1Resources(context?.resources);
  if (
    !Number.isSafeInteger(context.lastValidBlockHeight) ||
    context.lastValidBlockHeight <= 0 ||
    context.blockhash === "11111111111111111111111111111111"
  ) {
    throw SdkError.validation(
      "a recent blockhash and its last valid block height are required",
    );
  }
  blockhash(context.blockhash);
  return Object.freeze({
    ...context,
    resources: Object.freeze({ ...context.resources }),
  });
}

function validateMessage(
  message: CompiledV1,
  context: V1TransactionContext,
): void {
  const { header, staticAccounts, instructionHeaders, instructionPayloads } =
    message;
  if (
    staticAccounts.length > 64 ||
    new Set(staticAccounts).size !== staticAccounts.length ||
    header.numSignerAccounts > 12 ||
    header.numReadonlySignerAccounts >= header.numSignerAccounts ||
    header.numSignerAccounts + header.numReadonlyNonSignerAccounts >
      staticAccounts.length
  ) {
    throw SdkError.validation(
      "invalid v1 account or signer counts (at most 64 addresses and 12 signers)",
    );
  }
  if (
    message.configMask & ~31 ||
    ((message.configMask & 3) !== 0 && (message.configMask & 3) !== 3)
  ) {
    throw SdkError.validation("invalid v1 resource configuration mask");
  }
  if (!instructionHeaders.length || instructionHeaders.length > 64)
    throw SdkError.validation("v1 transactions require 1..=64 instructions");
  for (let i = 0; i < instructionHeaders.length; i++) {
    const header = instructionHeaders[i];
    const payload = instructionPayloads[i];
    if (
      header.programAccountIndex === 0 ||
      header.programAccountIndex >= staticAccounts.length ||
      payload.instructionAccountIndices.some(
        (index) => index >= staticAccounts.length,
      )
    )
      throw SdkError.validation("invalid instruction account index");
    if (staticAccounts[header.programAccountIndex] === COMPUTE_BUDGET)
      throw SdkError.validation(
        "v1 transactions encode resources inline; ComputeBudget instructions are not supported",
      );
  }
  const decoded = decompileTransactionMessage(message);
  const config = decoded.config;
  const resources = context.resources;
  if (
    message.lifetimeToken !== context.blockhash ||
    config?.computeUnitLimit !== resources.computeUnitLimit ||
    config?.loadedAccountsDataSizeLimit !==
      resources.loadedAccountsDataSizeLimit ||
    config?.priorityFeeLamports !==
      (resources.priorityFeeLamports === 0n
        ? undefined
        : resources.priorityFeeLamports) ||
    config?.heapSize !== resources.heapSize
  ) {
    throw SdkError.validation(
      "message does not match its blockhash/resource context",
    );
  }
}

/**
 * Preserve Rust's merged account privileges and raw-address ordering.
 * Kit's compiler instead sorts base58 strings and rejects writable invoked
 * programs. The official Kit codecs still own every serialized wire field.
 */
function compileMessage(
  instructions: readonly TransactionInstruction[],
  payer: PublicKey,
  context: V1TransactionContext,
): CompiledV1 {
  if (instructions.length < 1 || instructions.length > 64)
    throw SdkError.validation("v1 transactions require 1..=64 instructions");
  type Account = { pubkey: PublicKey; isSigner: boolean; isWritable: boolean };
  const accounts = new Map<string, Account>();
  const merge = (
    pubkey: PublicKey,
    isSigner: boolean,
    isWritable: boolean,
  ): void => {
    const key = pubkey.toBase58();
    const existing = accounts.get(key);
    accounts.set(key, {
      pubkey,
      isSigner: isSigner || (existing?.isSigner ?? false),
      isWritable: isWritable || (existing?.isWritable ?? false),
    });
  };
  merge(payer, true, true);
  for (const instruction of instructions) {
    if (instruction.keys.length > 255 || instruction.data.length > 65_535)
      throw SdkError.validation(
        "v1 instruction account or data length exceeds its wire limit",
      );
    merge(instruction.programId, false, false);
    for (const key of instruction.keys)
      merge(key.pubkey, key.isSigner, key.isWritable);
  }
  if (accounts.size > 64)
    throw SdkError.validation("v1 transactions support at most 64 addresses");
  const group = (account: Account): number =>
    account.pubkey.equals(payer)
      ? 0
      : account.isSigner
        ? account.isWritable
          ? 1
          : 2
        : account.isWritable
          ? 3
          : 4;
  const ordered = Array.from(accounts.values()).sort(
    (a, b) =>
      group(a) - group(b) ||
      Buffer.compare(a.pubkey.toBuffer(), b.pubkey.toBuffer()),
  );
  const header = {
    numSignerAccounts: ordered.filter((account) => account.isSigner).length,
    numReadonlySignerAccounts: ordered.filter(
      (account) => account.isSigner && !account.isWritable,
    ).length,
    numReadonlyNonSignerAccounts: ordered.filter(
      (account) => !account.isSigner && !account.isWritable,
    ).length,
  };
  if (header.numSignerAccounts > 12)
    throw SdkError.validation("v1 transactions support at most 12 signers");
  const indexes = new Map(
    ordered.map((account, index) => [account.pubkey.toBase58(), index]),
  );
  const indexOf = (key: PublicKey): number => {
    const index = indexes.get(key.toBase58());
    if (index === undefined)
      throw SdkError.validation(
        "instruction account is missing from the compiled message",
      );
    return index;
  };
  const resources = context.resources;
  const priorityFeePresent = resources.priorityFeeLamports !== 0n;
  const configValues: CompiledV1["configValues"] = [];
  if (priorityFeePresent)
    configValues.push({ kind: "u64", value: resources.priorityFeeLamports });
  configValues.push(
    { kind: "u32", value: resources.computeUnitLimit },
    { kind: "u32", value: resources.loadedAccountsDataSizeLimit },
  );
  if (resources.heapSize !== undefined)
    configValues.push({ kind: "u32", value: resources.heapSize });
  const size =
    42 +
    32 * ordered.length +
    64 * header.numSignerAccounts +
    configValues.reduce(
      (bytes, value) => bytes + (value.kind === "u64" ? 8 : 4),
      0,
    ) +
    instructions.reduce(
      (bytes, instruction) =>
        bytes + 4 + instruction.keys.length + instruction.data.length,
      0,
    );
  if (size > 4096)
    throw SdkError.validation(
      "v1 transactions support at most 4096 bytes including every signature",
    );
  return {
    version: 1,
    header,
    lifetimeToken: blockhash(context.blockhash),
    configMask:
      (priorityFeePresent ? 3 : 0) |
      4 |
      8 |
      (resources.heapSize === undefined ? 0 : 16),
    configValues,
    numInstructions: instructions.length,
    numStaticAccounts: ordered.length,
    staticAccounts: ordered.map((account) =>
      address(account.pubkey.toBase58()),
    ),
    instructionHeaders: instructions.map((instruction) => ({
      programAccountIndex: indexOf(instruction.programId),
      numInstructionAccounts: instruction.keys.length,
      numInstructionDataBytes: instruction.data.length,
    })),
    instructionPayloads: instructions.map((instruction) => ({
      instructionAccountIndices: instruction.keys.map((key) =>
        indexOf(key.pubkey),
      ),
      instructionData: instruction.data,
    })),
  };
}

/**
 * Match Dalek verify_strict with a canonical scalar and nonweak points.
 * Noble 1.9 uses a cofactored verification equation even with zip215:false;
 * NaCl checks the uncofactored equation but needs these malleability guards.
 * Public-key decoding permits field reduction as Dalek does; R stays canonical.
 */
function verifyStrict(
  signature: Uint8Array,
  message: Uint8Array,
  publicKey: Uint8Array,
): boolean {
  try {
    const scalar = BigInt(
      `0x${Buffer.from(signature.subarray(32)).reverse().toString("hex")}`,
    );
    if (
      scalar >= ed25519.CURVE.n ||
      ed25519.Point.fromBytes(publicKey, true).isSmallOrder() ||
      ed25519.Point.fromBytes(signature.subarray(0, 32)).isSmallOrder()
    )
      return false;
    return nacl.sign.detached.verify(message, signature, publicKey);
  } catch {
    return false;
  }
}

/** Immutable v1 message and signatures, validated at every import boundary. */
export class V1Transaction {
  static readonly MAX_TRANSACTION_SIZE = 4096;
  static readonly MAX_ADDRESSES = 64;
  readonly #wire: Uint8Array;
  readonly #context: V1TransactionContext;

  private constructor(wire: Uint8Array, context: V1TransactionContext) {
    this.#wire = Uint8Array.from(wire);
    this.#context = context;
    Object.freeze(this);
  }

  /** Compile offline with all required signature slots included in size checks. */
  static compile(
    instructions: readonly TransactionInstruction[],
    payer: PublicKey,
    context: V1TransactionContext,
  ): V1Transaction {
    try {
      return V1Transaction.compileValidated(instructions, payer, context);
    } catch (error) {
      if (error instanceof SdkError) throw error;
      throw SdkError.validation(
        `invalid v1 transaction: ${error instanceof Error ? error.message : String(error)}`,
      );
    }
  }

  private static compileValidated(
    instructions: readonly TransactionInstruction[],
    payer: PublicKey,
    context: V1TransactionContext,
  ): V1Transaction {
    const snapshot = snapshotContext(context);
    const message = compileMessage(instructions, payer, snapshot);
    validateMessage(message, snapshot);
    const messageBytes = getCompiledTransactionMessageEncoder().encode(message);
    return V1Transaction.fromWireBytes(
      Buffer.concat([
        Uint8Array.from(messageBytes),
        Buffer.alloc(message.header.numSignerAccounts * 64),
      ]),
      snapshot,
    );
  }

  /** Import canonical v1 bytes only. Unsigned imports retain zeroed signature slots. */
  static fromWireBytes(
    wire: Uint8Array,
    context: V1TransactionContext,
  ): V1Transaction {
    try {
      if (wire.length > V1Transaction.MAX_TRANSACTION_SIZE || wire[0] !== 0x81)
        throw SdkError.validation(
          "only Solana v1 transactions of at most 4096 bytes are supported",
        );
      // Validate an owned snapshot, including when the caller supplied shared memory.
      wire = Uint8Array.from(wire);
      const snapshot = snapshotContext(context);
      const transaction = getTransactionDecoder().decode(wire);
      const message = getCompiledTransactionMessageDecoder().decode(
        transaction.messageBytes,
      );
      if (message.version !== 1)
        throw SdkError.validation("only Solana v1 transactions are supported");
      validateMessage(message, snapshot);
      if (
        !Buffer.from(
          getCompiledTransactionMessageEncoder().encode(message),
        ).equals(Buffer.from(transaction.messageBytes)) ||
        !Buffer.from(getTransactionEncoder().encode(transaction)).equals(
          Buffer.from(wire),
        )
      )
        throw SdkError.validation("noncanonical or trailing transaction bytes");
      return new V1Transaction(wire, snapshot);
    } catch (error) {
      if (error instanceof SdkError) throw error;
      throw SdkError.validation(
        `invalid v1 transaction: ${error instanceof Error ? error.message : String(error)}`,
      );
    }
  }

  get context(): V1TransactionContext {
    return this.#context;
  }
  get feePayer(): PublicKey {
    return new PublicKey(this.compiled().staticAccounts[0]);
  }
  get recentBlockhash(): string {
    return this.#context.blockhash;
  }
  get lastValidBlockHeight(): number {
    return this.#context.lastValidBlockHeight;
  }
  get requiredSigners(): PublicKey[] {
    const message = this.compiled();
    return message.staticAccounts
      .slice(0, message.header.numSignerAccounts)
      .map((key) => new PublicKey(key));
  }
  /** Return fresh decoded instruction objects; mutating them cannot change this message. */
  get instructions(): TransactionInstruction[] {
    const message = this.compiled();
    const header = message.header;
    return message.instructionHeaders.map(
      (ix, index) =>
        new TransactionInstruction({
          programId: new PublicKey(
            message.staticAccounts[ix.programAccountIndex],
          ),
          keys: message.instructionPayloads[
            index
          ].instructionAccountIndices.map((account) => ({
            pubkey: new PublicKey(message.staticAccounts[account]),
            isSigner: account < header.numSignerAccounts,
            isWritable:
              account < header.numSignerAccounts
                ? account <
                  header.numSignerAccounts - header.numReadonlySignerAccounts
                : account <
                  message.staticAccounts.length -
                    header.numReadonlyNonSignerAccounts,
          })),
          data: Buffer.from(message.instructionPayloads[index].instructionData),
        }),
    );
  }
  /** Canonical version-prefixed bytes signed by every required signer. */
  messageBytes(): Uint8Array {
    return Uint8Array.from(this.decoded().messageBytes);
  }
  /** Canonical transaction bytes, including every signature slot. */
  toWireBytes(): Uint8Array {
    return Uint8Array.from(this.#wire);
  }
  /** Sign every required key; duplicate supplied keys are harmless. */
  sign(signers: readonly Signer[]): V1Transaction {
    try {
      const byAddress = new Map(
        signers.map((signer) => [signer.publicKey.toBase58(), signer]),
      );
      const message = this.messageBytes();
      const signatures = this.requiredSigners.map((key) => {
        const signer = byAddress.get(key.toBase58());
        if (!signer)
          throw SdkError.signing(`missing required signer ${key.toBase58()}`);
        return nacl.sign.detached(message, signer.secretKey);
      });
      return this.acceptSignedBytes(Buffer.concat([message, ...signatures]));
    } catch (error) {
      if (error instanceof SdkError) throw error;
      throw SdkError.signing(
        error instanceof Error ? error.message : String(error),
      );
    }
  }
  /** Reject wallet message changes and verify every returned Ed25519 signature. */
  acceptSignedBytes(wire: Uint8Array): V1Transaction {
    const signed = V1Transaction.fromWireBytes(wire, this.#context);
    if (
      !Buffer.from(signed.messageBytes()).equals(
        Buffer.from(this.messageBytes()),
      )
    )
      throw SdkError.validation("wallet changed the transaction message");
    signed.verifySignatures();
    return signed;
  }
  /** Verify every required signature over the immutable canonical message. */
  verifySignatures(): void {
    const decoded = this.decoded();
    for (const key of this.requiredSigners) {
      const signature = decoded.signatures[address(key.toBase58())];
      if (
        !signature ||
        !verifyStrict(
          Uint8Array.from(signature),
          Uint8Array.from(decoded.messageBytes),
          key.toBytes(),
        )
      )
        throw SdkError.signing(
          `missing or invalid signature for ${key.toBase58()}`,
        );
    }
  }
  /** The first signature identifies a fully signed transaction for reconciliation. */
  get signature(): string {
    this.verifySignatures();
    const signature =
      this.decoded().signatures[address(this.feePayer.toBase58())];
    return bs58.encode(Uint8Array.from(signature as Uint8Array));
  }
  private decoded(): KitTransaction {
    return getTransactionDecoder().decode(this.#wire);
  }
  private compiled(): CompiledV1 {
    return getCompiledTransactionMessageDecoder().decode(
      this.decoded().messageBytes,
    ) as CompiledV1;
  }
}
