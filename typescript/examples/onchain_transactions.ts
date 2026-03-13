import { PublicKey } from "@solana/web3.js";
import {
  restClient,
  rpcClient,
  wallet,
  marketAndOrderbook,
  depositMint,
  numOutcomes,
} from "./common";

async function main() {
  const client = restClient();
  const rpc = rpcClient();
  const keypair = wallet();

  const [m] = await marketAndOrderbook(client);
  const marketPubkey = new PublicKey(m.pubkey);
  const dMint = depositMint(m);
  const outcomes = numOutcomes(m);

  // 1. Mint complete set (deposit collateral -> outcome tokens)
  const mintResult = await rpc.mintCompleteSet(
    {
      user: keypair.publicKey,
      market: marketPubkey,
      depositMint: dMint,
      amount: 1_000_000n,
    },
    outcomes
  );
  const mintTx = mintResult.transaction;
  mintTx.sign(keypair);
  console.log(
    "mint complete set:",
    mintTx.instructions.length,
    "instructions,",
    mintTx.serialize().length,
    "bytes"
  );
  console.log("signature:", mintTx.signatures[0]?.signature?.toString("base64") ?? "unsigned");

  // 2. Merge complete set (outcome tokens -> withdraw collateral)
  const mergeResult = await rpc.mergeCompleteSet(
    {
      user: keypair.publicKey,
      market: marketPubkey,
      depositMint: dMint,
      amount: 1_000_000n,
    },
    outcomes
  );
  const mergeTx = mergeResult.transaction;
  mergeTx.sign(keypair);
  console.log(
    "merge complete set:",
    mergeTx.instructions.length,
    "instructions,",
    mergeTx.serialize().length,
    "bytes"
  );
  console.log("signature:", mergeTx.signatures[0]?.signature?.toString("base64") ?? "unsigned");

  // 3. Increment nonce
  const nonceResult = await rpc.incrementNonce(keypair.publicKey);
  const nonceTx = nonceResult.transaction;
  nonceTx.sign(keypair);
  console.log(
    "increment nonce:",
    nonceTx.instructions.length,
    "instructions,",
    nonceTx.serialize().length,
    "bytes"
  );
  console.log("signature:", nonceTx.signatures[0]?.signature?.toString("base64") ?? "unsigned");

  // To actually submit:
  // const txid = await rpc.connection.sendRawTransaction(mintTx.serialize());
  // await rpc.connection.confirmTransaction(txid);
  // console.log("submitted:", txid);
}

main().catch(console.error);
