import { restClient, getKeypair, login, runExample } from "./common";

async function main() {
  const client = restClient();
  const keypair = getKeypair();
  await login(client, keypair);

  const snapshot = await client.orders().getUserOrders(50);
  const limitOrders = snapshot.orders.filter(
    (order) => order.order_type === "limit"
  );

  console.log(`orders: ${limitOrders.length} limit`);
  console.log(`market balances: ${snapshot.market_balances.length}`);
  console.log(`has more: ${snapshot.has_more}`);

  const firstOrder = limitOrders[0];
  if (firstOrder) {
    console.log(
      `first limit: ${firstOrder.order_hash} ${firstOrder.side} @ ${firstOrder.price}`
    );
  }

  if (snapshot.has_more && snapshot.next_cursor) {
    const page2 = await client
      .orders()
      .getUserOrders(50, snapshot.next_cursor);
    console.log(`next page: ${page2.orders.length} order(s)`);
  }
}

void runExample(main);
