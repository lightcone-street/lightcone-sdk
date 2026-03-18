import { PublicKey, Transaction } from "@solana/web3.js";
import {
  DepositToGlobalParams,
  ExtendPositionTokensParams,
  GlobalToMarketDepositParams,
  InitPositionTokensParams,
  getPositionAltPda,
  getPositionPda,
} from "../src/program";
import { depositMint, login, market, numOutcomes, rpcClient, wallet } from "./common";

async function main() {
  const client = rpcClient();
  const connection = client.rpc().inner();
  const keypair = wallet();
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

  // 2. Deposit to global — fund the global pool with collateral
  const depositIx = client.positions().depositToGlobalIx({
    user: keypair.publicKey,
    mint: dMint,
    amount,
  });

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

  // 4. Extend position tokens — add a new deposit mint to an existing ALT
  //    Derive the ALT address from the position PDA + the slot used during init
  const [positionPda] = getPositionPda(keypair.publicKey, marketPubkey);
  const [lookupTable] = getPositionAltPda(positionPda, recentSlot);

  const extendIx = client.positions().extendPositionTokensIx(
    {
      payer: keypair.publicKey,
      user: keypair.publicKey,
      market: marketPubkey,
      lookupTable,
      depositMints: [dMint],
    },
    outcomes
  );

  for (const [name, ix] of [
    ["init_position_tokens", initIx],
    ["deposit_to_global", depositIx],
    ["global_to_market_deposit", moveIx],
    ["extend_position_tokens", extendIx],
  ] as const) {
    const { blockhash, lastValidBlockHeight } = await client.rpc().getLatestBlockhash();
    const tx = new Transaction({ feePayer: keypair.publicKey, blockhash, lastValidBlockHeight }).add(ix);
    tx.sign(keypair);
    const signature = await connection.sendRawTransaction(tx.serialize());
    await connection.confirmTransaction(signature);
    console.log(`${name}: confirmed ${signature}`);
  }
}

main().catch(console.error);
