import { V1Transaction } from "../src";
import {
  rpcClient,
  getKeypair,
  marketAndOrderbook,
  quoteDepositMint,
  runExample,
} from "./common";

function describeTx(name: string, tx: V1Transaction): void {
  console.log(
    `${name}: ${tx.instructions.length} instruction(s), ${tx.toWireBytes().length} bytes, signature=${tx.signature}`
  );
}

async function main() {
  const client = rpcClient();
  const keypair = getKeypair();

  const [m, ob] = await marketAndOrderbook(client);
  const dMint = quoteDepositMint(ob);

  const context = await client.transactionContext();

  const transactions: Array<[string, V1Transaction]> = [
    [
      "deposit",
      client.positions().deposit()
        .user(keypair.publicKey)
        .mint(dMint)
        .amount(1_000_000n)
        .withMarketDepositSource(m)
        .buildTx(context),
    ],
    [
      "merge",
      client.positions().merge()
        .user(keypair.publicKey)
        .market(m)
        .mint(dMint)
        .amount(1_000_000n)
        .buildTx(context),
    ],
    ["increment_nonce", client.orders().incrementNonceTx(keypair.publicKey, context)],
  ];

  for (const [name, tx] of transactions) {
    const signed = tx.sign([keypair]);
    describeTx(name, signed);
    const signature = await client.rpc().submitSignedTransaction(signed);
    await client.rpc().confirmSignature(signature, context.lastValidBlockHeight);
    console.log(`${name}: confirmed ${signature}`);
  }
}

void runExample(main);
