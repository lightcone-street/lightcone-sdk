// TypeScript example — the signed-in user's SPL deposit-token balances via the
// gentype facade. Authenticate with a wallet keypair first; the facade returns a
// throwing Promise of a `{ [mint]: depositTokenBalance }` map.
// Run with: npx tsx examples/DepositTokenBalances.ts
import * as fs from "node:fs";
import * as os from "node:os";
import { makeForEnv, useNativeSigner, AuthClient, PositionClient } from "../src/TypeScriptApi.gen.ts";
import type { t as Env_t } from "../src/Env.gen.ts";

function walletSecretKey(): Uint8Array {
  const path = process.env.LIGHTCONE_WALLET_PATH ?? "~/.config/solana/id.json";
  const resolved = path.startsWith("~") ? path.replace("~", os.homedir()) : path;
  return Uint8Array.from(JSON.parse(fs.readFileSync(resolved, "utf-8")) as number[]);
}

async function main(): Promise<void> {
  const env = (process.env.LIGHTCONE_ENV ?? "prod") as Env_t;
  const client = makeForEnv(env);

  await useNativeSigner(client, walletSecretKey());
  await AuthClient.login(client, undefined);

  const balances = await PositionClient.depositTokenBalances(client);
  const entries = Object.values(balances);
  console.log(`tracked balances: ${entries.length}`);

  for (const balance of entries.sort((a, b) => a.symbol.localeCompare(b.symbol))) {
    console.log(`  ${balance.symbol}  ${balance.mint}  idle=${balance.idle}`);
  }
}

main().catch((error: unknown) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
