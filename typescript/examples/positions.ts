import { restClient, wallet, login, market } from "./common";

async function main() {
  const client = restClient();
  const keypair = wallet();

  // Login required for positions
  const user = await login(client, keypair);
  console.log("logged in:", user.wallet_address);

  // 1. All positions across markets
  const all = await client.positions().get(user.wallet_address);
  console.log("positions:", JSON.stringify(all, null, 2));

  // 2. Positions for a specific market
  const m = await market(client);
  const forMarket = await client
    .positions()
    .getForMarket(user.wallet_address, m.pubkey);
  console.log("market positions:", JSON.stringify(forMarket, null, 2));
}

main().catch(console.error);
