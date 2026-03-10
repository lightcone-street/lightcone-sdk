import { restClient, wallet, login } from "./common";

async function main() {
  const client = restClient();
  const keypair = wallet();

  // Login required
  await login(client, keypair);
  const pubkey = keypair.publicKey.toBase58();
  console.log("logged in:", pubkey);

  // 1. First page of user orders
  const snapshot = await client.orders().getUserOrders(pubkey, 50);
  console.log("orders:", snapshot.orders.length);
  console.log("balances:", snapshot.balances.length);

  for (const order of snapshot.orders) {
    if (order.order_type === "limit") {
      console.log(
        `  [limit] ${order.common.order_hash} ${order.common.side} @ ${order.common.price}`
      );
    } else {
      console.log(
        `  [trigger] ${order.trigger_order_id} ${order.common.side} @ ${order.common.price} trigger=${order.trigger_price}`
      );
    }
  }

  // 2. Pagination
  if (snapshot.has_more && snapshot.next_cursor) {
    const page2 = await client
      .orders()
      .getUserOrders(pubkey, 50, snapshot.next_cursor);
    console.log("page 2 orders:", page2.orders.length);
  } else {
    console.log("no more pages");
  }
}

main().catch(console.error);
