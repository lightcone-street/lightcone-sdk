/**
 * RPC failover: automatic switch to a backup Solana RPC endpoint on
 * infrastructure errors, with 120 s cooldown recovery to primary.
 *
 * Mirrors rust/src/rpc_failover.rs.
 */

export const FAST_RETRY_DELAY_MS = 100;
export const COOLDOWN_DURATION_MS = 120_000;

export enum ActiveRpc {
  Primary = "primary",
  Backup = "backup",
}

export class RpcFailoverState {
  active: ActiveRpc = ActiveRpc.Primary;
  private flippedToBackupAt?: number;

  maybeRecoverToPrimary(): void {
    if (
      this.active === ActiveRpc.Backup &&
      this.flippedToBackupAt !== undefined
    ) {
      if (Date.now() - this.flippedToBackupAt >= COOLDOWN_DURATION_MS) {
        this.active = ActiveRpc.Primary;
        this.flippedToBackupAt = undefined;
      }
    }
  }

  flipToBackup(): void {
    this.active = ActiveRpc.Backup;
    this.flippedToBackupAt = Date.now();
  }

  flipToPrimary(): void {
    this.active = ActiveRpc.Primary;
    this.flippedToBackupAt = undefined;
  }
}

export function isInfrastructureError(err: unknown): boolean {
  if (err instanceof TypeError) {
    const message = err.message.toLowerCase();
    if (
      message.includes("fetch") ||
      message.includes("network") ||
      message.includes("failed")
    ) {
      return true;
    }
  }

  if (err instanceof Error) {
    const message = err.message.toLowerCase();
    if (message.includes("timeout") || message.includes("timed out")) {
      return true;
    }
    if (
      message.includes("502") ||
      message.includes("503") ||
      message.includes("504")
    ) {
      return true;
    }
    if (
      message.includes("econnrefused") ||
      message.includes("econnreset") ||
      message.includes("enotfound")
    ) {
      return true;
    }
  }

  return false;
}

export function sleep(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
