import { Transaction } from "@solana/web3.js";
import { asPubkeyStr } from "../src";
import { cancelBodySigned } from "../src/domain/order/client";
import { generateSalt } from "../src/program";
import {
  confirmTransactionOrThrow,
  depositMint,
  freshOrderNonce,
  getKeypair,
  login,
  marketAndOrderbook,
  rpcClient,
  runExample,
} from "./common";

// Quote needed for the bid below (price * size, scaled to the deposit asset's
// decimals). Keeping this as a constant so deposit, order, and withdraw stay
// in sync.
const ORDER_QUOTE_AMOUNT = 1_100_000n; // 0.55 * 2 USDC, 6 decimals

async function main() {
  const keypair = getKeypair();
  const client = rpcClient();
  client.setSigningStrategy({ type: "native", keypair });
  await login(client, keypair);

  const [market, orderbook] = await marketAndOrderbook(client);
  const mint = depositMint(market);
  const connection = client.rpc().inner();

  // 1. Deposit collateral into the global pool.
  //
  // submit_order uses the client's default deposit source (Global), so the
  // global pool must cover `price * size` in the deposit asset's base units
  // before the order can be placed.
  const depositIx = client
    .positions()
    .depositToGlobal()
    .user(keypair.publicKey)
    .mint(mint)
    .amount(ORDER_QUOTE_AMOUNT)
    .buildIx();
  {
    const { blockhash, lastValidBlockHeight } = await client.rpc().getLatestBlockhash();
    const tx = new Transaction({
      feePayer: keypair.publicKey,
      blockhash,
      lastValidBlockHeight,
    }).add(depositIx);
    tx.sign(keypair);
    const sig = await connection.sendRawTransaction(tx.serialize());
    await confirmTransactionOrThrow(connection, sig, { blockhash, lastValidBlockHeight });
    console.log(`deposit_to_global: confirmed ${sig}`);
  }

  // 2. Submit the limit order. Fetch and cache the on-chain nonce once —
  //    subsequent orders that omit `.nonce()` use this cached value.
  const nonce = await freshOrderNonce(client, keypair.publicKey);
  client.setOrderNonce(nonce);

  const response = await client
    .orders()
    .limitOrder()
    .maker(keypair.publicKey)
    .bid()
    .price("0.55")
    .size("2")
    .salt(generateSalt())
    .submit(client, orderbook);
  console.log(
    `submitted: ${response.order_hash} filled=${response.filled} remaining=${response.remaining} fills=${response.fills.length}`
  );

  // 3. Clean up so the example is net-neutral on the wallet's token balance.
  //    Cancel the open order to release the locked collateral, then withdraw
  //    it from the global pool back to the user's token account.
  const cancel = cancelBodySigned(
    response.order_hash,
    asPubkeyStr(keypair.publicKey.toBase58()),
    keypair
  );
  const cancelled = await client.orders().cancel(cancel);
  console.log(`cancelled: ${cancelled.order_hash} remaining=${cancelled.remaining}`);

  const withdrawIx = client
    .positions()
    .withdrawFromGlobal()
    .user(keypair.publicKey)
    .mint(mint)
    .amount(ORDER_QUOTE_AMOUNT)
    .buildIx();
  {
    const { blockhash, lastValidBlockHeight } = await client.rpc().getLatestBlockhash();
    const tx = new Transaction({
      feePayer: keypair.publicKey,
      blockhash,
      lastValidBlockHeight,
    }).add(withdrawIx);
    tx.sign(keypair);
    const sig = await connection.sendRawTransaction(tx.serialize());
    await confirmTransactionOrThrow(connection, sig, { blockhash, lastValidBlockHeight });
    console.log(`withdraw_from_global: confirmed ${sig}`);
  }
}

void runExample(main);
