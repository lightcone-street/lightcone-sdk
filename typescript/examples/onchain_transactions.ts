import { Transaction } from "@solana/web3.js";
import {
  confirmTransactionOrThrow,
  rpcClient,
  getKeypair,
  marketAndOrderbook,
  quoteDepositMint,
  runExample,
} from "./common";

function describeTx(name: string, tx: Transaction): void {
  console.log(
    `${name}: ${tx.instructions.length} instruction(s), ${tx.serialize().length} bytes, signature=${tx.signature?.toString("base64") ?? "unsigned"}`
  );
}

async function main() {
  const client = rpcClient();
  const keypair = getKeypair();
  const connection = client.rpc().inner();

  const [m, ob] = await marketAndOrderbook(client);
  const dMint = quoteDepositMint(ob);

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
    await confirmTransactionOrThrow(connection, signature, {
      blockhash,
      lastValidBlockHeight,
    });
    console.log(`${name}: confirmed ${signature}`);
  }
}

void runExample(main);
