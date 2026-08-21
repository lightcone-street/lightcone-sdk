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
  type Connection,
} from "@solana/web3.js";

import type { ClientContext } from "../src/context";
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
import { RpcFailoverState } from "../src/rpcFailover";

/** Build complete wallet authority with exact native and canonical components. */
function stateFor(
  wallet: PublicKey,
  native: string,
  wrapped: string
): WalletDepositBalancesState {
  const state = new WalletDepositBalancesState();
  const balance: DepositTokenBalance = {
    mint: WRAPPED_SOL_MINT,
    idle: wrapped,
    symbol: "WSOL",
    name: "Wrapped SOL",
  };
  state.applyRestSnapshot(wallet.toBase58() as PubkeyStr, {
    context_slot: 1,
    native_sol_balance: native,
    balances: { [WRAPPED_SOL_MINT]: balance } as Record<
      PubkeyStr,
      DepositTokenBalance
    >,
  });
  return state;
}

/** Deterministic chain authority exposed to one planner test. */
interface PlanningHarnessOptions {
  /** Whether the persistent canonical Tokenkeg ATA exists. */
  canonicalExists?: boolean;
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
  /** Ordered blockhashes used to prove final seed/message binding. */
  blockhashValues?: string[];
}

/** Build a planner with deterministic RPC responses and account-read recording. */
function planningHarness(
  wallet: Keypair,
  options: PlanningHarnessOptions = {}
): { positions: Positions; accountLookups: PublicKey[] } {
  const canonical = getAssociatedTokenAddressSync(NATIVE_MINT, wallet.publicKey);
  const accountLookups: PublicKey[] = [];
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
          : { data: Buffer.alloc(165) };
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
    getFeeForMessage: async () => ({
      context: { slot: 1 },
      value: feeValues.length > 1 ? feeValues.shift()! : feeValues[0]!,
    }),
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
    authCredentials: {
      user_id: "user",
      wallet_address: (
        options.credentialWallet ?? wallet.publicKey
      ).toBase58() as PubkeyStr,
      expires_at: options.expiresAt ?? new Date(Date.now() + 60_000),
    },
  } as unknown as ClientContext;
  return { positions: new Positions(context), accountLookups };
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
    assert.throws(() =>
      solBalanceAvailability(
        { nativeLamports: 999_999n, canonicalWsolLamports: 10_000_000n },
        {
          feeLamports: 5_000n,
          upfrontRentLamports: 0n,
          createsCanonicalWsolAccount: false,
          sponsored: false,
        }
      )
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
});
