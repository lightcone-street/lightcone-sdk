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
  console.log(
    `funding accounts: ${all.funding_accounts.length} (revision ${all.committed_revision}, more: ${all.has_more})`
  );
  for (const account of all.funding_accounts) {
    console.log(
      `  ${account.source} ${account.mint} observed=${account.raw_observed ?? "unobserved"} reserved=${account.order_reserved}`
    );
  }
  console.log(`accounts usable in ${m.slug}: ${perMarket.funding_accounts.length}`);
}

void runExample(main);
