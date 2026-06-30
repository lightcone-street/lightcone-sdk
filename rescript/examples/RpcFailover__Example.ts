// TypeScript example — configure a (dead) primary RPC + a working backup, then
// call latestBlockhash. Entirely via the gentype facade. Ported from
// rust/examples/rpc_failover.rs.
//
// NOTE: the current Rpc layer uses the primary URL only — automatic failover is a
// documented TODO (see TODO.md / src/Rpc.res). This shows the primary/backup
// configuration and a live blockhash call.
import { make, RpcClient } from "../src/TypeScriptApi.gen.ts";

const deadPrimary = "https://dead-primary.invalid";
const backupRpc = "https://api.devnet.solana.com";

async function main(): Promise<void> {
  // make(env, baseUrl, wsUrl, rpcUrl, backupRpcUrl, programId, depositSource, unit)
  const client = make(undefined, undefined, undefined, deadPrimary, backupRpc, undefined, undefined, undefined);
  console.log(`primary : ${deadPrimary}`);
  console.log(`backup  : ${backupRpc}`);

  try {
    const blockhash = await RpcClient.latestBlockhash(client);
    console.log(`blockhash: ${blockhash}`);
  } catch (error: unknown) {
    console.error(`(expected until failover lands) ${error instanceof Error ? error.message : error}`);
  }
}

main().catch((error: unknown) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
