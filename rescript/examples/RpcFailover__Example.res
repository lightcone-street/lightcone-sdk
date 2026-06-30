// Configures a client with a (dead) primary RPC + a working backup, then calls
// getLatestBlockhash. Ported from rust/examples/rpc_failover.rs.
//
// NOTE: the current Rpc layer uses the primary URL only — automatic failover to
// the backup is a documented TODO (see `// TODO(failover)` in src/Rpc.res). This
// example shows the primary/backup configuration and a live blockhash call; once
// failover lands it will transparently fall back to the backup when the primary
// is unreachable. ReScript surface (result core).
let deadPrimary = "https://dead-primary.invalid"
let backupRpc = "https://api.devnet.solana.com"

let main = async () => {
  let client = Client.make(~rpcUrl=deadPrimary, ~backupRpcUrl=backupRpc, ())
  Console.log(`primary : ${client.rpcUrl}`)
  Console.log(`backup  : ${client.backupRpcUrl->Option.getOr("(none)")}`)

  switch await Rpc.getLatestBlockhash(client) {
  | Ok(blockhash) => Console.log(`blockhash: ${blockhash}`)
  | Error(error) => Console.error(`(expected until failover lands) ${SdkError.toMessage(error)}`)
  }
}

let _ = main()
