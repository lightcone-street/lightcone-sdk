import type { Keypair } from "@solana/web3.js";
import { SdkError } from "../error";

export interface ExternalSigner {
  /** Wallet controlled by this signer for identity-bound transactions. */
  readonly walletAddress?: string;
  signMessage(message: Uint8Array): Promise<Uint8Array>;
  /** Sign canonical Solana v1 bytes without changing any message field; return all required signatures. */
  signTransaction(txBytes: Uint8Array): Promise<Uint8Array>;
}

export type SigningStrategy =
  | { type: "native"; keypair: Keypair }
  | { type: "walletAdapter"; signer: ExternalSigner }
  | { type: "privy"; walletId: string; walletAddress?: string };

/** Local keypair strategy required by explicit wrap and unwrap-all planning. */
export type NativeSigningStrategy = Extract<SigningStrategy, { type: "native" }>;

/**
 * Return the native strategy required by standalone WSOL conversion planning.
 *
 * A wallet-adapter or Privy strategy returns a validation error. Conversion
 * planners call this guard before RPC reads. Ordinary planners do not call it and
 * can build unsigned plans for external strategies. Submission requires verifiable
 * signed v1 bytes, so Privy sign-and-send transaction submission is unavailable.
 */
export function requireNativeSigningStrategy(
  strategy: SigningStrategy
): NativeSigningStrategy {
  if (strategy.type !== "native") {
    throw SdkError.validation(
      "standalone WSOL conversion requires a native signing strategy"
    );
  }
  return strategy;
}

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
