// TypeScript example — configure a (dead) primary RPC + a working backup, then call
// latestBlockhash twice: the dead primary transparently fails over to the backup,
// then the next call is served straight from the backup. Entirely via the gentype
// facade. Ported from rust/examples/rpc_failover.rs.
import { make, RpcClient } from "../src/TypeScriptApi.gen.ts";

const deadPrimary = "https://dead-primary.invalid";
const backupRpc = "https://api.devnet.solana.com";

async function main(): Promise<void> {
  // make(env, baseUrl, wsUrl, rpcUrl, backupRpcUrl, programId, depositSource, unit)
  const client = make(undefined, undefined, undefined, deadPrimary, backupRpc, undefined, undefined, undefined);
  console.log(`primary : ${deadPrimary}`);
  console.log(`backup  : ${backupRpc}`);
  console.log(`active  : ${RpcClient.activeRpc(client)}`);

  // Call #1: dead primary → 100ms fast retry → fail over to the backup.
  const first = await RpcClient.latestBlockhash(client);
  console.log(`call #1: ${first} (now active: ${RpcClient.activeRpc(client)})`);

  // Call #2: state is on the backup now → straight there, no retry delay.
  const second = await RpcClient.latestBlockhash(client);
  console.log(`call #2: ${second} (active: ${RpcClient.activeRpc(client)})`);
}

main().catch((error: unknown) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
