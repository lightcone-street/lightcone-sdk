import { Keypair, SystemProgram } from "@solana/web3.js";
import {
  V1Transaction,
  type V1TransactionContext,
} from "../src/program/transaction";

export const TEST_CONTEXT: V1TransactionContext = Object.freeze({
  blockhash: Keypair.fromSeed(Buffer.alloc(32, 9)).publicKey.toBase58(),
  lastValidBlockHeight: 100,
  resources: Object.freeze({
    computeUnitLimit: 200_000,
    loadedAccountsDataSizeLimit: 65_536,
    priorityFeeLamports: 7n,
  }),
});
export function transferTransaction(
  payer = Keypair.fromSeed(Buffer.alloc(32, 1)),
  context = TEST_CONTEXT,
): V1Transaction {
  return V1Transaction.compile(
    [
      SystemProgram.transfer({
        fromPubkey: payer.publicKey,
        toPubkey: Keypair.fromSeed(Buffer.alloc(32, 2)).publicKey,
        lamports: 1n,
      }),
    ],
    payer.publicKey,
    context,
  );
}
