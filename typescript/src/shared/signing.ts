import type { Keypair } from "@solana/web3.js";

export interface ExternalSigner {
  signMessage(message: Uint8Array): Promise<Uint8Array>;
  signTransaction(txBytes: Uint8Array): Promise<Uint8Array>;
}

export type SigningStrategy =
  | { type: "native"; keypair: Keypair }
  | { type: "walletAdapter"; signer: ExternalSigner }
  | { type: "privy"; walletId: string };

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
