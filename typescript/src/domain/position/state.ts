import Decimal from "decimal.js";
import { exactScaledInteger } from "../../shared";
import type { PubkeyStr } from "../../shared";
import type {
  DepositTokenBalance,
  DepositTokenBalancesSnapshot,
  WalletDepositBalancesEvent,
} from "./index";

/** Canonical Tokenkeg wrapped-SOL mint used by state and conversion preflight. */
export const WRAPPED_SOL_MINT =
  "So11111111111111111111111111111111111111112" as PubkeyStr;

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
   * The caller supplies the wallet omitted by REST; prior slots are not compared.
   * Unlike WebSocket parsing, this TypeScript REST boundary performs no runtime
   * shape validation. Malformed exact values fail when a derived method scales them.
   */
  applyRestSnapshot(
    walletAddress: PubkeyStr,
    snapshot: DepositTokenBalancesSnapshot
  ): WalletDepositBalancesApplyResult {
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
   * Complete snapshots always replace state. Matching component events replace
   * one absolute value, zero SPL removes its mint, and status, pre-initialization,
   * or wrong-wallet events return `ignored` without mutation.
   */
  applyEvent(event: WalletDepositBalancesEvent): WalletDepositBalancesApplyResult {
    switch (event.event_type) {
      case "wallet_deposit_balance_snapshot":
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

  /** Scale cached native SOL exactly to lamports; requires initialized state. */
  nativeSolLamports(): bigint {
    if (this.nativeSolBalance === undefined) {
      throw new Error("wallet balance state is not initialized");
    }
    return exactScaledInteger(this.nativeSolBalance, 9);
  }

  /** Validate and test the canonical WSOL idle amount used by unwrap preflight. */
  hasPositiveWsol(): boolean {
    const wrapped = this.balances.get(WRAPPED_SOL_MINT);
    return wrapped !== undefined && exactScaledInteger(wrapped.idle, 9) > 0n;
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
