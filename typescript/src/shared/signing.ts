import type { Keypair } from "@solana/web3.js";

export interface ExternalSigner {
  /** Wallet controlled by this signer for identity-bound transactions. */
  readonly walletAddress?: string;
  signMessage(message: Uint8Array): Promise<Uint8Array>;
  signTransaction(txBytes: Uint8Array): Promise<Uint8Array>;
}

export type SigningStrategy =
  | { type: "native"; keypair: Keypair }
  | { type: "walletAdapter"; signer: ExternalSigner }
  | { type: "privy"; walletId: string; walletAddress?: string };

/** Return the wallet identity this strategy can prove before signing. */
export function signingStrategyWalletAddress(
  strategy: SigningStrategy
): string | undefined {
  switch (strategy.type) {
    case "native":
      return strategy.keypair.publicKey.toBase58();
    case "walletAdapter":
      return strategy.signer.walletAddress;
    case "privy":
      return strategy.walletAddress;
  }
}

export function isUserCancellation(error: string): boolean {
  const lower = error.toLowerCase();
  return (
    lower.includes("reject") ||
    lower.includes("cancel") ||
    lower.includes("denied") ||
    lower.includes("user refused") ||
    lower.includes("declined") ||
    lower.includes("reflect.get called on non-object")
  );
}
