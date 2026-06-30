// TypeScript example — claim testnet SOL + whitelisted deposit tokens via the
// gentype facade. Testnet-only: succeeds only where the backend faucet is enabled
// (typically local / staging). Run with: npx tsx examples/FaucetClaim.ts
import * as fs from "node:fs";
import * as os from "node:os";
import { createKeyPairFromBytes, getAddressFromPublicKey } from "@solana/kit";
import { makeForEnv, FaucetClient } from "../src/TypeScriptApi.gen.ts";
import type { t as Env_t } from "../src/Env.gen.ts";

function walletSecretKey(): Uint8Array {
  const path = process.env.LIGHTCONE_WALLET_PATH ?? "~/.config/solana/id.json";
  const resolved = path.startsWith("~") ? path.replace("~", os.homedir()) : path;
  return Uint8Array.from(JSON.parse(fs.readFileSync(resolved, "utf-8")) as number[]);
}

async function main(): Promise<void> {
  const env = (process.env.LIGHTCONE_ENV ?? "prod") as Env_t;
  const client = makeForEnv(env);

  // The faucet needs only the wallet's base58 address (no signing).
  const keypair = await createKeyPairFromBytes(walletSecretKey());
  const walletAddress = await getAddressFromPublicKey(keypair.publicKey);

  const result = await FaucetClient.claim(client, walletAddress);
  console.log(`claim tx: ${result.signature}`);
  console.log(`sol: ${result.sol}`);
  for (const token of result.tokens) {
    console.log(`  - ${token.symbol}: ${token.amount}`);
  }
}

main().catch((error: unknown) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
