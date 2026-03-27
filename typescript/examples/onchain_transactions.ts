import { Transaction } from "@solana/web3.js";
import {
  rpcClient,
  wallet,
  marketAndOrderbook,
  depositMint,
} from "./common";

function describeTx(name: string, tx: Transaction): void {
  console.log(
    `${name}: ${tx.instructions.length} instruction(s), ${tx.serialize().length} bytes, signature=${tx.signature?.toString("base64") ?? "unsigned"}`
  );
}

async function main() {
  const client = rpcClient();
  const keypair = wallet();
  const connection = client.rpc().inner();

  const [m] = await marketAndOrderbook(client);
  const dMint = depositMint(m);

  const { blockhash, lastValidBlockHeight } = await client.rpc().getLatestBlockhash();

  const transactions: Array<[string, Transaction]> = [
    [
      "deposit",
      client.positions().deposit()
        .user(keypair.publicKey)
        .mint(dMint)
        .amount(1_000_000n)
        .withMarketDepositSource(m)
        .buildTx(),
    ],
    [
      "merge",
      client.positions().merge()
        .user(keypair.publicKey)
        .market(m)
        .mint(dMint)
        .amount(1_000_000n)
        .buildTx(),
    ],
    ["increment_nonce", client.orders().incrementNonceTx(keypair.publicKey)],
  ];

  for (const [name, tx] of transactions) {
    tx.recentBlockhash = blockhash;
    tx.lastValidBlockHeight = lastValidBlockHeight;
    tx.sign(keypair);
    describeTx(name, tx);
    const signature = await connection.sendRawTransaction(tx.serialize());
    await connection.confirmTransaction(signature);
    console.log(`${name}: confirmed ${signature}`);
  }
}

main().catch((error) => { console.error(error); process.exit(1); });
