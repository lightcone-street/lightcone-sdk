import { PublicKey } from "@solana/web3.js";

/**
 * Lightcone deployment environment.
 *
 * Pass to `LightconeClientBuilder.env()` to configure the client for a
 * specific deployment. Defaults to `Prod` when not specified.
 *
 * @example
 * ```ts
 * const client = LightconeClient.builder()
 *   .env(LightconeEnv.Staging)
 *   .build();
 * ```
 */
export enum LightconeEnv {
  Local = "local",
  Staging = "staging",
  Prod = "prod",
}

/** REST API base URL for the given environment. */
export function apiUrl(environment: LightconeEnv): string {
  switch (environment) {
    case LightconeEnv.Local:
      return "https://api.local.lightcone.xyz";
    case LightconeEnv.Staging:
      return "https://api.staging.lightcone.xyz";
    case LightconeEnv.Prod:
      return "https://api.lightcone.xyz";
  }
}

/** WebSocket URL for the given environment. */
export function wsUrl(environment: LightconeEnv): string {
  switch (environment) {
    case LightconeEnv.Local:
      return "wss://ws.local.lightcone.xyz/ws";
    case LightconeEnv.Staging:
      return "wss://ws.staging.lightcone.xyz/ws";
    case LightconeEnv.Prod:
      return "wss://ws.lightcone.xyz/ws";
  }
}

/**
 * Solana RPC URL for the given environment.
 *
 * If the `SDK_RPC_URL` environment variable is set, its value is used
 * regardless of the selected environment.
 */
export function rpcUrl(environment: LightconeEnv): string {
  const overrideUrl = typeof process !== "undefined" ? process.env.SDK_RPC_URL : undefined;
  if (overrideUrl) return overrideUrl;
  switch (environment) {
    case LightconeEnv.Local:
      return "https://api.devnet.solana.com";
    case LightconeEnv.Staging:
      return "https://api.devnet.solana.com";
    case LightconeEnv.Prod:
      return "https://api.mainnet-beta.solana.com";
  }
}

/**
 * On-chain Lightcone program ID for the given environment.
 *
 * If the `SDK_PROGRAM_ID` environment variable is set, its value is used
 * regardless of the selected environment.
 */
export function programId(environment: LightconeEnv): PublicKey {
  const override_id = typeof process !== "undefined" ? process.env.SDK_PROGRAM_ID : undefined;
  if (override_id) {
    return new PublicKey(override_id);
  }
  switch (environment) {
    case LightconeEnv.Local:
      return new PublicKey("HQZW84F7WbpDLDdd6eaDsBh6LjDQ2uCxpkZgkLakcago");
    case LightconeEnv.Staging:
      return new PublicKey("FAq4NbwPVWNzoaNjcJGhWz4VFT5CbdysLPo7ZWWiWuuE");
    case LightconeEnv.Prod:
      return new PublicKey("B9rCvafkkjh749284jfDu5UB268pHeRLkzFpFf7t4mxK");
  }
}

/**
 * Default program ID (production). Used as the default argument in PDA and
 * instruction helper functions. When targeting a non-production environment,
 * always pass `programId` explicitly via `LightconeClient.programId` or
 * `programId(env)`.
 */
export const PROGRAM_ID = programId(LightconeEnv.Prod);
