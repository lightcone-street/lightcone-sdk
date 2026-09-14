import { V1Transaction } from "../src";
import { asPubkeyStr } from "../src";
import {
  cancelBodySigned,
  cancelAllBodySigned,
} from "../src/domain/order/client";
import { generateCancelAllSalt, OrderSide } from "../src/program";
import { scalePriceSize } from "../src/shared";
import {
  getKeypair,
  login,
  marketAndOrderbook,
  quoteDepositMint,
  restClient,
  runExample,
  unixTimestamp,
} from "./common";

async function main() {
  const client = restClient();
  const keypair = getKeypair();
  await login(client, keypair);
  const pubkey = keypair.publicKey.toBase58();

  const snapshot = await client.orders().getUserOrders(50);
  const limitOrder = snapshot.orders.find((o) => o.order_type === "limit");

  if (!limitOrder) {
    console.log("No open limit orders to cancel.");
    return;
  }

  const orderHash = limitOrder.order_hash;
  const orderbookId = limitOrder.orderbook_id;

  const cancel = cancelBodySigned(orderHash, asPubkeyStr(pubkey), keypair);
  const cancelled = await client.orders().cancel(cancel);
  console.log(`cancelled: ${cancelled.order_hash} remaining=${cancelled.remaining}`);

  const timestamp = unixTimestamp();
  const salt = generateCancelAllSalt();
  const cancelAll = cancelAllBodySigned(
    asPubkeyStr(pubkey),
    orderbookId,
    timestamp,
    salt,
    keypair
  );
  const cleared = await client.orders().cancelAll(cancelAll);
  console.log(`cancel-all removed ${cleared.count} order(s) in ${cleared.orderbook_id}`);

  // Cleanup: cancelling the order released its locked collateral back into
  // the global pool. Withdraw that amount to the user's token account so the
  // companion `submit_order` → `cancel_order` cycle is net-neutral on the
  // wallet's balance and the global pool.
  const [, orderbook] = await marketAndOrderbook(client);
  const rules = await client.orderbooks().decimals(orderbook.orderbookId);
  const orderQuoteAmount = scalePriceSize(
    rules.tradingRules.priceQuantum,
    "1",
    OrderSide.BID,
    rules
  ).quoteAtoms;
  const mint = quoteDepositMint(orderbook);
  const withdrawIx = client
    .positions()
    .withdrawFromGlobal()
    .user(keypair.publicKey)
    .mint(mint)
    .amount(orderQuoteAmount)
    .buildIx();
  const context = await client.transactionContext();
  const tx = V1Transaction.compile([withdrawIx], keypair.publicKey, context);
  const signed = tx.sign([keypair]);
  const sig = await client.rpc().submitSignedTransaction(signed);
  await client.rpc().confirmSignature(sig, context.lastValidBlockHeight);
  console.log(`withdraw_from_global: confirmed ${sig}`);
}

void runExample(main);
