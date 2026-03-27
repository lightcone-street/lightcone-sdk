import { asPubkeyStr } from "../src";
import {
  cancelBodySigned,
  cancelAllBodySigned,
} from "../src/domain/order/client";
import { generateCancelAllSalt } from "../src/program";
import { login, restClient, unixTimestamp, wallet } from "./common";

async function main() {
  const client = restClient();
  const keypair = wallet();
  await login(client, keypair);
  const pubkey = keypair.publicKey.toBase58();

  const snapshot = await client.orders().getUserOrders(pubkey, 50);
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
}

main().catch((error) => { console.error(error); process.exit(1); });
