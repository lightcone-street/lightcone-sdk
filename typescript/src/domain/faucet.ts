import type { PubkeyStr } from "../shared";

/**
 * Testnet faucet — types consumed by `LightconeClient.claim()`.
 *
 * There is no faucet sub-client; the single entry point is the `claim()`
 * method on the root client.
 */

/** Request payload for `POST /api/claim`. */
export interface FaucetRequest {
  wallet_address: PubkeyStr;
}

/** A single token minted to the wallet by the faucet. */
export interface FaucetToken {
  symbol: string;
  amount: number;
}

/** Response from `POST /api/claim`. */
export interface FaucetResponse {
  /** Signature of the on-chain mint transaction. */
  signature: string;
  /** Amount of testnet SOL transferred. */
  sol: number;
  tokens: FaucetToken[];
}
