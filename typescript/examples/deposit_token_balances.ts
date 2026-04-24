import { restClient, getKeypair, login, runExample } from "./common";

async function main() {
  const client = restClient();
  const keypair = getKeypair();
  const user = await login(client, keypair);

  const balances = await client.positions().depositTokenBalances();

  console.log("wallet:", user.wallet_address);
  console.log("tracked balances:", Object.keys(balances).length);

  const entries = Object.values(balances).sort((a, b) =>
    a.symbol.localeCompare(b.symbol),
  );
  for (const balance of entries) {
    console.log(
      `  ${balance.symbol.padStart(8)}  ${balance.mint.padEnd(42)}  idle=${balance.idle}`,
    );
  }
}

void runExample(main);
