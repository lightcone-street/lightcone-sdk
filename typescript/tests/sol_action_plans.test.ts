/** Cross-SDK SOL planning invariants at RPC, account, and instruction boundaries. */
import assert from "node:assert/strict";
import { describe, it } from "node:test";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  decodeCloseAccountInstruction,
  decodeSyncNativeInstruction,
  decodeTransferInstruction,
  getAssociatedTokenAddressSync,
  NATIVE_MINT,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import {
  Keypair,
  PublicKey,
  SystemInstruction,
  SystemProgram,
  type Connection,
} from "@solana/web3.js";

import type { ClientContext } from "../src/context";
import {
  Rpc,
  SdkError,
  unwrapAllSolBalanceAvailability,
  type CanonicalWsolAccountInfo,
} from "../src";
import type { Market } from "../src/domain/market";
import {
  nativeWithdrawSeed,
  Positions,
} from "../src/domain/position/client";
import type { DepositTokenBalance } from "../src/domain/position";
import {
  solBalanceAvailability,
  WalletDepositBalancesState,
  WRAPPED_SOL_MINT,
} from "../src/domain/position/state";
import { DepositSource, type PubkeyStr } from "../src/shared";
import type { SigningStrategy } from "../src/shared/signing";
import { RpcFailoverState } from "../src/rpcFailover";

/** Build complete wallet authority with exact native and canonical components. */
function stateFor(
  wallet: PublicKey,
  native: string,
  wrapped?: string
): WalletDepositBalancesState {
  const state = new WalletDepositBalancesState();
  const balances = {} as Record<PubkeyStr, DepositTokenBalance>;
  if (wrapped !== undefined) {
    balances[WRAPPED_SOL_MINT] = {
      mint: WRAPPED_SOL_MINT,
      idle: wrapped,
      symbol: "WSOL",
      name: "Wrapped SOL",
    };
  }
  state.applyRestSnapshot(wallet.toBase58() as PubkeyStr, {
    context_slot: 1,
    native_sol_balance: native,
    balances,
  });
  return state;
}

/** Render exact lamports as canonical nine-decimal SOL text for state fixtures. */
function solText(lamports: bigint): string {
  const scale = 1_000_000_000n;
  return `${lamports / scale}.${(lamports % scale).toString().padStart(9, "0")}`;
}

/** Mutable decoded fields used to exercise every canonical-account guard. */
interface CanonicalAccountDataOptions {
  mint?: PublicKey;
  authority?: PublicKey;
  delegateOption?: number;
  state?: number;
  isNative?: boolean;
  isNativeOption?: number;
  nativeReserve?: bigint;
  closeAuthority?: PublicKey;
  closeAuthorityOption?: number;
  dataLength?: number;
}

/** Encode one exact legacy-token account for deterministic RPC validation. */
function canonicalAccountData(
  wallet: PublicKey,
  tokenAmount: bigint,
  options: CanonicalAccountDataOptions = {}
): Buffer {
  const data = Buffer.alloc(165);
  (options.mint ?? NATIVE_MINT).toBuffer().copy(data, 0);
  (options.authority ?? wallet).toBuffer().copy(data, 32);
  data.writeBigUInt64LE(tokenAmount, 64);
  data.writeUInt32LE(options.delegateOption ?? 0, 72);
  data[108] = options.state ?? 1;
  const isNativeOption =
    options.isNativeOption ?? ((options.isNative ?? true) ? 1 : 0);
  data.writeUInt32LE(isNativeOption, 109);
  if (isNativeOption !== 0) {
    data.writeBigUInt64LE(options.nativeReserve ?? 2_039_280n, 113);
  }
  const closeAuthorityOption =
    options.closeAuthorityOption ?? (options.closeAuthority ? 1 : 0);
  data.writeUInt32LE(closeAuthorityOption, 129);
  if (options.closeAuthority) {
    options.closeAuthority.toBuffer().copy(data, 133);
  }
  return options.dataLength === undefined
    ? data
    : data.subarray(0, options.dataLength);
}

/** Deterministic chain authority exposed to one planner test. */
interface PlanningHarnessOptions {
  /** Whether the persistent canonical Tokenkeg ATA exists. */
  canonicalExists?: boolean;
  /** Return an occupied address that is not a valid canonical token account. */
  invalidCanonicalAccount?: boolean;
  /** Full account lamports returned by JSON RPC before exact conversion. */
  canonicalAccountLamports?: number;
  /** Decoded canonical token amount used for standalone state matching. */
  canonicalTokenAmount?: bigint;
  /** Overrides for decoded mint, authority, lifecycle, native flag, or size. */
  canonicalData?: CanonicalAccountDataOptions;
  /** Number of deterministic temporary seeds reported as already occupied. */
  occupiedTemporaryAttempts?: number;
  /** Ordered live fee results; `null` models unavailable fee authority. */
  feeValues?: Array<number | null>;
  /** Rent-exempt minimum returned for the 165-byte temporary allocation. */
  rentLamports?: number;
  /** Credential expiry used at the planning identity boundary. */
  expiresAt?: Date;
  /** Credential wallet override for stale/wrong-wallet tests. */
  credentialWallet?: PublicKey;
  /** Signing wallet override for trust-boundary mismatch tests. */
  signingWallet?: Keypair;
  /** Exact signing strategy override for native-only conversion tests. */
  signingStrategy?: SigningStrategy;
  /** Omit authenticated credentials at the planner authority boundary. */
  authenticated?: boolean;
  /** Omit the signing strategy at the planner authority boundary. */
  hasSigningStrategy?: boolean;
  /** Ordered blockhashes used to prove final seed/message binding. */
  blockhashValues?: string[];
}

/** Build a planner with deterministic RPC responses and account-read recording. */
function planningHarness(
  wallet: Keypair,
  options: PlanningHarnessOptions = {}
): {
  positions: Positions;
  rpc: Rpc;
  accountLookups: PublicKey[];
  feeLookups: unknown[];
} {
  const canonical = getAssociatedTokenAddressSync(NATIVE_MINT, wallet.publicKey);
  const accountLookups: PublicKey[] = [];
  const feeLookups: unknown[] = [];
  const canonicalTokenAmount = options.canonicalTokenAmount ?? 0n;
  const canonicalNativeReserve =
    options.canonicalData?.nativeReserve ?? 2_039_280n;
  let occupiedTemporaryAttempts = options.occupiedTemporaryAttempts ?? 0;
  const feeValues = [...(options.feeValues ?? [5_000])];
  const blockhashValues = [
    ...(options.blockhashValues ?? ["11111111111111111111111111111111"]),
  ];
  const connection = {
    /** Return ordered blockhash authority while retaining the final fallback. */
    getLatestBlockhash: async () => ({
      blockhash:
        blockhashValues.length > 1
          ? blockhashValues.shift()!
          : blockhashValues[0]!,
      lastValidBlockHeight: 100,
    }),
    /** Distinguish canonical existence and bounded temporary-seed collisions. */
    getAccountInfo: async (address: PublicKey) => {
      accountLookups.push(address);
      if (address.equals(canonical)) {
        return options.canonicalExists === false
          ? null
          : {
              data: canonicalAccountData(
                wallet.publicKey,
                canonicalTokenAmount,
                options.canonicalData
              ),
              executable: false,
              lamports:
                options.canonicalAccountLamports ??
                Number(canonicalTokenAmount + canonicalNativeReserve),
              owner: options.invalidCanonicalAccount
                ? PublicKey.default
                : TOKEN_PROGRAM_ID,
              rentEpoch: 0,
            };
      }
      if (occupiedTemporaryAttempts > 0) {
        occupiedTemporaryAttempts -= 1;
        return { data: Buffer.alloc(165) };
      }
      return null;
    },
    /** Return exact configured rent in lamports. */
    getMinimumBalanceForRentExemption: async () =>
      options.rentLamports ?? 2_039_280,
    /** Return ordered fee authority, including explicit unavailability. */
    getFeeForMessage: async (message: unknown) => {
      feeLookups.push(message);
      return {
        context: { slot: 1 },
        value: feeValues.length > 1 ? feeValues.shift()! : feeValues[0]!,
      };
    },
  } as unknown as Connection;
  const context = {
    http: {
      /** Satisfy the context contract without permitting accidental network I/O. */
      baseUrl: () => "https://api.example.test",
    },
    programId: PublicKey.default,
    primaryConnection: connection,
    rpcFailoverState: new RpcFailoverState(),
    depositSource: DepositSource.Global,
    signingStrategy:
      options.hasSigningStrategy === false
        ? undefined
        : options.signingStrategy ?? {
            type: "native",
            keypair: options.signingWallet ?? wallet,
          },
    authCredentials:
      options.authenticated === false
        ? undefined
        : {
            user_id: "user",
            wallet_address: (
              options.credentialWallet ?? wallet.publicKey
            ).toBase58() as PubkeyStr,
            expires_at: options.expiresAt ?? new Date(Date.now() + 60_000),
          },
  } as unknown as ClientContext;
  return {
    positions: new Positions(context),
    rpc: new Rpc(context),
    accountLookups,
    feeLookups,
  };
}

/** True only when an instruction closes this wallet's canonical WSOL ATA. */
function closesCanonicalWsol(
  instruction: Parameters<typeof decodeCloseAccountInstruction>[0],
  wallet: PublicKey
): boolean {
  try {
    const close = decodeCloseAccountInstruction(instruction, TOKEN_PROGRAM_ID);
    return close.keys.account.pubkey.equals(
      getAssociatedTokenAddressSync(NATIVE_MINT, wallet)
    );
  } catch {
    return false;
  }
}

/** Return the smallest active market shape required by split planning. */
function market(): Market {
  return {
    pubkey: Keypair.generate().publicKey.toBase58(),
    numOutcomes: 2,
  } as Market;
}

describe("SOL action plans", () => {
  it("uses the byte-exact cross-language temporary seed", async () => {
    const wallet = new PublicKey(new Uint8Array(32).fill(1));
    assert.equal(
      nativeWithdrawSeed(
        "11111111111111111111111111111111",
        wallet,
        new PublicKey(new Uint8Array(32).fill(2)),
        0x0102_0304_0506_0708n,
        7
      ),
      "4dce744c636478f024df5aefd987f933"
    );
    assert.equal(
      (
        await PublicKey.createWithSeed(
        wallet,
        "4dce744c636478f024df5aefd987f933",
        TOKEN_PROGRAM_ID
        )
      ).toBase58(),
      "71S4MLz9scZhY8BomAjfTkVn6HhFo8yFU7G6tSLto5g6"
    );
  });

  it("inspects exact canonical account facts and preserves boolean presence", async () => {
    const wallet = Keypair.generate();
    const canonical = getAssociatedTokenAddressSync(NATIVE_MINT, wallet.publicKey);
    const present = planningHarness(wallet, {
      canonicalAccountLamports: 2_039_403,
      canonicalTokenAmount: 123n,
    });

    const info: CanonicalWsolAccountInfo | null =
      await present.rpc.canonicalWsolAccountInfo(canonical, wallet.publicKey);
    assert.deepEqual(info, {
      accountLamports: 2_039_403n,
      tokenAmountLamports: 123n,
      nativeReserveLamports: 2_039_280n,
    });
    assert.equal(
      await present.rpc.canonicalWsolAccountExists(canonical, wallet.publicKey),
      true
    );

    const missing = planningHarness(wallet, { canonicalExists: false });
    assert.equal(
      await missing.rpc.canonicalWsolAccountInfo(canonical, wallet.publicKey),
      null
    );
    assert.equal(
      await missing.rpc.canonicalWsolAccountExists(canonical, wallet.publicKey),
      false
    );
  });

  it("rejects a non-canonical address before reading RPC", async () => {
    const wallet = Keypair.generate();
    const wrongAddress = Keypair.generate().publicKey;
    const harness = planningHarness(wallet);
    await assert.rejects(
      harness.rpc.canonicalWsolAccountInfo(wrongAddress, wallet.publicKey),
      /does not match the Trading Wallet Tokenkeg ATA/
    );
    await assert.rejects(
      harness.rpc.canonicalWsolAccountExists(wrongAddress, wallet.publicKey),
      /does not match the Trading Wallet Tokenkeg ATA/
    );
    assert.equal(harness.accountLookups.length, 0);
  });

  it("accepts no close authority or the Trading Wallet and rejects any other authority", async () => {
    const wallet = Keypair.generate();
    const canonical = getAssociatedTokenAddressSync(NATIVE_MINT, wallet.publicKey);
    for (const canonicalData of [
      undefined,
      { closeAuthority: wallet.publicKey },
    ]) {
      const accepted = planningHarness(wallet, {
        canonicalAccountLamports: 2_039_403,
        canonicalTokenAmount: 123n,
        canonicalData,
      });
      assert.deepEqual(
        await accepted.rpc.canonicalWsolAccountInfo(canonical, wallet.publicKey),
        {
          accountLamports: 2_039_403n,
          tokenAmountLamports: 123n,
          nativeReserveLamports: 2_039_280n,
        }
      );
      assert.equal(
        await accepted.rpc.canonicalWsolAccountExists(
          canonical,
          wallet.publicKey
        ),
        true
      );
    }

    const wrongAuthority = planningHarness(wallet, {
      canonicalAccountLamports: 2_039_403,
      canonicalTokenAmount: 123n,
      canonicalData: { closeAuthority: Keypair.generate().publicKey },
    });
    await assert.rejects(
      wrongAuthority.rpc.canonicalWsolAccountInfo(canonical, wallet.publicKey),
      /incompatible mint, authority, or native state/
    );
    await assert.rejects(
      wrongAuthority.rpc.canonicalWsolAccountExists(canonical, wallet.publicKey),
      /incompatible mint, authority, or native state/
    );
  });

  it("rejects impossible canonical token amount and native reserve accounting", async () => {
    const wallet = Keypair.generate();
    const canonical = getAssociatedTokenAddressSync(NATIVE_MINT, wallet.publicKey);
    for (const options of [
      {
        canonicalAccountLamports: Number.MAX_SAFE_INTEGER,
        canonicalTokenAmount: 0xffff_ffff_ffff_ffffn,
        canonicalData: { nativeReserve: 1n },
      },
      {
        canonicalAccountLamports: 2_039_402,
        canonicalTokenAmount: 123n,
      },
    ]) {
      const impossible = planningHarness(wallet, options);
      await assert.rejects(
        impossible.rpc.canonicalWsolAccountInfo(canonical, wallet.publicKey),
        /token amount and native reserve exceed account lamports/
      );
    }
  });

  it("uses the existing-account floor and requires native reserve", () => {
    const available = solBalanceAvailability(
      { nativeLamports: 10_000_000n, canonicalWsolLamports: 5_000_000n },
      {
        feeLamports: 5_000n,
        upfrontRentLamports: 0n,
        createsCanonicalWsolAccount: false,
        sponsored: false,
      }
    );
    assert.equal(available.reserveLamports, 1_000_000n);
    assert.equal(available.spendableLamports, 14_000_000n);
    assert.throws(
      () =>
        solBalanceAvailability(
          { nativeLamports: 999_999n, canonicalWsolLamports: 10_000_000n },
          {
            feeLamports: 5_000n,
            upfrontRentLamports: 0n,
            createsCanonicalWsolAccount: false,
            sponsored: false,
          }
        ),
      (error: unknown) => {
        assert.ok(error instanceof SdkError);
        assert.equal(error.variant, "InsufficientSolForTransactionFees");
        assert.equal(error.availableLamports, 999_999n);
        assert.equal(error.requiredLamports, 1_000_000n);
        return true;
      }
    );
  });

  it("uses account-creation live costs and sponsored zero reserve", () => {
    const components = {
      nativeLamports: 10_000_000n,
      canonicalWsolLamports: 5_000_000n,
    };
    assert.equal(
      solBalanceAvailability(components, {
        feeLamports: 1_000_000n,
        upfrontRentLamports: 3_000_000n,
        createsCanonicalWsolAccount: true,
        sponsored: false,
      }).reserveLamports,
      4_000_000n
    );
    assert.equal(
      solBalanceAvailability(components, {
        feeLamports: 20_000_000n,
        upfrontRentLamports: 20_000_000n,
        createsCanonicalWsolAccount: true,
        sponsored: true,
      }).reserveLamports,
      0n
    );
  });

  it("rejects malformed or overflowing action costs", () => {
    const components = {
      nativeLamports: 10_000_000n,
      canonicalWsolLamports: 5_000_000n,
    };
    for (const costs of [
      { feeLamports: -1n, upfrontRentLamports: 0n },
      { feeLamports: 0x1_0000_0000_0000_0000n, upfrontRentLamports: 0n },
      {
        feeLamports: 0xffff_ffff_ffff_ffffn,
        upfrontRentLamports: 1n,
      },
    ]) {
      assert.throws(
        () =>
          solBalanceAvailability(components, {
            ...costs,
            createsCanonicalWsolAccount: false,
            sponsored: true,
          }),
        /u64/
      );
    }
    assert.throws(
      () =>
        solBalanceAvailability(
          {
            nativeLamports: 0xffff_ffff_ffff_ffffn,
            canonicalWsolLamports: 1n,
          },
          {
            feeLamports: 0n,
            upfrontRentLamports: 0n,
            createsCanonicalWsolAccount: false,
            sponsored: true,
          }
        ),
      /displayed SOL exceeds the transaction u64 range/
    );
    for (const components of [
      { nativeLamports: -1n, canonicalWsolLamports: 0n },
      {
        nativeLamports: 0n,
        canonicalWsolLamports: 0x1_0000_0000_0000_0000n,
      },
    ]) {
      assert.throws(
        () =>
          solBalanceAvailability(components, {
            feeLamports: 0n,
            upfrontRentLamports: 0n,
            createsCanonicalWsolAccount: false,
            sponsored: true,
          }),
        /non-negative u64 lamport range/
      );
    }
  });

  it("uses exact fee-only availability for unwrap-all", () => {
    const components = {
      nativeLamports: 5_000n,
      canonicalWsolLamports: 500_000_000n,
    };
    const costs = {
      feeLamports: 5_000n,
      upfrontRentLamports: 0n,
      createsCanonicalWsolAccount: false,
      sponsored: false,
    };
    assert.deepEqual(unwrapAllSolBalanceAvailability(components, costs), {
      components,
      displayedLamports: 500_005_000n,
      reserveLamports: 5_000n,
      spendableLamports: 500_000_000n,
    });
    let error: unknown;
    try {
      unwrapAllSolBalanceAvailability(components, {
        ...costs,
        feeLamports: 5_001n,
      });
    } catch (caught) {
      error = caught;
    }
    assert.ok(error instanceof SdkError);
    assert.equal(error.variant, "InsufficientSolForTransactionFees");
    assert.equal(error.availableLamports, 5_000n);
    assert.equal(error.requiredLamports, 5_001n);
    assert.throws(
      () =>
        unwrapAllSolBalanceAvailability(
          {
            nativeLamports: 0xffff_ffff_ffff_ffffn,
            canonicalWsolLamports: 1n,
          },
          { ...costs, feeLamports: 0n }
        ),
      /displayed SOL exceeds the transaction u64 range/
    );
    for (const invalidCosts of [
      { ...costs, upfrontRentLamports: 1n },
      { ...costs, createsCanonicalWsolAccount: true },
      { ...costs, sponsored: true },
    ]) {
      assert.throws(
        () => unwrapAllSolBalanceAvailability(components, invalidCosts),
        /must be unsponsored with no upfront rent or account creation/
      );
    }
    for (const invalidCosts of [
      { ...costs, feeLamports: -1n },
      { ...costs, upfrontRentLamports: 0x1_0000_0000_0000_0000n },
    ]) {
      assert.throws(
        () => unwrapAllSolBalanceAvailability(components, invalidCosts),
        /must fit the non-negative u64 lamport range/
      );
    }
    assert.throws(
      () =>
        unwrapAllSolBalanceAvailability(
          { nativeLamports: -1n, canonicalWsolLamports: 1n },
          costs
        ),
      /native SOL must fit the non-negative u64 lamport range/
    );
  });

  it("plans exact wrap into an existing canonical account", async () => {
    const wallet = Keypair.generate();
    const canonical = getAssociatedTokenAddressSync(NATIVE_MINT, wallet.publicKey);
    const { positions } = planningHarness(wallet, {
      canonicalTokenAmount: 250_000_000n,
    });
    const plan = await positions.planWrapSol(
      100_000_000n,
      stateFor(wallet.publicKey, "2.000000000", "0.250000000")
    );

    assert.equal(plan.kind, "wrap");
    assert.equal(plan.transaction.feePayer?.equals(wallet.publicKey), true);
    assert.equal(plan.transaction.recentBlockhash, "11111111111111111111111111111111");
    assert.equal(plan.transaction.lastValidBlockHeight, 100);
    assert.equal(plan.transaction.instructions.length, 2);
    const transfer = SystemInstruction.decodeTransfer(
      plan.transaction.instructions[0]!
    );
    assert.equal(transfer.fromPubkey.equals(wallet.publicKey), true);
    assert.equal(transfer.toPubkey.equals(canonical), true);
    assert.equal(transfer.lamports, 100_000_000n);
    const sync = decodeSyncNativeInstruction(
      plan.transaction.instructions[1]!,
      TOKEN_PROGRAM_ID
    );
    assert.equal(sync.keys.account.pubkey.equals(canonical), true);
    assert.deepEqual(plan.costs, {
      feeLamports: 5_000n,
      upfrontRentLamports: 0n,
      createsCanonicalWsolAccount: false,
      sponsored: false,
    });
    assert.equal(plan.availability.reserveLamports, 1_000_000n);
    assert.deepEqual(plan.expectedDelta, {
      nativeLamports: -100_005_000n,
      canonicalWsolLamports: 100_000_000n,
    });
  });

  it("strictly creates only the planned-missing Tokenkeg ATA before exact wrap", async () => {
    const wallet = Keypair.generate();
    const canonical = getAssociatedTokenAddressSync(NATIVE_MINT, wallet.publicKey);
    const { positions } = planningHarness(wallet, { canonicalExists: false });
    const plan = await positions.planWrapSol(
      100_000_000n,
      stateFor(wallet.publicKey, "2.000000000", "0.000000000")
    );

    assert.equal(plan.transaction.instructions.length, 3);
    const create = plan.transaction.instructions[0]!;
    assert.equal(create.programId.equals(ASSOCIATED_TOKEN_PROGRAM_ID), true);
    assert.deepEqual(create.data, Buffer.alloc(0));
    assert.equal(create.keys[0]!.pubkey.equals(wallet.publicKey), true);
    assert.equal(create.keys[0]!.isSigner, true);
    assert.equal(create.keys[1]!.pubkey.equals(canonical), true);
    assert.equal(create.keys[1]!.isWritable, true);
    assert.equal(create.keys[2]!.pubkey.equals(wallet.publicKey), true);
    assert.equal(create.keys[3]!.pubkey.equals(NATIVE_MINT), true);
    assert.equal(create.keys[4]!.pubkey.equals(SystemProgram.programId), true);
    assert.equal(create.keys[5]!.pubkey.equals(TOKEN_PROGRAM_ID), true);
    assert.deepEqual(plan.costs, {
      feeLamports: 5_000n,
      upfrontRentLamports: 2_039_280n,
      createsCanonicalWsolAccount: true,
      sponsored: false,
    });
    assert.equal(plan.availability.reserveLamports, 3_500_000n);
    assert.deepEqual(plan.expectedDelta, {
      nativeLamports: -102_044_280n,
      canonicalWsolLamports: 100_000_000n,
    });
  });

  it("uses live wrap costs above both reserve floors", async () => {
    const wallet = Keypair.generate();
    const existing = planningHarness(wallet, { feeValues: [1_250_000] });
    const existingPlan = await existing.positions.planWrapSol(
      1n,
      stateFor(wallet.publicKey, "1.300000000", "0.000000000")
    );
    assert.equal(existingPlan.availability.reserveLamports, 1_250_000n);

    const missing = planningHarness(wallet, {
      canonicalExists: false,
      feeValues: [2_000_000],
    });
    const missingPlan = await missing.positions.planWrapSol(
      1n,
      stateFor(wallet.publicKey, "1.300000000", "0.000000000")
    );
    assert.equal(missingPlan.availability.reserveLamports, 4_039_280n);
  });

  it("requires native SOL to fund wrap amount plus the applicable reserve", async () => {
    const wallet = Keypair.generate();
    for (const [options, nativeLamports] of [
      [{}, 100_999_999n],
      [{ canonicalExists: false }, 103_499_999n],
    ] as const) {
      const harness = planningHarness(wallet, options);
      await assert.rejects(
        harness.positions.planWrapSol(
          100_000_000n,
          stateFor(wallet.publicKey, solText(nativeLamports), "0.000000000")
        ),
        /cannot fund the wrap amount and transaction reserve/
      );
    }
  });

  it("fails wrap when live fee or account rent is unavailable or inexact", async () => {
    const wallet = Keypair.generate();
    const feeUnavailable = planningHarness(wallet, { feeValues: [null] });
    await assert.rejects(
      feeUnavailable.positions.planWrapSol(
        1n,
        stateFor(wallet.publicKey, "1.000000000", "0.000000000")
      ),
      /fee estimate is unavailable/
    );

    for (const rentLamports of [-1, 1.5, Number.MAX_SAFE_INTEGER + 1]) {
      const invalidRent = planningHarness(wallet, {
        canonicalExists: false,
        rentLamports,
      });
      await assert.rejects(
        invalidRent.positions.planWrapSol(
          1n,
          stateFor(wallet.publicKey, "1.000000000", "0.000000000")
        ),
        /rent-exempt minimum must be a non-negative safe integer/
      );
    }
  });

  it("rejects stale live wrap state and invalid amounts", async () => {
    const wallet = Keypair.generate();
    const stale = planningHarness(wallet, {
      canonicalTokenAmount: 200_000_000n,
    });
    await assert.rejects(
      stale.positions.planWrapSol(
        1n,
        stateFor(wallet.publicKey, "1.000000000", "0.100000000")
      ),
      /live canonical WSOL amount does not match/
    );
    const missing = planningHarness(wallet, { canonicalExists: false });
    await assert.rejects(
      missing.positions.planWrapSol(
        1n,
        stateFor(wallet.publicKey, "1.000000000", "0.100000000")
      ),
      /canonical WSOL balance is positive but its account is unavailable/
    );

    for (const amount of [0n, -1n, 0x1_0000_0000_0000_0000n]) {
      const harness = planningHarness(wallet);
      await assert.rejects(
        harness.positions.planWrapSol(
          amount,
          stateFor(wallet.publicKey, "1.000000000", "0.000000000")
        ),
        amount > 0n ? /fit u64/ : /greater than zero/
      );
      assert.equal(harness.accountLookups.length, 0);
    }
    for (const amount of [1, 1.5, "1"]) {
      const harness = planningHarness(wallet);
      await assert.rejects(
        harness.positions.planWrapSol(
          amount as unknown as bigint,
          stateFor(wallet.publicKey, "1.000000000", "0.000000000")
        ),
        /must be exact bigint lamports/
      );
      assert.equal(harness.accountLookups.length, 0);
    }
  });

  it("rejects wrap amount plus reserve u64 overflow", async () => {
    const wallet = Keypair.generate();
    const requiredOverflow = planningHarness(wallet, { canonicalExists: false });
    await assert.rejects(
      requiredOverflow.positions.planWrapSol(
        0xffff_ffff_ffff_ffffn,
        stateFor(
          wallet.publicKey,
          solText(0xffff_ffff_ffff_ffffn),
          "0.000000000"
        )
      ),
      /wrap amount and transaction reserve exceed u64 lamports/
    );
  });

  it("rejects donated lamports for wrap but credits them in unwrap-all", async () => {
    const wallet = Keypair.generate();
    const canonical = getAssociatedTokenAddressSync(NATIVE_MINT, wallet.publicKey);
    const accountLamports = 502_049_280;
    const { positions, feeLookups } = planningHarness(wallet, {
      canonicalAccountLamports: accountLamports,
      canonicalTokenAmount: 500_000_000n,
    });
    const state = stateFor(
      wallet.publicKey,
      "0.000005000",
      "0.500000000"
    );
    await assert.rejects(
      positions.planWrapSol(1n, state),
      /canonical WSOL account has unsynchronized native lamports/
    );
    assert.equal(feeLookups.length, 0);

    const plan = await positions.planUnwrapWsolAll(state);
    assert.equal(feeLookups.length, 1);

    assert.equal(plan.kind, "unwrapAll");
    assert.equal(plan.transaction.feePayer?.equals(wallet.publicKey), true);
    assert.equal(plan.transaction.recentBlockhash, "11111111111111111111111111111111");
    assert.equal(plan.transaction.lastValidBlockHeight, 100);
    assert.equal(plan.transaction.instructions.length, 1);
    const close = decodeCloseAccountInstruction(
      plan.transaction.instructions[0]!,
      TOKEN_PROGRAM_ID
    );
    assert.equal(close.keys.account.pubkey.equals(canonical), true);
    assert.equal(close.keys.destination.pubkey.equals(wallet.publicKey), true);
    assert.equal(close.keys.authority.pubkey.equals(wallet.publicKey), true);
    assert.equal(close.keys.authority.isSigner, true);
    assert.deepEqual(plan.costs, {
      feeLamports: 5_000n,
      upfrontRentLamports: 0n,
      createsCanonicalWsolAccount: false,
      sponsored: false,
    });
    assert.deepEqual(plan.availability, {
      components: {
        nativeLamports: 5_000n,
        canonicalWsolLamports: 500_000_000n,
      },
      displayedLamports: 500_005_000n,
      reserveLamports: 5_000n,
      spendableLamports: 500_000_000n,
    });
    // The account includes standard rent plus a 10,000-lamport direct donation;
    // close returns every account lamport rather than only the decoded token amount.
    assert.deepEqual(plan.expectedDelta, {
      nativeLamports: BigInt(accountLamports) - 5_000n,
      canonicalWsolLamports: -500_000_000n,
    });
  });

  it("rejects existing canonical u64 overflow before fee preparation", async () => {
    const wallet = Keypair.generate();
    const tokenLamports = 9_000_000_000_000_000n;
    const harness = planningHarness(wallet, {
      canonicalAccountLamports: Number(tokenLamports + 2_039_280n),
      canonicalTokenAmount: tokenLamports,
    });

    await assert.rejects(
      harness.positions.planWrapSol(
        0xffff_ffff_ffff_ffffn - tokenLamports + 1n,
        stateFor(
          wallet.publicKey,
          solText(0xffff_ffff_ffff_ffffn),
          solText(tokenLamports)
        )
      ),
      /canonical WSOL token or account u64 range/
    );
    assert.equal(harness.feeLookups.length, 0);
  });

  it("retains final native u64 overflow protection for unwrap-all", async () => {
    const wallet = Keypair.generate();
    const harness = planningHarness(wallet, { canonicalTokenAmount: 1n });
    await assert.rejects(
      harness.positions.planUnwrapWsolAll(
        stateFor(
          wallet.publicKey,
          solText(0xffff_ffff_ffff_fffen),
          "0.000000001"
        )
      ),
      /unwrap-all projected native SOL exceeds the transaction u64 range/
    );
    assert.equal(harness.feeLookups.length, 1);
  });

  it("rejects zero or absent authoritative canonical balance before RPC", async () => {
    const wallet = Keypair.generate();
    for (const wrapped of ["0.000000000", undefined]) {
      const harness = planningHarness(wallet);
      await assert.rejects(
        harness.positions.planUnwrapWsolAll(
          stateFor(wallet.publicKey, "1.000000000", wrapped)
        ),
        /requires a positive canonical WSOL balance/
      );
      assert.equal(harness.accountLookups.length, 0);
    }
  });

  it("rejects missing or mismatched live canonical unwrap authority", async () => {
    const wallet = Keypair.generate();
    const state = stateFor(
      wallet.publicKey,
      "1.000000000",
      "0.500000000"
    );
    const missing = planningHarness(wallet, { canonicalExists: false });
    await assert.rejects(
      missing.positions.planUnwrapWsolAll(state),
      /canonical WSOL account is required for unwrap-all/
    );

    for (const canonicalTokenAmount of [0n, 499_999_999n, 500_000_001n]) {
      const mismatch = planningHarness(wallet, { canonicalTokenAmount });
      await assert.rejects(
        mismatch.positions.planUnwrapWsolAll(state),
        /live canonical WSOL amount does not match/
      );
    }
  });

  it("rejects incompatible canonical account layout and state", async () => {
    const wallet = Keypair.generate();
    const state = stateFor(
      wallet.publicKey,
      "1.000000000",
      "0.000000001"
    );
    const cases: Array<{
      options: PlanningHarnessOptions;
      message: RegExp;
    }> = [
      {
        options: { invalidCanonicalAccount: true, canonicalTokenAmount: 1n },
        message: /not a legacy Token Program account/,
      },
      {
        options: {
          canonicalTokenAmount: 1n,
          canonicalData: { delegateOption: 2 },
        },
        message: /incompatible state or option tags/,
      },
      {
        options: {
          canonicalTokenAmount: 1n,
          canonicalData: { dataLength: 164 },
        },
        message: /not a legacy Token Program account/,
      },
      {
        options: {
          canonicalTokenAmount: 1n,
          canonicalData: { mint: Keypair.generate().publicKey },
        },
        message: /incompatible mint, authority, or native state/,
      },
      {
        options: {
          canonicalTokenAmount: 1n,
          canonicalData: { authority: Keypair.generate().publicKey },
        },
        message: /incompatible mint, authority, or native state/,
      },
      {
        options: { canonicalTokenAmount: 1n, canonicalData: { state: 0 } },
        message: /incompatible state or option tags/,
      },
      {
        options: { canonicalTokenAmount: 1n, canonicalData: { state: 2 } },
        message: /incompatible state or option tags/,
      },
      {
        options: { canonicalTokenAmount: 1n, canonicalData: { state: 3 } },
        message: /incompatible state or option tags/,
      },
      {
        options: {
          canonicalTokenAmount: 1n,
          canonicalData: { isNative: false },
        },
        message: /incompatible state or option tags/,
      },
      {
        options: {
          canonicalTokenAmount: 1n,
          canonicalData: { isNativeOption: 2 },
        },
        message: /incompatible state or option tags/,
      },
      {
        options: {
          canonicalTokenAmount: 1n,
          canonicalData: {
            closeAuthority: wallet.publicKey,
            closeAuthorityOption: 2,
          },
        },
        message: /incompatible state or option tags/,
      },
    ];
    for (const { options, message } of cases) {
      await assert.rejects(
        planningHarness(wallet, options).positions.planUnwrapWsolAll(state),
        message
      );
    }
  });

  it("fails unwrap-all on unavailable, insufficient, or inexact live costs", async () => {
    const wallet = Keypair.generate();
    const base = {
      canonicalTokenAmount: 1n,
      canonicalAccountLamports: 2_039_281,
    };
    const unavailable = planningHarness(wallet, {
      ...base,
      feeValues: [null],
    });
    await assert.rejects(
      unavailable.positions.planUnwrapWsolAll(
        stateFor(wallet.publicKey, "1.000000000", "0.000000001")
      ),
      /fee estimate is unavailable/
    );

    const insufficient = planningHarness(wallet, {
      ...base,
      feeValues: [5_000],
    });
    await assert.rejects(
      insufficient.positions.planUnwrapWsolAll(
        stateFor(wallet.publicKey, "0.000004999", "0.000000001")
      ),
      (error: unknown) => {
        assert.ok(error instanceof SdkError);
        assert.equal(error.variant, "InsufficientSolForTransactionFees");
        assert.equal(error.availableLamports, 4_999n);
        assert.equal(error.requiredLamports, 5_000n);
        return true;
      }
    );

    for (const fee of [-1, 1.5, Number.MAX_SAFE_INTEGER + 1]) {
      const inexact = planningHarness(wallet, {
        ...base,
        feeValues: [fee],
      });
      await assert.rejects(
        inexact.positions.planUnwrapWsolAll(
          stateFor(wallet.publicKey, "1.000000000", "0.000000001")
        ),
        /non-negative safe integer/
      );
    }

    for (const canonicalAccountLamports of [
      -1,
      1.5,
      Number.MAX_SAFE_INTEGER + 1,
    ]) {
      const invalidAccountLamports = planningHarness(wallet, {
        ...base,
        canonicalAccountLamports,
      });
      await assert.rejects(
        invalidAccountLamports.positions.planUnwrapWsolAll(
          stateFor(wallet.publicKey, "1.000000000", "0.000000001")
        ),
        /canonical WSOL account lamports must be a non-negative safe integer/
      );
    }
  });

  it("plans a split from canonical WSOL without standalone wrapping", async () => {
    const wallet = Keypair.generate();
    const { positions } = planningHarness(wallet);
    const plan = await positions.planSolSplit(
      market(),
      500_000_000n,
      stateFor(wallet.publicKey, "1.000000000", "1.000000000"),
      false
    );

    assert.equal(plan.kind, "split");
    assert.equal(plan.transaction.instructions.length, 1);
    assert.equal(plan.expectedDelta.nativeLamports, -5_000n);
    assert.equal(plan.expectedDelta.canonicalWsolLamports, -500_000_000n);
  });

  it("wraps only a split shortfall before the market instruction", async () => {
    const wallet = Keypair.generate();
    const { positions } = planningHarness(wallet);
    const plan = await positions.planSolSplit(
      market(),
      500_000_000n,
      stateFor(wallet.publicKey, "1.500000000", "0.200000000"),
      false
    );

    assert.equal(plan.transaction.instructions.length, 3);
    const transfer = SystemInstruction.decodeTransfer(
      plan.transaction.instructions[0]!
    );
    assert.equal(transfer.lamports, 300_000_000n);
    assert.equal(
      decodeSyncNativeInstruction(plan.transaction.instructions[1]!)
        .keys.account.pubkey.toBase58(),
      getAssociatedTokenAddressSync(NATIVE_MINT, wallet.publicKey).toBase58()
    );
    assert.equal(plan.expectedDelta.nativeLamports, -300_005_000n);
    assert.equal(plan.expectedDelta.canonicalWsolLamports, -200_000_000n);
  });

  it("creates a missing canonical account inside a split and reserves its rent", async () => {
    const wallet = Keypair.generate();
    const { positions } = planningHarness(wallet, { canonicalExists: false });
    const plan = await positions.planSolSplit(
      market(),
      500_000_000n,
      stateFor(wallet.publicKey, "1.000000000", "0.000000000"),
      false
    );

    assert.equal(plan.transaction.instructions.length, 4);
    assert.equal(
      plan.transaction.instructions[0]!.programId.equals(
        ASSOCIATED_TOKEN_PROGRAM_ID
      ),
      true
    );
    assert.equal(plan.availability.reserveLamports, 3_500_000n);
    assert.equal(plan.expectedDelta.nativeLamports, -502_044_280n);
    assert.equal(plan.expectedDelta.canonicalWsolLamports, 0n);
  });

  it("keeps merge and redeem proceeds in the canonical account", async () => {
    const wallet = Keypair.generate();
    for (const action of ["merge", "redeem"] as const) {
      const { positions } = planningHarness(wallet, { canonicalExists: false });
      const state = stateFor(wallet.publicKey, "1.000000000", "0.000000000");
      const plan =
        action === "merge"
          ? await positions.planSolMerge(market(), 250_000_000n, state, false)
          : await positions.planSolRedeem(
              Keypair.generate().publicKey,
              250_000_000n,
              0,
              2,
              state,
              false
            );

      assert.equal(plan.transaction.instructions.length, 2);
      assert.equal(
        plan.transaction.instructions.some((instruction) => {
          try {
            decodeCloseAccountInstruction(instruction, TOKEN_PROGRAM_ID);
            return true;
          } catch {
            return false;
          }
        }),
        false
      );
      assert.equal(plan.expectedDelta.nativeLamports, -2_044_280n);
      assert.equal(plan.expectedDelta.canonicalWsolLamports, 250_000_000n);
    }
  });

  it("plans direct native withdrawal when native funds amount plus reserve", async () => {
    const wallet = Keypair.generate();
    const recipient = Keypair.generate().publicKey;
    const { positions } = planningHarness(wallet);
    const plan = await positions.planNativeSolWithdrawal(
      recipient,
      500_000_000n,
      stateFor(wallet.publicKey, "1.000000000", "1.000000000"),
      false
    );

    assert.equal(plan.kind, "nativeWithdraw");
    assert.equal(plan.transaction.instructions.length, 1);
    assert.equal(plan.expectedDelta.nativeLamports, -500_005_000n);
    assert.equal(plan.expectedDelta.canonicalWsolLamports, 0n);
  });

  it("converts only the shortfall through a temporary account", async () => {
    const wallet = Keypair.generate();
    const recipient = Keypair.generate().publicKey;
    const directBlockhash = Keypair.generate().publicKey.toBase58();
    const plannedBlockhash = Keypair.generate().publicKey.toBase58();
    const replacementBlockhash = Keypair.generate().publicKey.toBase58();
    const { positions } = planningHarness(wallet, {
      blockhashValues: [
        directBlockhash,
        plannedBlockhash,
        replacementBlockhash,
      ],
    });
    const plan = await positions.planNativeSolWithdrawal(
      recipient,
      500_000_000n,
      stateFor(wallet.publicKey, "0.010000000", "1.000000000"),
      false
    );

    assert.equal(plan.transaction.instructions.length, 5);
    assert.equal(plan.availability.reserveLamports, 2_044_280n);
    assert.equal(plan.expectedDelta.canonicalWsolLamports, -492_044_280n);
    const create = SystemInstruction.decodeCreateWithSeed(
      plan.transaction.instructions[0]!
    );
    assert.equal(plan.transaction.recentBlockhash, plannedBlockhash);
    const expectedSeed = nativeWithdrawSeed(
      plan.transaction.recentBlockhash!,
      wallet.publicKey,
      recipient,
      500_000_000n,
      0
    );
    const expectedTemporary = await PublicKey.createWithSeed(
      wallet.publicKey,
      expectedSeed,
      TOKEN_PROGRAM_ID
    );
    assert.equal(create.newAccountPubkey.equals(expectedTemporary), true);
    assert.equal(create.basePubkey.equals(wallet.publicKey), true);
    assert.equal(create.programId.equals(TOKEN_PROGRAM_ID), true);
    assert.equal(create.lamports, 2_039_280);
    assert.equal(
      decodeTransferInstruction(plan.transaction.instructions[2]!)
        .data.amount,
      492_044_280n
    );
    const canonical = getAssociatedTokenAddressSync(NATIVE_MINT, wallet.publicKey);
    const close = decodeCloseAccountInstruction(
      plan.transaction.instructions[3]!,
      TOKEN_PROGRAM_ID
    );
    assert.equal(close.keys.account.pubkey.equals(canonical), false);
    assert.equal(close.keys.destination.pubkey.equals(wallet.publicKey), true);
    const nativeTransfer = SystemInstruction.decodeTransfer(
      plan.transaction.instructions[4]!
    );
    assert.equal(nativeTransfer.toPubkey.equals(recipient), true);
    assert.equal(nativeTransfer.lamports, 500_000_000n);
  });

  it("checks bounded temporary seeds and fails after eight collisions", async () => {
    const wallet = Keypair.generate();
    const { positions, accountLookups } = planningHarness(wallet, {
      occupiedTemporaryAttempts: 8,
    });

    await assert.rejects(
      positions.planNativeSolWithdrawal(
        Keypair.generate().publicKey,
        500_000_000n,
        stateFor(wallet.publicKey, "0.010000000", "1.000000000"),
        false
      ),
      /seed attempts are exhausted/
    );
    assert.equal(accountLookups.length, 9);
  });

  it("fails closed on unavailable fees and stale wallet identity", async () => {
    const wallet = Keypair.generate();
    const feeUnavailable = planningHarness(wallet, { feeValues: [null] });
    await assert.rejects(
      feeUnavailable.positions.planNativeSolWithdrawal(
        Keypair.generate().publicKey,
        1n,
        stateFor(wallet.publicKey, "1.000000000", "0.000000000"),
        false
      ),
      /fee estimate is unavailable/
    );

    const mismatched = planningHarness(wallet, {
      credentialWallet: Keypair.generate().publicKey,
    });
    await assert.rejects(
      mismatched.positions.planNativeSolWithdrawal(
        Keypair.generate().publicKey,
        1n,
        stateFor(wallet.publicKey, "1.000000000", "0.000000000"),
        false
      ),
      /does not match wallet balance state/
    );
    assert.equal(mismatched.accountLookups.length, 0);

    const wrongSigner = planningHarness(wallet, {
      signingWallet: Keypair.generate(),
    });
    await assert.rejects(
      wrongSigner.positions.planNativeSolWithdrawal(
        Keypair.generate().publicKey,
        1n,
        stateFor(wallet.publicKey, "1.000000000", "0.000000000"),
        false
      ),
      /signing strategy does not control authenticated wallet/
    );
    assert.equal(wrongSigner.accountLookups.length, 0);
  });

  it("admits conversion only for complete native-keypair wallet authority", async () => {
    const wallet = Keypair.generate();
    const state = stateFor(
      wallet.publicKey,
      "1.000000000",
      "0.000000001"
    );
    const externalSigner: SigningStrategy = {
      type: "walletAdapter",
      signer: {
        walletAddress: wallet.publicKey.toBase58(),
        signMessage: async () => new Uint8Array(),
        signTransaction: async () => new Uint8Array(),
      },
    };
    const privy: SigningStrategy = {
      type: "privy",
      walletId: "wallet-id",
      walletAddress: wallet.publicKey.toBase58(),
    };
    for (const signingStrategy of [externalSigner, privy]) {
      const harness = planningHarness(wallet, { signingStrategy });
      await assert.rejects(
        harness.positions.planWrapSol(1n, state),
        /requires a native signing strategy/
      );
      await assert.rejects(
        harness.positions.planUnwrapWsolAll(state),
        /requires a native signing strategy/
      );
      assert.equal(harness.accountLookups.length, 0);
    }

    const authorityCases: Array<{
      options: PlanningHarnessOptions;
      message: RegExp;
    }> = [
      {
        options: { authenticated: false },
        message: /authenticated credentials are required/,
      },
      {
        options: { hasSigningStrategy: false },
        message: /Signing strategy not configured/,
      },
      {
        options: { expiresAt: new Date(Date.now() - 1) },
        message: /authenticated credentials have expired/,
      },
      {
        options: { credentialWallet: Keypair.generate().publicKey },
        message: /authenticated wallet does not match wallet balance state/,
      },
      {
        options: { signingWallet: Keypair.generate() },
        message: /signing strategy does not control authenticated wallet/,
      },
    ];
    for (const { options, message } of authorityCases) {
      const harness = planningHarness(wallet, options);
      await assert.rejects(harness.positions.planWrapSol(1n, state), message);
      await assert.rejects(
        harness.positions.planUnwrapWsolAll(state),
        message
      );
      assert.equal(harness.accountLookups.length, 0);
    }

    const incomplete = planningHarness(wallet);
    const incompleteState = new WalletDepositBalancesState();
    await assert.rejects(
      incomplete.positions.planWrapSol(
        1n,
        incompleteState
      ),
      /wallet balance state is not initialized/
    );
    await assert.rejects(
      incomplete.positions.planUnwrapWsolAll(incompleteState),
      /wallet balance state is not initialized/
    );
    assert.equal(incomplete.accountLookups.length, 0);
  });

  it("fails closed on inexact or negative RPC lamport values", async () => {
    const wallet = Keypair.generate();
    for (const fee of [-1, 1.5, Number.MAX_SAFE_INTEGER + 1]) {
      const harness = planningHarness(wallet, { feeValues: [fee] });
      await assert.rejects(
        harness.positions.planNativeSolWithdrawal(
          Keypair.generate().publicKey,
          1n,
          stateFor(wallet.publicKey, "1.000000000", "0.000000000"),
          false
        ),
        /non-negative safe integer/
      );
    }

    for (const rentLamports of [-1, 1.5, Number.MAX_SAFE_INTEGER + 1]) {
      const harness = planningHarness(wallet, {
        canonicalExists: false,
        rentLamports,
      });
      await assert.rejects(
        harness.positions.planSolSplit(
          market(),
          1n,
          stateFor(wallet.publicKey, "1.000000000", "0.000000000"),
          false
        ),
        /non-negative safe integer/
      );
    }
  });

  it("rejects fee changes that would make a temporary transfer negative", async () => {
    const wallet = Keypair.generate();
    const state = stateFor(wallet.publicKey, "0.110000000", "0.500000000");

    for (const feeValues of [
      [20_000_000, 0],
      [20_000_000, 15_000_000, 0],
    ]) {
      const harness = planningHarness(wallet, { feeValues });
      await assert.rejects(
        harness.positions.planNativeSolWithdrawal(
          Keypair.generate().publicKey,
          100_000_000n,
          state,
          false
        ),
        /invalid temporary withdrawal requirement/
      );
    }
  });

  it("rejects SOL action amounts outside u64 before RPC", async () => {
    const wallet = Keypair.generate();
    const harness = planningHarness(wallet);
    const state = stateFor(wallet.publicKey, "1.000000000", "1.000000000");
    await assert.rejects(
      harness.positions.planNativeSolWithdrawal(
        Keypair.generate().publicKey,
        0x1_0000_0000_0000_0000n,
        state,
        false
      ),
      /fit u64/
    );
    assert.equal(harness.accountLookups.length, 0);
  });

  it("rejects ordinary sponsorship and invalid redeem outcomes before RPC", async () => {
    const wallet = Keypair.generate();
    const harness = planningHarness(wallet);
    const state = stateFor(wallet.publicKey, "2.000000000", "0.500000000");

    await assert.rejects(
      harness.positions.planSolSplit(market(), 1n, state, true),
      /sponsored SOL action planning is not supported/
    );
    await assert.rejects(
      harness.positions.planSolRedeem(
        Keypair.generate().publicKey,
        1n,
        2,
        2,
        state,
        false
      ),
      /outcome index/i
    );
    assert.equal(harness.accountLookups.length, 0);
  });

  it("rejects an occupied invalid canonical account", async () => {
    const wallet = Keypair.generate();
    const harness = planningHarness(wallet, { invalidCanonicalAccount: true });

    await assert.rejects(
      harness.positions.planSolSplit(
        market(),
        1n,
        stateFor(wallet.publicKey, "1.000000000", "1.000000000"),
        false
      ),
      /canonical WSOL account is not a legacy Token Program account/
    );
  });

  it("never closes canonical WSOL from an ordinary planner", async () => {
    const wallet = Keypair.generate();
    const recipient = Keypair.generate().publicKey;
    const ordinaryPlans = [
      await planningHarness(wallet).positions.planSolSplit(
        market(),
        1n,
        stateFor(wallet.publicKey, "1.000000000", "0.000000000"),
        false
      ),
      await planningHarness(wallet).positions.planSolMerge(
        market(),
        1n,
        stateFor(wallet.publicKey, "1.000000000", "0.000000000"),
        false
      ),
      await planningHarness(wallet).positions.planSolRedeem(
        Keypair.generate().publicKey,
        1n,
        0,
        2,
        stateFor(wallet.publicKey, "1.000000000", "0.000000000"),
        false
      ),
      await planningHarness(wallet).positions.planNativeSolWithdrawal(
        recipient,
        1n,
        stateFor(wallet.publicKey, "1.000000000", "0.000000000"),
        false
      ),
      await planningHarness(wallet).positions.planNativeSolWithdrawal(
        recipient,
        500_000_000n,
        stateFor(wallet.publicKey, "0.010000000", "1.000000000"),
        false
      ),
    ];

    for (const plan of ordinaryPlans) {
      assert.equal(
        plan.transaction.instructions.some((instruction) =>
          closesCanonicalWsol(instruction, wallet.publicKey)
        ),
        false,
        `${plan.kind} must preserve canonical WSOL`
      );
    }
  });
});
