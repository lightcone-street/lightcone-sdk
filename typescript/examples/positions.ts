import { restClient, wallet, login, market } from "./common";

async function main() {
  const client = restClient();
  const keypair = wallet();
  const user = await login(client, keypair);
  const all = await client.positions().get(user.wallet_address);
  const m = await market(client);
  const perMarket = await client
    .positions()
    .getForMarket(user.wallet_address, m.pubkey);
  console.log("wallet:", user.wallet_address);
  console.log("markets with positions:", all.total_markets);
  console.log(`positions in ${m.slug}: ${perMarket.positions.length}`);
}

main().catch(console.error);
