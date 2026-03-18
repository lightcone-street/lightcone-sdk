import {
  PublicKey,
  Transaction,
  type Keypair,
  type TransactionInstruction,
} from "@solana/web3.js";
import { type LightconeClient } from "../src";
import {
  depositMint,
  login,
  market,
  numOutcomes,
  rpcClient,
  wallet,
} from "./common";

const GLOBAL_DEPOSIT_AMOUNT = 1_000_000n;
const MARKET_DEPOSIT_AMOUNT = 500_000n;

function globalBalanceFor(
  globalDeposits: Array<{ deposit_mint: string; balance: string }>,
  mint: string
): string {
  return globalDeposits.find((deposit) => deposit.deposit_mint === mint)?.balance ?? "0";
}

async function sendAndConfirm(
  client: LightconeClient,
  keypair: Keypair,
  name: string,
  ix: TransactionInstruction
): Promise<void> {
  const connection = client.rpc().inner();
  const { blockhash, lastValidBlockHeight } = await client.rpc().getLatestBlockhash();
  const tx = new Transaction({
    feePayer: keypair.publicKey,
    blockhash,
    lastValidBlockHeight,
  }).add(ix);

  tx.sign(keypair);
  const signature = await connection.sendRawTransaction(tx.serialize());
  await connection.confirmTransaction({ signature, blockhash, lastValidBlockHeight });
  console.log(`${name}: confirmed ${signature}`);
}

async function main() {
  const client = rpcClient();
  const keypair = wallet();
  await login(client, keypair);

  const m = await market(client);
  const marketPubkey = new PublicKey(m.pubkey);
  const dMint = depositMint(m);
  const outcomes = numOutcomes(m);
  const pubkey = keypair.publicKey.toBase58();

  const globalDepositToken = await client.rpc().getGlobalDepositToken(dMint);
  console.log(`market: ${m.slug}`);
  console.log(
    `deposit mint: ${dMint.toBase58()} active=${globalDepositToken.active} index=${globalDepositToken.index}`
  );
  console.log(
    `pdas: global=${client.rpc().getGlobalDepositTokenPda(dMint).toBase58()} user=${client.rpc().getUserGlobalDepositPda(keypair.publicKey, dMint).toBase58()}`
  );

  const depositIx = client.positions().depositToGlobalIx({
    user: keypair.publicKey,
    mint: dMint,
    amount: GLOBAL_DEPOSIT_AMOUNT,
  });
  await sendAndConfirm(client, keypair, "deposit_to_global", depositIx);

  const afterDeposit = await client.positions().get(pubkey);
  console.log(
    `global balance after deposit: ${globalBalanceFor(afterDeposit.global_deposits, dMint.toBase58())}`
  );

  const marketDepositIx = client.positions().globalToMarketDepositIx(
    {
      user: keypair.publicKey,
      market: marketPubkey,
      depositMint: dMint,
      amount: MARKET_DEPOSIT_AMOUNT,
    },
    outcomes
  );
  await sendAndConfirm(client, keypair, "global_to_market_deposit", marketDepositIx);

  const afterMove = await client.positions().getForMarket(pubkey, m.pubkey);
  console.log(`positions in ${m.slug}: ${afterMove.positions.length}`);
  console.log(
    `remaining global balance: ${globalBalanceFor(afterMove.global_deposits, dMint.toBase58())}`
  );
}

main().catch(console.error);
