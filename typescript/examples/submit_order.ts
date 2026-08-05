import { Transaction } from "@solana/web3.js";
import { generateSalt, OrderSide } from "../src/program";
import { scalePriceSize } from "../src/shared";
import {
  confirmTransactionOrThrow,
  freshOrderNonce,
  getKeypair,
  login,
  marketAndOrderbook,
  quoteDepositMint,
  rpcClient,
  runExample,
  waitForGlobalBalance,
} from "./common";

async function main() {
  const keypair = getKeypair();
  const client = rpcClient();
  client.setSigningStrategy({ type: "native", keypair });
  await login(client, keypair);

  const [market, orderbook] = await marketAndOrderbook(client);
  const rules = await client.orderbooks().decimals(orderbook.orderbookId);
  const orderPrice = rules.tradingRules.priceQuantum;
  const orderSize = "1";
  const orderQuoteAmount = scalePriceSize(
    orderPrice,
    orderSize,
    OrderSide.BID,
    rules
  ).quoteAtoms;
  const requiredBalance = Number(orderQuoteAmount) / 10 ** rules.quoteDecimals;
  const mint = quoteDepositMint(orderbook);
  const connection = client.rpc().inner();

  // 1. Deposit collateral into the global pool.
  //
  // submit_order uses the client's default deposit source (Global), so the
  // global pool must cover `price * size` in the deposit asset's base units
  // before the order can be placed. The companion `cancel_order` example
  // cancels this order and withdraws the same amount back to the user's
  // token account, keeping the deposit/submit/cancel/withdraw cycle
  // net-neutral across CI runs.
  const depositIx = client
    .positions()
    .depositToGlobal()
    .user(keypair.publicKey)
    .mint(mint)
    .amount(orderQuoteAmount)
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

  await waitForGlobalBalance(client, mint, requiredBalance);

  // 2. Submit the limit order. Fetch and cache the on-chain nonce once —
  //    subsequent orders that omit `.nonce()` use this cached value.
  const nonce = await freshOrderNonce(client, keypair.publicKey);
  client.setOrderNonce(nonce);

  const response = await client
    .orders()
    .limitOrder()
    .maker(keypair.publicKey)
    .bid()
    .price(orderPrice)
    .size(orderSize)
    .salt(generateSalt())
    .submit(client, orderbook);
  console.log(
    `submitted: ${response.order_hash} status=${response.status} filled=${response.filled} remaining=${response.remaining} fills=${response.fills.length}`
  );
}

void runExample(main);
