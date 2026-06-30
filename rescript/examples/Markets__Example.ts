// TypeScript example — consumes the gentype-exported facade, exactly as a TS user
// migrating from the old TS SDK would. Run with: npx tsx examples/ts/markets.ts
import { makeForEnv, MarketClient } from "../src/TypeScriptApi.gen.ts";
import type { t as Env_t } from "../src/Env.gen.ts";

async function main(): Promise<void> {
  const env = (process.env.LIGHTCONE_ENV ?? "prod") as Env_t;
  const client = makeForEnv(env);

  const featured = await MarketClient.featured(client);
  console.log(`Featured markets: ${featured.length}`);

  const page = await MarketClient.get(client, undefined, 5);
  console.log(`First ${page.markets.length} markets:`);
  for (const m of page.markets) {
    console.log(`  ${m.slug} — ${m.name}`);
  }
}

main().catch((error: unknown) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
