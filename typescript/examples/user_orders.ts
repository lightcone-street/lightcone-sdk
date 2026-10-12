import { restClient, getKeypair, login, runExample } from "./common";

async function main() {
  const client = restClient();
  const keypair = getKeypair();
  await login(client, keypair);

  const snapshot = await client.orders().getUserOrders(50);
  console.log(
    `orders: ${snapshot.orders.length} (revision ${snapshot.committed_revision}, more: ${snapshot.has_more})`
  );
  console.log(`funding accounts: ${snapshot.funding_accounts.length}`);

  const firstOrder = snapshot.orders[0];
  if (firstOrder) {
    console.log(
      `first order: ${firstOrder.order_hash} ${firstOrder.side} @ ${firstOrder.price} open=${firstOrder.state?.open_base ?? "unknown"} tif=${firstOrder.tif ?? "unknown"}`
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
