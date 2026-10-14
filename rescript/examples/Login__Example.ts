// TypeScript example — authenticate with a wallet keypair via the gentype facade.
// Run with: npx tsx examples/ts/login.ts
import * as fs from "node:fs";
import * as os from "node:os";
import { makeForEnv, useNativeSigner, AuthClient } from "../src/TypeScriptApi.gen.ts";
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
  const session = await AuthClient.login(client, undefined);
  console.log(`Logged in as user ${session.user.userId}`);
  console.log(`login method: ${session.authMethod}, beta: ${session.isBeta}`);
}

main().catch((error: unknown) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
