// Shared example helpers — mirror the other SDKs' `common` module.
// Client env comes from LIGHTCONE_ENV; the wallet keypair from
// LIGHTCONE_WALLET_PATH (a Solana id.json: a JSON array of 64 bytes).

@module("node:fs") external readFileSync: (string, string) => string = "readFileSync"
@module("node:os") external homedir: unit => string = "homedir"

let client = (): Client.t => {
  let env = Env.getEnv("LIGHTCONE_ENV")->Option.flatMap(Env.fromString)->Option.getOr(Env.Prod)
  // Explicit example policy; real applications choose limits for their workload.
  let transactionResources: V1Transaction.Resources.t = {
    computeUnitLimit: 1400000,
    loadedAccountsDataSizeLimit: 67108864,
    priorityFeeLamports: 0n,
  }
  Client.make(~env, ~transactionResources, ())
}

let secretKeyFromJson: string => Uint8Array.t = %raw(`(raw) => Uint8Array.from(JSON.parse(raw))`)

let walletSecretKey = (): Uint8Array.t => {
  let path = Env.getEnv("LIGHTCONE_WALLET_PATH")->Option.getOr("~/.config/solana/id.json")
  let resolved = String.startsWith(path, "~") ? String.replace(path, "~", homedir()) : path
  secretKeyFromJson(readFileSync(resolved, "utf-8"))
}
