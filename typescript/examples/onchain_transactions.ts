import { PublicKey, Transaction } from "@solana/web3.js";
import {
  restClient,
  rpcClient,
  wallet,
  marketAndOrderbook,
  depositMint,
  numOutcomes,
} from "./common";

function describeTx(name: string, tx: Transaction): void {
  console.log(
    `${name}: ${tx.instructions.length} instruction(s), ${tx.serialize().length} bytes, signature=${tx.signature?.toString("base64") ?? "unsigned"}`
  );
}

async function main() {
  const client = restClient();
  const rpc = rpcClient();
  const keypair = wallet();

  const [m] = await marketAndOrderbook(client);
  const marketPubkey = new PublicKey(m.pubkey);
  const dMint = depositMint(m);
  const outcomes = numOutcomes(m);

  const transactions: Array<[string, Transaction]> = [
    [
      "mint_complete_set",
      (
        await rpc.mintCompleteSet(
          {
            user: keypair.publicKey,
            market: marketPubkey,
            depositMint: dMint,
            amount: 1_000_000n,
          },
          outcomes
        )
      ).transaction,
    ],
    [
      "merge_complete_set",
      (
        await rpc.mergeCompleteSet(
          {
            user: keypair.publicKey,
            market: marketPubkey,
            depositMint: dMint,
            amount: 1_000_000n,
          },
          outcomes
        )
      ).transaction,
    ],
    ["increment_nonce", (await rpc.incrementNonce(keypair.publicKey)).transaction],
  ];

  for (const [name, tx] of transactions) {
    tx.sign(keypair);
    describeTx(name, tx);
    const signature = await rpc.connection.sendRawTransaction(tx.serialize());
    await rpc.connection.confirmTransaction(signature);
    console.log(`${name}: confirmed ${signature}`);
  }
}

main().catch(console.error);
