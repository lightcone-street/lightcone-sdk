import Decimal from "decimal.js";
import { SdkError } from "../../error";
import { exactScaledInteger } from "../../shared";
import type { PubkeyStr } from "../../shared";
import type {
  DepositTokenBalance,
  DepositTokenBalancesSnapshot,
  WalletDepositBalancesEvent,
} from "./index";

/**
 * Canonical WSOL mint under Solana's legacy SPL Token Program (“Tokenkeg”).
 * ATA derivation stays pinned to Tokenkeg rather than Token-2022 so state and
 * transaction planning address the protocol's one canonical account.
 */
export const WRAPPED_SOL_MINT =
  "So11111111111111111111111111111111111111112" as PubkeyStr;

/** Unsponsored native reserve floor, in lamports, when canonical ATA creation is required. */
export const SOL_RESERVE_WITH_ACCOUNT_CREATION_LAMPORTS = 3_500_000n;
/** Unsponsored native reserve floor, in lamports, when canonical WSOL already exists. */
export const SOL_RESERVE_WITH_EXISTING_ACCOUNT_LAMPORTS = 1_000_000n;
/** Maximum exact non-negative lamport value representable by Solana's u64 fields. */
const MAX_SOLANA_LAMPORTS = 0xffff_ffff_ffff_ffffn;

/** Exact components behind the single displayed SOL asset. */
export interface SolBalanceComponents {
  /** Lamports held by the Trading Wallet system account. */
  nativeLamports: bigint;
  /** Token amount in the Trading Wallet's persistent canonical WSOL ATA. */
  canonicalWsolLamports: bigint;
}

/**
 * Stores live chain costs used to derive transaction funding requirements.
 *
 * {@link solBalanceAvailability} applies ordinary safety floors.
 * {@link unwrapAllSolBalanceAvailability} rejects account creation, upfront rent,
 * and sponsorship, then uses only `feeLamports` as the unwrap-all reserve.
 */
export interface SolActionCosts {
  /** Live `getFeeForMessage` result, in lamports. */
  feeLamports: bigint;
  /** Rent funded up front, even when a temporary account refunds it later. */
  upfrontRentLamports: bigint;
  /** Whether this transaction must create the persistent canonical WSOL ATA. */
  createsCanonicalWsolAccount: boolean;
  /** Exact public sponsorship capability supplied by the caller. */
  sponsored: boolean;
}

/** Action-specific displayed, reserved, and spendable SOL values. */
export interface SolBalanceAvailability {
  /** Separately authoritative native and canonical WSOL balances. */
  components: SolBalanceComponents;
  /** Sum of both components in lamports, before reserve. */
  displayedLamports: bigint;
  /** Native lamports withheld for live costs and any action-specific safety floor. */
  reserveLamports: bigint;
  /** Displayed lamports available to this action after reserve. */
  spendableLamports: bigint;
}

/** Derive fail-closed availability and type an insufficient native reserve as fee funding. */
export function solBalanceAvailability(
  components: SolBalanceComponents,
  costs: SolActionCosts
): SolBalanceAvailability {
  for (const [label, value] of [
    ["native SOL", components.nativeLamports],
    ["canonical WSOL", components.canonicalWsolLamports],
  ] as const) {
    if (typeof value !== "bigint" || value < 0n || value > MAX_SOLANA_LAMPORTS) {
      throw SdkError.validation(`${label} must fit the non-negative u64 lamport range`);
    }
  }
  for (const [label, value] of [
    ["transaction fee", costs.feeLamports],
    ["upfront rent", costs.upfrontRentLamports],
  ] as const) {
    if (typeof value !== "bigint" || value < 0n || value > MAX_SOLANA_LAMPORTS) {
      throw SdkError.validation(`${label} must fit the non-negative u64 lamport range`);
    }
  }
  const displayedLamports =
    components.nativeLamports + components.canonicalWsolLamports;
  if (displayedLamports > MAX_SOLANA_LAMPORTS) {
    throw SdkError.validation("displayed SOL exceeds the transaction u64 range");
  }
  const liveCosts = costs.feeLamports + costs.upfrontRentLamports;
  if (liveCosts > MAX_SOLANA_LAMPORTS) {
    throw SdkError.validation("combined transaction costs must fit u64 lamports");
  }
  const floor = costs.createsCanonicalWsolAccount
    ? SOL_RESERVE_WITH_ACCOUNT_CREATION_LAMPORTS
    : SOL_RESERVE_WITH_EXISTING_ACCOUNT_LAMPORTS;
  const reserveLamports = costs.sponsored
    ? 0n
    : liveCosts > floor
      ? liveCosts
      : floor;
  if (components.nativeLamports < reserveLamports) {
    throw SdkError.insufficientSolForTransactionFees(
      components.nativeLamports,
      reserveLamports
    );
  }
  return {
    components,
    displayedLamports,
    reserveLamports,
    spendableLamports: displayedLamports - reserveLamports,
  };
}

/**
 * Return unwrap-all availability with the live fee as its entire reserve.
 *
 * This function rejects costs that include sponsorship, account creation, or
 * upfront rent. It rejects malformed or out-of-range components and costs. It
 * rejects a displayed-balance sum outside Solana's unsigned 64-bit range. Native
 * SOL must fund the fee without relying on lamports that a later `CloseAccount`
 * instruction may transfer. The ordinary persistent-account floor does not apply
 * because unwrap-all removes that account. An insufficient native fee balance
 * returns the typed transaction-fee error.
 */
export function unwrapAllSolBalanceAvailability(
  components: SolBalanceComponents,
  costs: SolActionCosts
): SolBalanceAvailability {
  for (const [label, value] of [
    ["native SOL", components.nativeLamports],
    ["canonical WSOL", components.canonicalWsolLamports],
  ] as const) {
    if (typeof value !== "bigint" || value < 0n || value > MAX_SOLANA_LAMPORTS) {
      throw SdkError.validation(`${label} must fit the non-negative u64 lamport range`);
    }
  }
  for (const [label, value] of [
    ["transaction fee", costs.feeLamports],
    ["upfront rent", costs.upfrontRentLamports],
  ] as const) {
    if (typeof value !== "bigint" || value < 0n || value > MAX_SOLANA_LAMPORTS) {
      throw SdkError.validation(`${label} must fit the non-negative u64 lamport range`);
    }
  }
  if (
    costs.upfrontRentLamports !== 0n ||
    costs.createsCanonicalWsolAccount ||
    costs.sponsored
  ) {
    throw SdkError.validation(
      "unwrap-all costs must be unsponsored with no upfront rent or account creation"
    );
  }
  const displayedLamports =
    components.nativeLamports + components.canonicalWsolLamports;
  if (displayedLamports > MAX_SOLANA_LAMPORTS) {
    throw SdkError.validation("displayed SOL exceeds the transaction u64 range");
  }
  if (components.nativeLamports < costs.feeLamports) {
    throw SdkError.insufficientSolForTransactionFees(
      components.nativeLamports,
      costs.feeLamports
    );
  }
  return {
    components,
    displayedLamports,
    reserveLamports: costs.feeLamports,
    spendableLamports: displayedLamports - costs.feeLamports,
  };
}

/** Whether an event was accepted by, or rejected by, its lifecycle guard. */
export type WalletDepositBalancesApplyResult =
  | { kind: "applied" }
  | { kind: "ignored" }
  | { kind: "rejected" };

/**
 * Mutable application-owned wallet balance state.
 *
 * The default instance is uninitialized. A complete REST or WebSocket snapshot
 * establishes its wallet baseline; later component events must match that wallet
 * and carry absolute values. Native SOL never enters the sparse SPL mint map, and
 * complete cross-component snapshots may replace `contextSlot` with a lower slot.
 * Map containers are copied, but mutable balance objects are retained by reference;
 * treat payload entries as immutable after applying them.
 */
export class WalletDepositBalancesState {
  /** Wallet owning the current baseline, or `undefined` before initialization. */
  walletAddress: PubkeyStr | undefined;
  /** Slot of the last accepted event, not a globally monotonic stream version. */
  contextSlot: number | undefined;
  /** Sparse mint-keyed SPL balances; explicit zero updates remove their mint. */
  readonly balances = new Map<PubkeyStr, DepositTokenBalance>();
  /** Exact nine-decimal native SOL, separate from `balances`. */
  nativeSolBalance: string | undefined;

  /**
   * Initialize or wholesale-replace state from a complete REST snapshot.
   * The caller supplies the wallet omitted by REST. When `minimumSnapshotSlot`
   * is present, a lower complete snapshot is ignored without mutation; otherwise
   * prior slots are not compared.
   * Unlike WebSocket parsing, this TypeScript REST boundary performs no runtime
   * shape validation. Malformed exact values fail when a derived method scales them.
   */
  applyRestSnapshot(
    walletAddress: PubkeyStr,
    snapshot: DepositTokenBalancesSnapshot,
    minimumSnapshotSlot?: number
  ): WalletDepositBalancesApplyResult {
    if (
      minimumSnapshotSlot !== undefined &&
      snapshot.context_slot < minimumSnapshotSlot
    ) {
      return { kind: "ignored" };
    }
    this.replace(
      walletAddress,
      snapshot.context_slot,
      snapshot.balances,
      snapshot.native_sol_balance
    );
    return { kind: "applied" };
  }

  /**
   * Apply the wallet event state machine.
   *
   * Complete snapshots replace state unless they are below an optional minimum
   * snapshot slot. The floor never applies to component or status events.
   * Matching component events replace one absolute value, zero SPL removes its
   * mint, and status, pre-initialization, or wrong-wallet events return `ignored`.
   */
  applyEvent(
    event: WalletDepositBalancesEvent,
    minimumSnapshotSlot?: number
  ): WalletDepositBalancesApplyResult {
    switch (event.event_type) {
      case "wallet_deposit_balance_snapshot":
        if (
          minimumSnapshotSlot !== undefined &&
          event.context_slot < minimumSnapshotSlot
        ) {
          return { kind: "ignored" };
        }
        // Complete snapshots are authoritative even when their lower
        // cross-component slot trails a previously observed update.
        this.replace(
          event.wallet_address,
          event.context_slot,
          event.balances,
          event.native_sol_balance
        );
        return { kind: "applied" };
      case "wallet_deposit_balance_update":
        if (!this.matchesInitializedWallet(event.wallet_address)) {
          return { kind: "ignored" };
        }
        let isZero: boolean;
        try {
          isZero = isZeroTokenAmount(event.balance.idle);
        } catch {
          return { kind: "rejected" };
        }
        if (isZero) {
          this.balances.delete(event.balance.mint);
        } else {
          this.balances.set(event.balance.mint, event.balance);
        }
        this.contextSlot = event.context_slot;
        return { kind: "applied" };
      case "wallet_native_sol_balance_update":
        if (!this.matchesInitializedWallet(event.wallet_address)) {
          return { kind: "ignored" };
        }
        this.nativeSolBalance = event.native_sol_balance;
        this.contextSlot = event.context_slot;
        return { kind: "applied" };
      case "wallet_deposit_balance_status":
        return { kind: "ignored" };
    }
  }

  /**
   * Return exact native SOL plus canonical WSOL with nine fractional digits.
   * Uses arbitrary-width integer arithmetic and never merges the stored assets.
   */
  combinedSolBalance(): string {
    const native = this.nativeSolLamports();
    const wrapped = this.balances.get(WRAPPED_SOL_MINT);
    const wrappedLamports = wrapped
      ? exactScaledInteger(wrapped.idle, 9)
      : 0n;
    return formatLamports(native + wrappedLamports);
  }

  /** Return exact native and canonical WSOL components for transaction planning. */
  solComponents(): SolBalanceComponents {
    let nativeLamports: bigint;
    let canonicalWsolLamports: bigint;
    try {
      nativeLamports = this.nativeSolLamports();
      canonicalWsolLamports = this.canonicalWsolLamports();
    } catch (error) {
      throw SdkError.validation(
        `invalid SOL balance component: ${error instanceof Error ? error.message : String(error)}`
      );
    }
    if (
      nativeLamports > MAX_SOLANA_LAMPORTS ||
      canonicalWsolLamports > MAX_SOLANA_LAMPORTS
    ) {
      throw SdkError.validation("SOL component exceeds the transaction u64 range");
    }
    return { nativeLamports, canonicalWsolLamports };
  }

  /** Scale cached native SOL exactly to lamports; requires initialized state. */
  nativeSolLamports(): bigint {
    if (this.nativeSolBalance === undefined) {
      throw new Error("wallet balance state is not initialized");
    }
    return exactScaledInteger(this.nativeSolBalance, 9);
  }

  /** Scale the canonical WSOL idle balance exactly to lamports. */
  canonicalWsolLamports(): bigint {
    const wrapped = this.balances.get(WRAPPED_SOL_MINT);
    return wrapped ? exactScaledInteger(wrapped.idle, 9) : 0n;
  }

  private matchesInitializedWallet(walletAddress: PubkeyStr): boolean {
    return (
      this.walletAddress === walletAddress &&
      this.contextSlot !== undefined &&
      this.nativeSolBalance !== undefined
    );
  }

  private replace(
    walletAddress: PubkeyStr,
    contextSlot: number,
    balances: Record<PubkeyStr, DepositTokenBalance>,
    nativeSolBalance: string
  ): void {
    // Own the map lifecycle while retaining payload objects by reference. Callers
    // must treat balance records as immutable after handing them to state.
    this.walletAddress = walletAddress;
    this.contextSlot = contextSlot;
    this.balances.clear();
    for (const balance of Object.values(balances)) {
      this.balances.set(balance.mint, balance);
    }
    this.nativeSolBalance = nativeSolBalance;
  }
}

function formatLamports(value: bigint): string {
  // Integer division restores fixed-scale SOL text without floating-point loss.
  const scale = 1_000_000_000n;
  const whole = value / scale;
  const fraction = (value % scale).toString().padStart(9, "0");
  return `${whole}.${fraction}`;
}

/** Detect explicit zero without imposing SOL's nine-decimal scale on SPL tokens. */
function isZeroTokenAmount(value: string): boolean {
  const amount = new Decimal(value);
  if (!amount.isFinite() || amount.isNegative()) {
    throw new Error(`invalid deposit-token balance: ${value}`);
  }
  return amount.isZero();
}
