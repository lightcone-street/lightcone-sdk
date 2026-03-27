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
      return "https://local-api.lightcone.xyz";
    case LightconeEnv.Staging:
      return "https://tapi2.lightcone.xyz";
    case LightconeEnv.Prod:
      return "https://tapi.lightcone.xyz";
  }
}

/** WebSocket URL for the given environment. */
export function wsUrl(environment: LightconeEnv): string {
  switch (environment) {
    case LightconeEnv.Local:
      return "wss://local-ws.lightcone.xyz/ws";
    case LightconeEnv.Staging:
      return "wss://tws2.lightcone.xyz/ws";
    case LightconeEnv.Prod:
      return "wss://tws.lightcone.xyz/ws";
  }
}

/** Solana RPC URL for the given environment. */
export function rpcUrl(environment: LightconeEnv): string {
  switch (environment) {
    case LightconeEnv.Local:
      return "https://api.devnet.solana.com";
    case LightconeEnv.Staging:
      return "https://api.devnet.solana.com";
    case LightconeEnv.Prod:
      return "https://api.devnet.solana.com";
  }
}

/** On-chain Lightcone program ID for the given environment. */
export function programId(environment: LightconeEnv): PublicKey {
  switch (environment) {
    case LightconeEnv.Local:
    case LightconeEnv.Staging:
      return new PublicKey("H3qkHTWUDUUw4ZvGNPdwdU4CYqks69bijo1CzVR12mq");
    case LightconeEnv.Prod:
      return new PublicKey("8nzsoyHZFYig3uN3M717Q47MtLqzx2V2UAKaPTqDy5rV");
  }
}
