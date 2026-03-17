import { PublicKey, Transaction } from "@solana/web3.js";
import {
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
  const client = rpcClient();
  const keypair = wallet();
  const connection = client.rpc().inner();

  const [m] = await marketAndOrderbook(client);
  const marketPubkey = new PublicKey(m.pubkey);
  const dMint = depositMint(m);
  const outcomes = numOutcomes(m);

  const { blockhash, lastValidBlockHeight } = await client.rpc().getLatestBlockhash();

  const mintIx = client.markets().mintCompleteSetIx(
    {
      user: keypair.publicKey,
      market: marketPubkey,
      depositMint: dMint,
      amount: 1_000_000n,
    },
    outcomes
  );

  const mergeIx = client.markets().mergeCompleteSetIx(
    {
      user: keypair.publicKey,
      market: marketPubkey,
      depositMint: dMint,
      amount: 1_000_000n,
    },
    outcomes
  );

  const nonceIx = client.orders().incrementNonceIx(keypair.publicKey);

  const transactions: Array<[string, Transaction]> = [
    ["mint_complete_set", new Transaction({ feePayer: keypair.publicKey, blockhash, lastValidBlockHeight }).add(mintIx)],
    ["merge_complete_set", new Transaction({ feePayer: keypair.publicKey, blockhash, lastValidBlockHeight }).add(mergeIx)],
    ["increment_nonce", new Transaction({ feePayer: keypair.publicKey, blockhash, lastValidBlockHeight }).add(nonceIx)],
  ];

  for (const [name, tx] of transactions) {
    tx.sign(keypair);
    describeTx(name, tx);
    const signature = await connection.sendRawTransaction(tx.serialize());
    await connection.confirmTransaction(signature);
    console.log(`${name}: confirmed ${signature}`);
  }
}

main().catch(console.error);
