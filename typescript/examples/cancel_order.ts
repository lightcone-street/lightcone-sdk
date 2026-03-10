import { asPubkeyStr, asOrderBookId } from "../src/shared/types";
import { signCancelOrder, signCancelAll } from "../src/program/orders";
import { restClient, wallet, login, marketAndOrderbook } from "./common";

async function main() {
  const client = restClient();
  const keypair = wallet();

  // 1. Login
  await login(client, keypair);
  const pubkey = keypair.publicKey.toBase58();
  console.log("logged in:", pubkey);

  // 2. Find an open limit order
  const snapshot = await client.orders().getUserOrders(pubkey, 50);
  const limitOrder = snapshot.orders.find((o) => o.order_type === "limit");

  if (!limitOrder) {
    console.log("no open limit orders to cancel");
    return;
  }

  const orderHash = limitOrder.order_hash;
  const orderbookId = limitOrder.orderbook_id;
  console.log("cancelling order:", orderHash);

  // 3. Cancel a single order
  const signature = signCancelOrder(orderHash, keypair);
  const cancelResult = await client.orders().cancel({
    order_hash: orderHash,
    maker: asPubkeyStr(pubkey),
    signature,
  });
  console.log("cancelled:", cancelResult.order_hash);
  console.log("remaining:", cancelResult.remaining);

  // 4. Cancel all orders in an orderbook
  const timestamp = Math.floor(Date.now() / 1000);
  const cancelAllSig = signCancelAll(pubkey, timestamp, keypair);
  const cancelAllResult = await client.orders().cancelAll({
    user_pubkey: asPubkeyStr(pubkey),
    orderbook_id: asOrderBookId(orderbookId),
    signature: cancelAllSig,
    timestamp,
  });
  console.log("cancel all count:", cancelAllResult.count);
  console.log("orderbook:", cancelAllResult.orderbook_id);
}

main().catch(console.error);
