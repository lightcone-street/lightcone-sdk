// Configures a client with a (dead) primary RPC + a working backup, then calls
// getLatestBlockhash twice — transparently failing over to the backup on the dead
// primary, then serving the next call straight from the backup with no retry delay.
// Ported from rust/examples/rpc_failover.rs. ReScript surface (result core).
let deadPrimary = "https://dead-primary.invalid"
let backupRpc = "https://devnet.helius-rpc.com/?api-key=55558885-9601-4d35-a25a-55af783fce2b"

let main = async () => {
  let client = Client.make(~rpcUrl=deadPrimary, ~backupRpcUrl=backupRpc, ())
  Console.log(`primary : ${client.rpcUrl}`)
  Console.log(`backup  : ${client.backupRpcUrl->Option.getOr("(none)")}`)
  Console.log(`active  : ${Rpc.activeRpc(client)->RpcFailover.toString}`)

  // Call #1: the dead primary fails → 100ms fast retry → fail over to the backup.
  switch await Rpc.getLatestBlockhash(client) {
  | Ok(blockhash) =>
    Console.log(`call #1: ${blockhash} (now active: ${Rpc.activeRpc(client)->RpcFailover.toString})`)
  | Error(error) => Console.error(SdkError.toMessage(error))
  }

  // Call #2: state is on the backup now → straight there, no retry delay.
  switch await Rpc.getLatestBlockhash(client) {
  | Ok(blockhash) =>
    Console.log(`call #2: ${blockhash} (active: ${Rpc.activeRpc(client)->RpcFailover.toString})`)
  | Error(error) => Console.error(SdkError.toMessage(error))
  }
}

let _ = main()
