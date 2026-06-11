import { tradingWallet } from "../src/auth";
import { restClient, getKeypair, login, runExample } from "./common";

async function main() {
  const client = restClient();
  const keypair = getKeypair();
  const session = await login(client, keypair);
  const wallet = tradingWallet(session.user, session.auth_method);

  const balances = await client.positions().depositTokenBalances();

  console.log("wallet:", wallet);
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
