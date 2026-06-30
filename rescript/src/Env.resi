// Lightcone deployment environment. Public surface: the env enum, the per-env
// URL / program-id resolvers (overridden by `SDK_*` env vars), and string
// conversions. The built-in URL/program-id tables live in the implementation.
@genType
type t =
  | @as("local") Local
  | @as("staging") Staging
  | @as("prod") Prod

// Read an environment variable (None in the browser, where `process` is absent).
let getEnv: string => option<string>

let apiUrl: t => string
let wsUrl: t => string
let rpcUrl: t => string
// Base58 program id for this environment.
let programId: t => string

let toString: t => string
let fromString: string => option<t>
