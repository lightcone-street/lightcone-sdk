import { tradingWallet } from "../src/auth";
import { restClient, getKeypair, login, market, runExample } from "./common";

async function main() {
  const client = restClient();
  const keypair = getKeypair();
  const session = await login(client, keypair);
  const wallet = tradingWallet(session.user, session.auth_method);
  const all = await client.positions().get(wallet);
  const m = await market(client);
  const perMarket = await client.positions().getForMarket(wallet, m.pubkey);
  console.log("wallet:", wallet);
  console.log("markets with positions:", all.total_markets);
  console.log(`positions in ${m.slug}: ${perMarket.positions.length}`);
}

void runExample(main);
