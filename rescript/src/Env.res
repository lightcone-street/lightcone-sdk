// Lightcone deployment environment.
// Each variant maps to an API URL, WebSocket URL, Solana RPC URL, and program id.
// The `SDK_*` environment variables override the built-ins when set.
//
// `@as` makes the runtime value + the gentype TS union lowercase ("local" /
// "staging" / "prod"), matching the TS SDK's `LightconeEnv` enum values.

type t =
  | @as("local") Local
  | @as("staging") Staging
  | @as("prod") Prod

// Read an environment variable, working in Node/Bun and degrading to None in the
// browser (where `process` is absent).
let getEnv: string => option<string> = %raw(`function (key) {
  return (typeof process !== "undefined" && process.env) ? process.env[key] : undefined;
}`)

let apiUrl = (env: t): string =>
  switch getEnv("SDK_API_URL") {
  | Some(url) => url
  | None =>
    switch env {
    | Local => "https://api.local.lightcone.xyz"
    | Staging => "https://api.staging.lightcone.xyz"
    | Prod => "https://api.lightcone.xyz"
    }
  }

let wsUrl = (env: t): string =>
  switch getEnv("SDK_WS_URL") {
  | Some(url) => url
  | None =>
    switch env {
    | Local => "wss://ws.local.lightcone.xyz/ws"
    | Staging => "wss://ws.staging.lightcone.xyz/ws"
    | Prod => "wss://ws.lightcone.xyz/ws"
    }
  }

let rpcUrl = (env: t): string =>
  switch getEnv("SDK_RPC_URL") {
  | Some(url) => url
  | None =>
    switch env {
    | Local => "https://api.devnet.solana.com"
    | Staging => "https://api.devnet.solana.com"
    | Prod => "https://api.mainnet-beta.solana.com"
    }
  }

// Base58 program id for this environment (parsed into a `SolanaKit.address` at use sites).
let programId = (env: t): string =>
  switch getEnv("SDK_PROGRAM_ID") {
  | Some(id) => id
  | None =>
    switch env {
    | Local => "HQZW84F7WbpDLDdd6eaDsBh6LjDQ2uCxpkZgkLakcago"
    | Staging => "5G2fWZGHB5BA8gbABVBuR1bU4Ziri9cRxFoojz5C5Rxk"
    | Prod => "B9rCvafkkjh749284jfDu5UB268pHeRLkzFpFf7t4mxK"
    }
  }

let toString = (env: t): string =>
  switch env {
  | Local => "local"
  | Staging => "staging"
  | Prod => "prod"
  }

let fromString = (raw: string): option<t> =>
  switch String.toLowerCase(raw) {
  | "local" => Some(Local)
  | "staging" => Some(Staging)
  | "prod" => Some(Prod)
  | _ => None
  }
