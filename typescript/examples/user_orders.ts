import { restClient, wallet, login } from "./common";

async function main() {
  const client = restClient();
  const keypair = wallet();
  await login(client, keypair);
  const pubkey = keypair.publicKey.toBase58();

  const snapshot = await client.orders().getUserOrders(pubkey, 50);
  const counts = snapshot.orders.reduce<[number, number]>(
    (acc, order) =>
      order.order_type === "limit"
        ? [acc[0] + 1, acc[1]]
        : [acc[0], acc[1] + 1],
    [0, 0]
  );

  console.log(`orders: ${counts[0]} limit / ${counts[1]} trigger`);
  console.log(
    `balances: ${snapshot.balances.length} market / ${snapshot.global_deposits.length} global`
  );
  console.log(`has more: ${snapshot.has_more}`);

  const firstOrder = snapshot.orders[0];
  if (firstOrder) {
    if (firstOrder.order_type === "limit") {
      console.log(
        `first limit: ${firstOrder.order_hash} ${firstOrder.side} @ ${firstOrder.price}`
      );
    } else {
      console.log(
        `first trigger: ${firstOrder.trigger_order_id} ${firstOrder.side} @ ${firstOrder.price} (trigger ${firstOrder.trigger_price})`
      );
    }
  }

  if (snapshot.has_more && snapshot.next_cursor) {
    const page2 = await client
      .orders()
      .getUserOrders(pubkey, 50, snapshot.next_cursor);
    console.log(`next page: ${page2.orders.length} order(s)`);
  }
}

main().catch(console.error);
