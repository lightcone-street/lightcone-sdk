import { PublicKey, Transaction } from "@solana/web3.js";
import {
  DepositToGlobalParams,
  ExtendPositionTokensParams,
  GlobalToMarketDepositParams,
  InitPositionTokensParams,
} from "../src/program";
import { depositMint, login, market, numOutcomes, rpcClient, wallet } from "./common";

async function sendAndConfirm(
  name: string,
  tx: Transaction
): Promise<void> {
  const signature = await connection.sendRawTransaction(tx.serialize());
  await connection.confirmTransaction(signature);
  console.log(`${name} confirmed: ${signature}`);
}

const client = rpcClient();
const connection = client.rpc().inner();
const keypair = wallet();

async function main() {
  await login(client, keypair);

  const m = await market(client);
  const marketPubkey = new PublicKey(m.pubkey);
  const dMint = depositMint(m);
  const outcomes = numOutcomes(m);
  const amount = 1_000_000n;

  // 1. Init position tokens — one-time setup per market (creates position + ALT)
  const recentSlot = BigInt(await connection.getSlot());
  const initIx = client.positions().initPositionTokensIx(
    {
      payer: keypair.publicKey,
      user: keypair.publicKey,
      market: marketPubkey,
      depositMints: [dMint],
      recentSlot,
    },
    outcomes
  );
  let { blockhash, lastValidBlockHeight } = await client.rpc().getLatestBlockhash();
  let tx = new Transaction({ feePayer: keypair.publicKey, blockhash, lastValidBlockHeight }).add(initIx);
  tx.sign(keypair);
  await sendAndConfirm("init_position_tokens", tx);

  // 2. Deposit to global — fund the global pool with collateral
  const depositIx = client.positions().depositToGlobalIx({
    user: keypair.publicKey,
    mint: dMint,
    amount,
  });
  ({ blockhash, lastValidBlockHeight } = await client.rpc().getLatestBlockhash());
  tx = new Transaction({ feePayer: keypair.publicKey, blockhash, lastValidBlockHeight }).add(depositIx);
  tx.sign(keypair);
  await sendAndConfirm("deposit_to_global", tx);

  // 3. Global to market deposit — move capital into a specific market
  const moveIx = client.positions().globalToMarketDepositIx(
    {
      user: keypair.publicKey,
      market: marketPubkey,
      depositMint: dMint,
      amount,
    },
    outcomes
  );
  ({ blockhash, lastValidBlockHeight } = await client.rpc().getLatestBlockhash());
  tx = new Transaction({ feePayer: keypair.publicKey, blockhash, lastValidBlockHeight }).add(moveIx);
  tx.sign(keypair);
  await sendAndConfirm("global_to_market_deposit", tx);

  // 4. Extend position tokens — add a new deposit mint to an existing ALT
  //    (only needed when a new deposit mint is whitelisted)
  const position = await client.positions().getOnchain(keypair.publicKey, marketPubkey);
  if (!position) throw new Error("position not found");

  const extendIx = client.positions().extendPositionTokensIx(
    {
      payer: keypair.publicKey,
      user: keypair.publicKey,
      market: marketPubkey,
      lookupTable: position.lookupTable,
      depositMints: [dMint],
    },
    outcomes
  );
  ({ blockhash, lastValidBlockHeight } = await client.rpc().getLatestBlockhash());
  tx = new Transaction({ feePayer: keypair.publicKey, blockhash, lastValidBlockHeight }).add(extendIx);
  tx.sign(keypair);
  await sendAndConfirm("extend_position_tokens", tx);
}

main().catch(console.error);
