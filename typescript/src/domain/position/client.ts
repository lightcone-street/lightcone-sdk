import {
  PublicKey,
  SystemProgram,
  Transaction,
  type TransactionInstruction,
} from "@solana/web3.js";
import {
  createAssociatedTokenAccountInstruction,
  createAssociatedTokenAccountIdempotentInstruction,
  createInitializeAccount3Instruction,
  createTransferInstruction,
  createCloseAccountInstruction,
  createSyncNativeInstruction,
  getAssociatedTokenAddressSync,
  NATIVE_MINT,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { sha256 } from "@noble/hashes/sha256";
import bs58 from "bs58";
import { isAuthenticated } from "../../auth";
import type { ClientContext } from "../../context";
import {
  requireConnection,
  requireSigningStrategy,
} from "../../context";
import { SdkError } from "../../error";
import { RetryPolicy } from "../../http";
import {
  buildDepositIx,
  buildMergeIx,
  buildRedeemWinningsIx,
  buildWithdrawConditionalFromPositionIx,
  buildInitPositionTokensIx,
  buildExtendPositionTokensIx,
  buildDepositToGlobalIx,
  buildDepositToGlobalIxWithAlt,
  buildGlobalToMarketDepositIx,
  buildWithdrawFromGlobalIx,
  buildClosePositionAltIx,
  buildClosePositionTokenAccountsIx,
} from "../../program/instructions";
import { Rpc } from "../../rpc";
import { getPositionPda } from "../../program/pda";
import { deserializePosition as deserializeProgramPosition } from "../../program/accounts";
import { validateOutcomeIndex, validateOutcomes } from "../../program/utils";
import {
  requireNativeSigningStrategy,
  signingStrategyWalletAddress,
} from "../../shared/signing";
import type {
  Position as ProgramPosition,
  RedeemWinningsParams,
  WithdrawConditionalFromPositionParams,
  WithdrawFromPositionParams,
  InitPositionTokensParams,
  ExtendPositionTokensParams,
  DepositToGlobalParams,
  DepositToGlobalAltContext,
  GlobalToMarketDepositParams,
  WithdrawFromGlobalParams,
  ClosePositionAltParams,
  ClosePositionTokenAccountsParams,
} from "../../program/types";
import type { Market } from "../market";
import type { DepositTokenBalancesSnapshot } from "./index";
import {
  solBalanceAvailability,
  unwrapAllSolBalanceAvailability,
  type SolActionCosts,
  type SolBalanceAvailability,
  type SolBalanceBreakdown,
  type WalletDepositBalancesState,
} from "./state";
import type { MarketPositionsResponse, PositionsResponse } from "./wire";
import {
  DepositBuilder,
  MergeBuilder,
  WithdrawBuilder,
  RedeemWinningsBuilder,
  WithdrawFromPositionBuilder,
  InitPositionTokensBuilder,
  ExtendPositionTokensBuilder,
  DepositToGlobalBuilder,
  WithdrawFromGlobalBuilder,
  GlobalToMarketDepositBuilder,
} from "./builders";

/** Byte allocation for a legacy SPL Token Program (Tokenkeg) account. */
const TOKEN_ACCOUNT_SPACE = 165;
/** Largest exact lamport amount accepted by Solana transaction instructions. */
const MAX_U64 = 0xffff_ffff_ffff_ffffn;

/**
 * Identifies the SOL-aware operation represented by an action plan.
 *
 * `wrap` and `unwrapAll` identify explicit native-keypair conversion plans.
 * Ordinary plan kinds do not call those planners or include canonical-account
 * closure.
 */
export type SolActionKind =
  | "split"
  | "merge"
  | "redeem"
  | "nativeWithdraw"
  | "wrap"
  | "unwrapAll";

/** Expected changes to the separately authoritative SOL balances. */
export interface SolBalanceDelta {
  /** System-account change in lamports, including unsponsored costs. */
  nativeLamports: bigint;
  /** Persistent canonical WSOL ATA change in lamports. */
  canonicalWsolLamports: bigint;
}

/** Unsigned prepared transaction plus the exact preflight facts authorizing it. */
export interface SolActionPlan {
  /** Operation whose balance semantics produced this plan. */
  kind: SolActionKind;
  /** Fee-prepared message that submission must preserve exactly. */
  transaction: Transaction;
  /** Live fee/rent observations and explicit sponsorship capability. */
  costs: SolActionCosts;
  /** Balance totals after action-specific native reserve. */
  availability: SolBalanceAvailability;
  /** Balance projection that does not replace authoritative state. */
  expectedDelta: SolBalanceDelta;
}

/** Reject non-bigint, non-positive, or non-u64 lamports before any RPC side effect. */
function assertSolActionAmount(amountLamports: bigint, action: string): void {
  if (typeof amountLamports !== "bigint") {
    throw SdkError.validation(`${action} amount must be exact bigint lamports`);
  }
  if (amountLamports <= 0n) {
    throw SdkError.validation(`${action} amount must be greater than zero`);
  }
  if (amountLamports > MAX_U64) {
    throw SdkError.validation(`${action} amount must fit u64`);
  }
}

function assertUnsponsoredPlan(sponsored: boolean): void {
  if (sponsored) {
    throw SdkError.validation("sponsored SOL action planning is not supported");
  }
}

/**
 * Derive the cross-SDK temporary-account seed for native withdrawal.
 *
 * SHA-256 receives the ASCII domain `lightcone:wsol-withdraw:v1`, one zero byte,
 * raw 32-byte blockhash, wallet, and recipient keys, the amount as unsigned
 * eight-byte big-endian lamports, then the one-byte attempt. The first 16 digest
 * bytes become 32 lowercase hexadecimal ASCII characters for Solana's seed limit.
 */
export function nativeWithdrawSeed(
  recentBlockhash: string,
  wallet: PublicKey,
  recipient: PublicKey,
  amountLamports: bigint,
  attempt: number
): string {
  if (!Number.isInteger(attempt) || attempt < 0 || attempt > 255) {
    throw SdkError.validation("temporary WSOL seed attempt must fit u8");
  }
  if (amountLamports < 0n || amountLamports > MAX_U64) {
    throw SdkError.validation("withdraw amount must fit u64");
  }
  const domain = new TextEncoder().encode("lightcone:wsol-withdraw:v1");
  const blockhash = bs58.decode(recentBlockhash);
  if (blockhash.length !== 32) {
    throw SdkError.validation("recent blockhash must decode to 32 bytes");
  }
  const preimage = new Uint8Array(domain.length + 1 + 32 + 32 + 32 + 8 + 1);
  let offset = 0;
  preimage.set(domain, offset);
  offset += domain.length;
  preimage[offset++] = 0;
  preimage.set(blockhash, offset);
  offset += 32;
  preimage.set(wallet.toBytes(), offset);
  offset += 32;
  preimage.set(recipient.toBytes(), offset);
  offset += 32;
  new DataView(preimage.buffer).setBigUint64(offset, amountLamports, false);
  offset += 8;
  preimage[offset] = attempt;
  return Array.from(sha256(preimage).slice(0, 16), (byte) =>
    byte.toString(16).padStart(2, "0")
  ).join("");
}

/**
 * Plans position operations and explicit canonical WSOL conversions.
 *
 * Explicit conversion state flows as follows: complete matching wallet state and
 * native keypair -> live account and cost reads -> signer, account, reserve, and
 * amount guards -> fee-prepared plan -> unchanged prepared submission -> complete
 * snapshot covering the confirmed slot. An uncertain submission returns control
 * to the caller, which refreshes authoritative state before planning again.
 */
export class Positions {
  constructor(private readonly client: ClientContext) {}

  // ── PDA helpers ──────────────────────────────────────────────────────

  pda(owner: PublicKey, market: PublicKey): PublicKey {
    return getPositionPda(owner, market, this.client.programId)[0];
  }

  // ── HTTP methods ─────────────────────────────────────────────────────

  async get(userPubkey: string): Promise<PositionsResponse> {
    const url = `${this.client.http.baseUrl()}/api/users/${encodeURIComponent(userPubkey)}/positions`;
    return this.client.http.get<PositionsResponse>(url, RetryPolicy.Idempotent);
  }

  async getForMarket(userPubkey: string, marketPubkey: string): Promise<MarketPositionsResponse> {
    const url = `${this.client.http.baseUrl()}/api/users/${encodeURIComponent(userPubkey)}/markets/${encodeURIComponent(marketPubkey)}/positions`;
    return this.client.http.get<MarketPositionsResponse>(url, RetryPolicy.Idempotent);
  }

  /**
   * Get all conditional-token positions for the authenticated user across
   * every market. The wallet is resolved server-side from the auth cookie,
   * so no parameter is required. Same response shape as `get()`.
   *
   * `GET /api/users/positions`
   */
  async positions(): Promise<PositionsResponse> {
    const url = `${this.client.http.baseUrl()}/api/users/positions`;
    return this.client.http.get<PositionsResponse>(url, RetryPolicy.Idempotent);
  }

  /**
   * Same as {@link positions}, but uses the supplied `cookieHeader` for this
   * call instead of the SDK's process-wide cookie store.
   *
   * Intended for server-side cookie forwarding (SSR / server functions)
   * where the per-request browser cookie can't propagate to the shared
   * client. In a browser context this is equivalent to {@link positions}
   * because the runtime is already attaching the cookie via
   * `credentials: "include"`.
   */
  async positionsWithCookies(cookieHeader: string): Promise<PositionsResponse> {
    const url = `${this.client.http.baseUrl()}/api/users/positions`;
    return this.client.http.getWithCookies<PositionsResponse>(
      url,
      RetryPolicy.Idempotent,
      cookieHeader,
    );
  }

  /**
   * Get the authenticated user's positions in a specific market. The wallet
   * is resolved server-side from the auth cookie.
   *
   * `GET /api/users/markets/{market_pubkey}/positions`
   */
  async positionsForMarket(marketPubkey: string): Promise<MarketPositionsResponse> {
    const url = `${this.client.http.baseUrl()}/api/users/markets/${encodeURIComponent(marketPubkey)}/positions`;
    return this.client.http.get<MarketPositionsResponse>(url, RetryPolicy.Idempotent);
  }

  /**
   * Same as {@link positionsForMarket}, but uses the supplied `cookieHeader`
   * for this call instead of the SDK's process-wide cookie store. For
   * server-side cookie forwarding (SSR / server functions).
   */
  async positionsForMarketWithCookies(
    marketPubkey: string,
    cookieHeader: string,
  ): Promise<MarketPositionsResponse> {
    const url = `${this.client.http.baseUrl()}/api/users/markets/${encodeURIComponent(marketPubkey)}/positions`;
    return this.client.http.getWithCookies<MarketPositionsResponse>(
      url,
      RetryPolicy.Idempotent,
      cookieHeader,
    );
  }

  /**
   * Fetch a complete authenticated SPL and native-SOL balance snapshot.
   *
   * `minContextSlot` lower-bounds the complete balance snapshot. Native SOL is
   * required canonical nine-decimal text and remains outside the SPL map. The
   * generic HTTP layer trusts that shape at runtime; WebSocket frames are decoded
   * strictly, while malformed REST exact values fail later when state scales them.
   */
  async depositTokenBalances(
    minContextSlot?: number
  ): Promise<DepositTokenBalancesSnapshot> {
    const query =
      minContextSlot === undefined
        ? ""
        : `?min_context_slot=${encodeURIComponent(minContextSlot)}`;
    const url = `${this.client.http.baseUrl()}/api/users/deposit-token-balances${query}`;
    return this.client.http.get<DepositTokenBalancesSnapshot>(
      url,
      RetryPolicy.Idempotent,
    );
  }

  /**
   * Same as {@link depositTokenBalances}, but uses the supplied `cookieHeader`
   * for this call instead of the SDK's process-wide cookie store.
   *
   * Intended for server-side cookie forwarding (SSR / server functions)
   * where the per-request browser cookie can't propagate to the shared
   * client. The complete response has the same separate, exact native-SOL
   * contract as {@link depositTokenBalances}. In a browser this is equivalent to
   * {@link depositTokenBalances} because the runtime is already attaching
   * the cookie via `credentials: "include"`.
   */
  async depositTokenBalancesWithCookies(
    minContextSlot: number | undefined,
    cookieHeader: string,
  ): Promise<DepositTokenBalancesSnapshot> {
    const query =
      minContextSlot === undefined
        ? ""
        : `?min_context_slot=${encodeURIComponent(minContextSlot)}`;
    const url = `${this.client.http.baseUrl()}/api/users/deposit-token-balances${query}`;
    return this.client.http.getWithCookies<DepositTokenBalancesSnapshot>(
      url,
      RetryPolicy.Idempotent,
      cookieHeader,
    );
  }

  /**
   * Return a fee-prepared plan for an exact canonical WSOL wrap.
   *
   * The authenticated Trading Wallet must have a local native keypair and complete
   * balance state. Live canonical account data must match that state. An existing
   * account must have account lamports equal to its token amount plus native
   * reserve. Otherwise a later `SyncNative` instruction would recalculate the WSOL
   * token amount from account lamports and wrap donated excess beyond
   * `amountLamports`. The returned transaction contains strict Tokenkeg ATA
   * creation only when the account is absent. It then contains the exact transfer
   * and `SyncNative`. Availability uses the ordinary reserve floor.
   *
   * Callers rebuild immediately before prepared submission. They retain the
   * returned balance projection until a complete snapshot covers the confirmed
   * slot. An uncertain outcome requires authoritative refresh before another plan.
   */
  async planWrapSol(
    amountLamports: bigint,
    state: WalletDepositBalancesState
  ): Promise<SolActionPlan> {
    assertSolActionAmount(amountLamports, "wrap");
    const wallet = this.conversionPlanningWallet(state);
    const breakdown = state.solBalanceBreakdown();
    const rpc = new Rpc(this.client);
    const canonical = getAssociatedTokenAddressSync(NATIVE_MINT, wallet);
    const account = await rpc.canonicalWsolAccountInfo(canonical, wallet);
    if (!account && breakdown.canonicalWsolLamports > 0n) {
      throw SdkError.validation(
        "canonical WSOL balance is positive but its account is unavailable"
      );
    }
    if (
      account &&
      account.tokenAmountLamports !== breakdown.canonicalWsolLamports
    ) {
      throw SdkError.validation(
        "live canonical WSOL amount does not match wallet balance state"
      );
    }
    if (
      account &&
      account.accountLamports !==
        account.tokenAmountLamports + account.nativeReserveLamports
    ) {
      throw SdkError.validation(
        "canonical WSOL account has unsynchronized native lamports"
      );
    }
    if (
      account &&
      (account.tokenAmountLamports + amountLamports > MAX_U64 ||
        account.accountLamports + amountLamports > MAX_U64)
    ) {
      throw SdkError.validation(
        "wrap would exceed canonical WSOL token or account u64 range"
      );
    }
    const createsCanonicalWsolAccount = account === null;
    const upfrontRentLamports = createsCanonicalWsolAccount
      ? await rpc.minimumBalanceForRentExemption(TOKEN_ACCOUNT_SPACE)
      : 0n;
    const transaction = this.buildWrapSolTransaction(
      wallet,
      amountLamports,
      createsCanonicalWsolAccount
    );
    const feeLamports = await rpc.prepareAndEstimateTransactionFee(transaction);
    const costs: SolActionCosts = {
      feeLamports,
      upfrontRentLamports,
      createsCanonicalWsolAccount,
      sponsored: false,
    };
    const availability = solBalanceAvailability(breakdown, costs);
    const requiredNativeLamports =
      amountLamports + availability.reserveLamports;
    if (requiredNativeLamports > MAX_U64) {
      throw SdkError.validation(
        "wrap amount and transaction reserve exceed u64 lamports"
      );
    }
    if (breakdown.nativeLamports < requiredNativeLamports) {
      throw SdkError.validation(
        "native SOL cannot fund the wrap amount and transaction reserve"
      );
    }
    return {
      kind: "wrap",
      transaction,
      costs,
      availability,
      expectedDelta: {
        nativeLamports:
          -amountLamports - feeLamports - upfrontRentLamports,
        canonicalWsolLamports: amountLamports,
      },
    };
  }

  /**
   * Return a fee-prepared plan for closing the complete canonical WSOL account.
   *
   * The Trading Wallet must have a local native keypair. Canonical WSOL in the
   * complete balance state must be positive and equal the live token amount. The
   * returned transaction contains one `CloseAccount` instruction whose authority,
   * destination, and fee payer are that wallet. If submitted successfully, the
   * instruction transfers the complete account balance, including rent and donated
   * lamports. The returned costs contain only the fresh fee. Availability requires
   * native SOL to fund that fee without relying on the later account transfer.
   *
   * Callers rebuild immediately before prepared submission. They retain the
   * returned balance projection until a complete snapshot covers the confirmed
   * slot. Signing, submission, or confirmation uncertainty requires authoritative
   * refresh and does not authorize automatic resubmission.
   */
  async planUnwrapWsolAll(
    state: WalletDepositBalancesState
  ): Promise<SolActionPlan> {
    const wallet = this.conversionPlanningWallet(state);
    const breakdown = state.solBalanceBreakdown();
    if (breakdown.canonicalWsolLamports === 0n) {
      throw SdkError.validation(
        "unwrap-all requires a positive canonical WSOL balance"
      );
    }
    const rpc = new Rpc(this.client);
    const canonical = getAssociatedTokenAddressSync(NATIVE_MINT, wallet);
    const account = await rpc.canonicalWsolAccountInfo(canonical, wallet);
    if (!account) {
      throw SdkError.validation(
        "canonical WSOL account is required for unwrap-all"
      );
    }
    if (account.tokenAmountLamports !== breakdown.canonicalWsolLamports) {
      throw SdkError.validation(
        "live canonical WSOL amount does not match wallet balance state"
      );
    }
    const transaction = this.buildUnwrapWsolAllTransaction(wallet);
    const feeLamports = await rpc.prepareAndEstimateTransactionFee(transaction);
    const costs: SolActionCosts = {
      feeLamports,
      upfrontRentLamports: 0n,
      createsCanonicalWsolAccount: false,
      sponsored: false,
    };
    // Unwrap-all removes the persistent account, so its availability validates
    // SolActionCosts and reserves the fee without the ordinary account floor.
    const availability = unwrapAllSolBalanceAvailability(
      breakdown,
      costs
    );
    const projectedNativeLamports =
      breakdown.nativeLamports + account.accountLamports - feeLamports;
    if (projectedNativeLamports > MAX_U64) {
      throw SdkError.validation(
        "unwrap-all projected native SOL exceeds the transaction u64 range"
      );
    }
    return {
      kind: "unwrapAll",
      transaction,
      costs,
      availability,
      expectedDelta: {
        nativeLamports: account.accountLamports - feeLamports,
        canonicalWsolLamports: -breakdown.canonicalWsolLamports,
      },
    };
  }

  /**
   * Plan one atomic split that consumes canonical WSOL before wrapping a shortfall.
   * Amounts and live costs are lamports; unavailable account, fee, or rent reads
   * fail closed, and sponsored planning is rejected until a sponsor owns costs.
   */
  async planSolSplit(
    market: Market,
    amountLamports: bigint,
    state: WalletDepositBalancesState,
    sponsored: boolean
  ): Promise<SolActionPlan> {
    assertUnsponsoredPlan(sponsored);
    assertSolActionAmount(amountLamports, "split");
    const wallet = this.planningWallet(state);
    const breakdown = state.solBalanceBreakdown();
    const rpc = new Rpc(this.client);
    const canonical = getAssociatedTokenAddressSync(NATIVE_MINT, wallet);
    const canonicalExists = await rpc.canonicalWsolAccountExists(canonical, wallet);
    if (breakdown.canonicalWsolLamports > 0n && !canonicalExists) {
      throw SdkError.validation(
        "canonical WSOL balance is positive but its account is unavailable"
      );
    }
    const shortfall =
      amountLamports > breakdown.canonicalWsolLamports
        ? amountLamports - breakdown.canonicalWsolLamports
        : 0n;
    const upfrontRentLamports = canonicalExists
      ? 0n
      : await rpc.minimumBalanceForRentExemption(TOKEN_ACCOUNT_SPACE);
    const transaction = new Transaction({ feePayer: wallet });
    if (!canonicalExists) {
      transaction.add(
        createAssociatedTokenAccountIdempotentInstruction(
          wallet,
          canonical,
          wallet,
          NATIVE_MINT,
          TOKEN_PROGRAM_ID
        )
      );
    }
    if (shortfall > 0n) {
      transaction.add(
        SystemProgram.transfer({ fromPubkey: wallet, toPubkey: canonical, lamports: shortfall }),
        createSyncNativeInstruction(canonical, TOKEN_PROGRAM_ID)
      );
    }
    transaction.add(
      buildDepositIx(
        {
          user: wallet,
          market: new PublicKey(market.pubkey),
          depositMint: NATIVE_MINT,
          amount: amountLamports,
        },
        market.numOutcomes,
        this.client.programId
      )
    );
    const feeLamports = await rpc.prepareAndEstimateTransactionFee(transaction);
    const costs: SolActionCosts = {
      feeLamports,
      upfrontRentLamports,
      createsCanonicalWsolAccount: !canonicalExists,
      sponsored,
    };
    const availability = solBalanceAvailability(breakdown, costs);
    if (amountLamports > availability.spendableLamports) {
      throw SdkError.validation(
        "split amount exceeds spendable SOL after transaction reserve"
      );
    }
    if (shortfall + availability.reserveLamports > breakdown.nativeLamports) {
      throw SdkError.validation(
        "native SOL cannot fund the wrap shortfall and transaction reserve"
      );
    }
    const walletCosts = sponsored ? 0n : feeLamports + upfrontRentLamports;
    return {
      kind: "split",
      transaction,
      costs,
      availability,
      expectedDelta: {
        nativeLamports: -shortfall - walletCosts,
        canonicalWsolLamports: shortfall - amountLamports,
      },
    };
  }

  /**
   * Plan a merge that leaves returned WSOL in the persistent canonical ATA.
   * The prepared transaction does not mutate cached state; refresh authority
   * after confirmed submission.
   */
  async planSolMerge(
    market: Market,
    amountLamports: bigint,
    state: WalletDepositBalancesState,
    sponsored: boolean
  ): Promise<SolActionPlan> {
    assertUnsponsoredPlan(sponsored);
    assertSolActionAmount(amountLamports, "merge");
    const wallet = this.planningWallet(state);
    const transaction = new Transaction({ feePayer: wallet });
    const { rpc, breakdown, canonicalExists, upfrontRentLamports } =
      await this.receivePlanContext(wallet, state);
    if (!canonicalExists) {
      transaction.add(this.createCanonicalWsolAccount(wallet));
    }
    transaction.add(
      buildMergeIx(
        {
          user: wallet,
          market: new PublicKey(market.pubkey),
          depositMint: NATIVE_MINT,
          amount: amountLamports,
        },
        market.numOutcomes,
        this.client.programId
      )
    );
    return this.finishReceivePlan(
      "merge",
      amountLamports,
      transaction,
      rpc,
      breakdown,
      upfrontRentLamports,
      !canonicalExists,
      sponsored
    );
  }

  /**
   * Plan a redemption that leaves returned WSOL in the persistent canonical ATA.
   * `amountLamports` is exact collateral scale; `outcomeIndex` is validated
   * against the supplied authoritative `numOutcomes`.
   */
  async planSolRedeem(
    market: PublicKey,
    amountLamports: bigint,
    outcomeIndex: number,
    numOutcomes: number,
    state: WalletDepositBalancesState,
    sponsored: boolean
  ): Promise<SolActionPlan> {
    assertUnsponsoredPlan(sponsored);
    assertSolActionAmount(amountLamports, "redeem");
    validateOutcomes(numOutcomes);
    validateOutcomeIndex(outcomeIndex, numOutcomes);
    const wallet = this.planningWallet(state);
    const transaction = new Transaction({ feePayer: wallet });
    const { rpc, breakdown, canonicalExists, upfrontRentLamports } =
      await this.receivePlanContext(wallet, state);
    if (!canonicalExists) {
      transaction.add(this.createCanonicalWsolAccount(wallet));
    }
    transaction.add(
      buildRedeemWinningsIx(
        {
          user: wallet,
          market,
          depositMint: NATIVE_MINT,
          amount: amountLamports,
        },
        outcomeIndex,
        this.client.programId
      )
    );
    return this.finishReceivePlan(
      "redeem",
      amountLamports,
      transaction,
      rpc,
      breakdown,
      upfrontRentLamports,
      !canonicalExists,
      sponsored
    );
  }

  /**
   * Plan exact native SOL delivery without closing the canonical WSOL ATA.
   *
   * Native funds are preferred. A shortfall uses a bounded seeded Tokenkeg
     * account whose rent returns on close; all account, rent, and fee reads fail
     * closed. At most eight blockhash-scoped candidates bound RPC latency while
     * making accidental exhaustion negligible. The returned transaction already
     * carries its prepared message.
   */
  async planNativeSolWithdrawal(
    recipient: PublicKey,
    amountLamports: bigint,
    state: WalletDepositBalancesState,
    sponsored: boolean
  ): Promise<SolActionPlan> {
    assertUnsponsoredPlan(sponsored);
    assertSolActionAmount(amountLamports, "withdraw");
    const wallet = this.planningWallet(state);
    const breakdown = state.solBalanceBreakdown();
    const rpc = new Rpc(this.client);
    const direct = new Transaction({ feePayer: wallet }).add(
      SystemProgram.transfer({ fromPubkey: wallet, toPubkey: recipient, lamports: amountLamports })
    );
    const directFee = await rpc.prepareAndEstimateTransactionFee(direct);
    const directCosts: SolActionCosts = {
      feeLamports: directFee,
      upfrontRentLamports: 0n,
      createsCanonicalWsolAccount: false,
      sponsored,
    };
    const directAvailability = solBalanceAvailability(breakdown, directCosts);
    if (amountLamports > directAvailability.spendableLamports) {
      throw SdkError.validation(
        "withdraw amount exceeds spendable SOL after transaction reserve"
      );
    }
    if (
      breakdown.nativeLamports >=
      amountLamports + directAvailability.reserveLamports
    ) {
      return {
        kind: "nativeWithdraw",
        transaction: direct,
        costs: directCosts,
        availability: directAvailability,
        expectedDelta: {
          nativeLamports: -amountLamports - (sponsored ? 0n : directFee),
          canonicalWsolLamports: 0n,
        },
      };
    }

    const canonical = getAssociatedTokenAddressSync(NATIVE_MINT, wallet);
    if (!(await rpc.canonicalWsolAccountExists(canonical, wallet))) {
      throw SdkError.validation(
        "canonical WSOL is required for this native withdrawal"
      );
    }
    const temporaryRent = await rpc.minimumBalanceForRentExemption(TOKEN_ACCOUNT_SPACE);
    const { blockhash, lastValidBlockHeight } = await rpc.getLatestBlockhash();
    let seed: string | undefined;
    let temporary: PublicKey | undefined;
    // Bound account-existence RPCs; the blockhash and attempt byte make eight collisions remote.
    for (let attempt = 0; attempt <= 7; attempt++) {
      const candidateSeed = nativeWithdrawSeed(
        blockhash,
        wallet,
        recipient,
        amountLamports,
        attempt
      );
      const candidate = await PublicKey.createWithSeed(
        wallet,
        candidateSeed,
        TOKEN_PROGRAM_ID
      );
      if (!(await rpc.accountExists(candidate))) {
        seed = candidateSeed;
        temporary = candidate;
        break;
      }
    }
    if (!seed || !temporary) {
      throw SdkError.validation("temporary WSOL seed attempts are exhausted");
    }

    let transaction = this.buildTemporaryNativeWithdrawal(
      wallet,
      recipient,
      amountLamports,
      1n,
      temporaryRent,
      seed,
      temporary
    );
    transaction.recentBlockhash = blockhash;
    transaction.lastValidBlockHeight = lastValidBlockHeight;
    const initialFee = await rpc.estimatePreparedTransactionFee(transaction);
    const initialCosts: SolActionCosts = {
      feeLamports: initialFee,
      upfrontRentLamports: temporaryRent,
      createsCanonicalWsolAccount: false,
      sponsored,
    };
    const initialAvailability = solBalanceAvailability(breakdown, initialCosts);
    const initialRequired = amountLamports + initialAvailability.reserveLamports;
    if (initialRequired < breakdown.nativeLamports) {
      throw SdkError.validation("invalid temporary withdrawal requirement");
    }
    const initialTransfer = initialRequired - breakdown.nativeLamports;
    transaction = this.buildTemporaryNativeWithdrawal(
      wallet,
      recipient,
      amountLamports,
      initialTransfer,
      temporaryRent,
      seed,
      temporary
    );
    transaction.recentBlockhash = blockhash;
    transaction.lastValidBlockHeight = lastValidBlockHeight;
    const finalFee = await rpc.estimatePreparedTransactionFee(transaction);
    const costs: SolActionCosts = {
      feeLamports: finalFee,
      upfrontRentLamports: temporaryRent,
      createsCanonicalWsolAccount: false,
      sponsored,
    };
    const availability = solBalanceAvailability(breakdown, costs);
    const finalRequired = amountLamports + availability.reserveLamports;
    if (finalRequired < breakdown.nativeLamports) {
      throw SdkError.validation("invalid temporary withdrawal requirement");
    }
    const canonicalTransfer = finalRequired - breakdown.nativeLamports;
    if (canonicalTransfer > breakdown.canonicalWsolLamports) {
      throw SdkError.validation(
        "canonical WSOL cannot fund the native withdrawal shortfall"
      );
    }
    if (canonicalTransfer !== initialTransfer) {
      transaction = this.buildTemporaryNativeWithdrawal(
        wallet,
        recipient,
        amountLamports,
        canonicalTransfer,
        temporaryRent,
        seed,
        temporary
      );
      transaction.recentBlockhash = blockhash;
      transaction.lastValidBlockHeight = lastValidBlockHeight;
      const stableFee = await rpc.estimatePreparedTransactionFee(transaction);
      if (stableFee !== finalFee) {
        throw SdkError.validation(
          "transaction fee changed while rebuilding native withdrawal"
        );
      }
    }
    return {
      kind: "nativeWithdraw",
      transaction,
      costs,
      availability,
      expectedDelta: {
        nativeLamports:
          canonicalTransfer - amountLamports - (sponsored ? 0n : finalFee),
        canonicalWsolLamports: -canonicalTransfer,
      },
    };
  }

  /**
   * Resolve an unexpired wallet from matching initialized balance state.
   * This requires the configured signing strategy to control the authenticated
   * wallet. It does not prove balance freshness.
   */
  private planningWallet(state: WalletDepositBalancesState): PublicKey {
    // Cached identity is a signing trust boundary: validate expiry, complete
    // state initialization, and wallet equality before constructing a transaction.
    const credentials = this.client.authCredentials;
    if (!credentials) {
      throw SdkError.validation("authenticated credentials are required");
    }
    if (!isAuthenticated(credentials)) {
      throw SdkError.validation("authenticated credentials have expired");
    }
    if (
      state.walletAddress === undefined ||
      state.contextSlot === undefined ||
      state.nativeSolBalance === undefined
    ) {
      throw SdkError.validation("wallet balance state is not initialized");
    }
    if (state.walletAddress !== credentials.wallet_address) {
      throw SdkError.validation(
        "authenticated wallet does not match wallet balance state"
      );
    }
    let wallet: PublicKey;
    try {
      wallet = new PublicKey(credentials.wallet_address);
    } catch (error) {
      throw SdkError.validation(
        `authenticated wallet is invalid: ${error instanceof Error ? error.message : String(error)}`
      );
    }
    const strategy = requireSigningStrategy(this.client);
    const signingAddress = signingStrategyWalletAddress(strategy);
    if (!signingAddress) {
      throw SdkError.validation("signing strategy wallet identity is required");
    }
    let signingWallet: PublicKey;
    try {
      signingWallet = new PublicKey(signingAddress);
    } catch (error) {
      throw SdkError.validation(
        `signing strategy wallet is invalid: ${error instanceof Error ? error.message : String(error)}`
      );
    }
    if (!signingWallet.equals(wallet)) {
      throw SdkError.validation(
        "signing strategy does not control authenticated wallet"
      );
    }
    return wallet;
  }

  /**
   * Return the authenticated Trading Wallet after native-keypair validation.
   *
   * Complete wallet state and identity are validated by `planningWallet`. This
   * additional guard rejects wallet-adapter and Privy strategies before conversion
   * RPC reads. Ordinary planners do not call this method.
   */
  private conversionPlanningWallet(
    state: WalletDepositBalancesState
  ): PublicKey {
    requireNativeSigningStrategy(requireSigningStrategy(this.client));
    return this.planningWallet(state);
  }

  /**
   * Return an unsigned wrap transaction with exact instruction ordering.
   *
   * When planning observed no canonical ATA, the first instruction is strict ATA
   * creation. A concurrently created ATA therefore makes execution fail instead of
   * using account state that was absent from the plan. The transfer and `SyncNative`
   * instructions follow. Fee preparation later attaches the live blockhash.
   */
  private buildWrapSolTransaction(
    wallet: PublicKey,
    amountLamports: bigint,
    createsCanonicalWsolAccount: boolean
  ): Transaction {
    const canonical = getAssociatedTokenAddressSync(NATIVE_MINT, wallet);
    const transaction = new Transaction({ feePayer: wallet });
    if (createsCanonicalWsolAccount) {
      transaction.add(
        createAssociatedTokenAccountInstruction(
          wallet,
          canonical,
          wallet,
          NATIVE_MINT,
          TOKEN_PROGRAM_ID
        )
      );
    }
    return transaction.add(
      SystemProgram.transfer({
        fromPubkey: wallet,
        toPubkey: canonical,
        lamports: amountLamports,
      }),
      createSyncNativeInstruction(canonical, TOKEN_PROGRAM_ID)
    );
  }

  /**
   * Return an unsigned transaction containing one canonical `CloseAccount`.
   *
   * The Trading Wallet is the fee payer, close authority, and destination. A later
   * successful submission transfers the complete account balance to that wallet.
   */
  private buildUnwrapWsolAllTransaction(wallet: PublicKey): Transaction {
    const canonical = getAssociatedTokenAddressSync(NATIVE_MINT, wallet);
    return new Transaction({ feePayer: wallet }).add(
      createCloseAccountInstruction(
        canonical,
        wallet,
        wallet,
        [],
        TOKEN_PROGRAM_ID
      )
    );
  }

  /**
   * Build idempotent creation of the persistent Tokenkeg WSOL ATA.
   * Tokenkeg is Solana's legacy SPL Token Program; canonical native-mint ATA
   * derivation is pinned to it rather than Token-2022 across the protocol.
   */
  private createCanonicalWsolAccount(wallet: PublicKey): TransactionInstruction {
    const canonical = getAssociatedTokenAddressSync(NATIVE_MINT, wallet);
    return createAssociatedTokenAccountIdempotentInstruction(
      wallet,
      canonical,
      wallet,
      NATIVE_MINT,
      TOKEN_PROGRAM_ID
    );
  }

  /** Read canonical-account existence and upfront rent for merge/redeem plans. */
  private async receivePlanContext(
    wallet: PublicKey,
    state: WalletDepositBalancesState
  ): Promise<{
    rpc: Rpc;
    breakdown: SolBalanceBreakdown;
    canonicalExists: boolean;
    upfrontRentLamports: bigint;
  }> {
    const rpc = new Rpc(this.client);
    const canonical = getAssociatedTokenAddressSync(NATIVE_MINT, wallet);
    const canonicalExists = await rpc.canonicalWsolAccountExists(canonical, wallet);
    const breakdown = state.solBalanceBreakdown();
    if (breakdown.canonicalWsolLamports > 0n && !canonicalExists) {
      throw SdkError.validation(
        "canonical WSOL balance is positive but its account is unavailable"
      );
    }
    return {
      rpc,
      breakdown,
      canonicalExists,
      upfrontRentLamports: canonicalExists
        ? 0n
        : await rpc.minimumBalanceForRentExemption(TOKEN_ACCOUNT_SPACE),
    };
  }

  /** Finish merge/redeem planning with live fee authority and balance deltas. */
  private async finishReceivePlan(
    kind: "merge" | "redeem",
    amountLamports: bigint,
    transaction: Transaction,
    rpc: Rpc,
    breakdown: SolBalanceBreakdown,
    upfrontRentLamports: bigint,
    createsCanonicalWsolAccount: boolean,
    sponsored: boolean
  ): Promise<SolActionPlan> {
    const feeLamports = await rpc.prepareAndEstimateTransactionFee(transaction);
    const costs: SolActionCosts = {
      feeLamports,
      upfrontRentLamports,
      createsCanonicalWsolAccount,
      sponsored,
    };
    const availability = solBalanceAvailability(breakdown, costs);
    const walletCosts = sponsored ? 0n : feeLamports + upfrontRentLamports;
    return {
      kind,
      transaction,
      costs,
      availability,
      expectedDelta: {
        nativeLamports: -walletCosts,
        canonicalWsolLamports: amountLamports,
      },
    };
  }

  /**
   * Build the sole WSOL-to-native path without closing canonical authority.
   * The temporary Tokenkeg account is initialized, funded, and closed back to
   * the wallet before the exact recipient transfer in the same transaction.
   */
  private buildTemporaryNativeWithdrawal(
    wallet: PublicKey,
    recipient: PublicKey,
    amountLamports: bigint,
    canonicalTransfer: bigint,
    temporaryRent: bigint,
    seed: string,
    temporary: PublicKey
  ): Transaction {
    const canonical = getAssociatedTokenAddressSync(NATIVE_MINT, wallet);
    return new Transaction({ feePayer: wallet }).add(
      SystemProgram.createAccountWithSeed({
        fromPubkey: wallet,
        newAccountPubkey: temporary,
        basePubkey: wallet,
        seed,
        lamports: Number(temporaryRent),
        space: TOKEN_ACCOUNT_SPACE,
        programId: TOKEN_PROGRAM_ID,
      }),
      createInitializeAccount3Instruction(
        temporary,
        NATIVE_MINT,
        wallet,
        TOKEN_PROGRAM_ID
      ),
      createTransferInstruction(
        canonical,
        temporary,
        wallet,
        canonicalTransfer,
        [],
        TOKEN_PROGRAM_ID
      ),
      createCloseAccountInstruction(
        temporary,
        wallet,
        wallet,
        [],
        TOKEN_PROGRAM_ID
      ),
      SystemProgram.transfer({
        fromPubkey: wallet,
        toPubkey: recipient,
        lamports: amountLamports,
      })
    );
  }

  // ── On-chain transaction builders ────────────────────────────────────

  redeemWinningsIx(
    params: RedeemWinningsParams,
    outcomeIndex: number
  ): TransactionInstruction {
    return buildRedeemWinningsIx(params, outcomeIndex, this.client.programId);
  }

  withdrawConditionalFromPositionIx(
    params: WithdrawConditionalFromPositionParams
  ): TransactionInstruction {
    return buildWithdrawConditionalFromPositionIx(params, this.client.programId);
  }

  withdrawFromPositionIx(
    params: WithdrawFromPositionParams
  ): TransactionInstruction {
    return this.withdrawConditionalFromPositionIx(params);
  }

  initPositionTokensIx(
    params: InitPositionTokensParams,
    numOutcomes: number
  ): TransactionInstruction {
    return buildInitPositionTokensIx(params, numOutcomes, this.client.programId);
  }

  extendPositionTokensIx(
    params: ExtendPositionTokensParams,
    numOutcomes: number
  ): TransactionInstruction {
    return buildExtendPositionTokensIx(params, numOutcomes, this.client.programId);
  }

  depositToGlobalIx(params: DepositToGlobalParams): TransactionInstruction {
    return buildDepositToGlobalIx(params, this.client.programId);
  }

  depositToGlobalIxWithAlt(
    params: DepositToGlobalParams,
    altContext: DepositToGlobalAltContext
  ): TransactionInstruction {
    return buildDepositToGlobalIxWithAlt(params, altContext, this.client.programId);
  }

  globalToMarketDepositIx(
    params: GlobalToMarketDepositParams,
    numOutcomes: number
  ): TransactionInstruction {
    return buildGlobalToMarketDepositIx(params, numOutcomes, this.client.programId);
  }

  withdrawFromGlobalIx(params: WithdrawFromGlobalParams): TransactionInstruction {
    return buildWithdrawFromGlobalIx(params, this.client.programId);
  }

  closePositionAltIx(params: ClosePositionAltParams): TransactionInstruction {
    return buildClosePositionAltIx(params, this.client.programId);
  }

  closePositionTokenAccountsIx(
    params: ClosePositionTokenAccountsParams,
    numOutcomes: number
  ): TransactionInstruction {
    return buildClosePositionTokenAccountsIx(
      params,
      numOutcomes,
      this.client.programId
    );
  }

  // ── Transaction builders (_tx convenience wrappers) ─────────────────

  redeemWinningsTx(
    params: RedeemWinningsParams,
    outcomeIndex: number
  ): Transaction {
    const ix = this.redeemWinningsIx(params, outcomeIndex);
    return new Transaction({ feePayer: params.user }).add(ix);
  }

  withdrawConditionalFromPositionTx(params: WithdrawConditionalFromPositionParams): Transaction {
    const ix = this.withdrawConditionalFromPositionIx(params);
    return new Transaction({ feePayer: params.user }).add(ix);
  }

  withdrawFromPositionTx(params: WithdrawFromPositionParams): Transaction {
    return this.withdrawConditionalFromPositionTx(params);
  }

  initPositionTokensTx(
    params: InitPositionTokensParams,
    numOutcomes: number
  ): Transaction {
    const ix = this.initPositionTokensIx(params, numOutcomes);
    return new Transaction({ feePayer: params.payer }).add(ix);
  }

  extendPositionTokensTx(
    params: ExtendPositionTokensParams,
    numOutcomes: number
  ): Transaction {
    const ix = this.extendPositionTokensIx(params, numOutcomes);
    return new Transaction({ feePayer: params.payer }).add(ix);
  }

  depositToGlobalTx(params: DepositToGlobalParams): Transaction {
    const ix = this.depositToGlobalIx(params);
    return new Transaction({ feePayer: params.user }).add(ix);
  }

  depositToGlobalTxWithAlt(
    params: DepositToGlobalParams,
    altContext: DepositToGlobalAltContext
  ): Transaction {
    const ix = this.depositToGlobalIxWithAlt(params, altContext);
    return new Transaction({ feePayer: params.user }).add(ix);
  }

  globalToMarketDepositTx(
    params: GlobalToMarketDepositParams,
    numOutcomes: number
  ): Transaction {
    const ix = this.globalToMarketDepositIx(params, numOutcomes);
    return new Transaction({ feePayer: params.user }).add(ix);
  }

  withdrawFromGlobalTx(params: WithdrawFromGlobalParams): Transaction {
    const ix = this.withdrawFromGlobalIx(params);
    return new Transaction({ feePayer: params.user }).add(ix);
  }

  closePositionAltTx(params: ClosePositionAltParams): Transaction {
    const ix = this.closePositionAltIx(params);
    return new Transaction({ feePayer: params.operator }).add(ix);
  }

  closePositionTokenAccountsTx(
    params: ClosePositionTokenAccountsParams,
    numOutcomes: number
  ): Transaction {
    const ix = this.closePositionTokenAccountsIx(params, numOutcomes);
    return new Transaction({ feePayer: params.operator }).add(ix);
  }

  // ── Builder factories ──────────────────────────────────────────────

  deposit(): DepositBuilder {
    return new DepositBuilder(this.client, this.client.depositSource);
  }

  merge(): MergeBuilder {
    return new MergeBuilder(this.client);
  }

  withdraw(): WithdrawBuilder {
    return new WithdrawBuilder(this.client, this.client.depositSource);
  }

  redeemWinnings(): RedeemWinningsBuilder {
    return new RedeemWinningsBuilder(this.client);
  }

  withdrawFromPosition(): WithdrawFromPositionBuilder {
    return new WithdrawFromPositionBuilder(this.client);
  }

  withdrawConditionalFromPosition(): WithdrawFromPositionBuilder {
    return new WithdrawFromPositionBuilder(this.client);
  }

  initPositionTokens(): InitPositionTokensBuilder {
    return new InitPositionTokensBuilder(this.client);
  }

  extendPositionTokens(): ExtendPositionTokensBuilder {
    return new ExtendPositionTokensBuilder(this.client);
  }

  depositToGlobal(): DepositToGlobalBuilder {
    return new DepositToGlobalBuilder(this.client);
  }

  withdrawFromGlobal(): WithdrawFromGlobalBuilder {
    return new WithdrawFromGlobalBuilder(this.client);
  }

  globalToMarketDeposit(): GlobalToMarketDepositBuilder {
    return new GlobalToMarketDepositBuilder(this.client);
  }

  // ── On-chain account fetchers (require Connection) ──────────────────

  async getOnchain(owner: PublicKey, market: PublicKey): Promise<ProgramPosition | null> {
    const connection = requireConnection(this.client);
    const positionPda = this.pda(owner, market);
    const accountInfo = await connection.getAccountInfo(positionPda);
    if (!accountInfo) {
      return null;
    }
    return deserializeProgramPosition(accountInfo.data as Buffer);
  }
}
