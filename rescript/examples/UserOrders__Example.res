// Authenticated example — the signed-in user's open orders snapshot. Ported from
// the Rust `examples/user_orders.rs`. The backend returns limit + trigger orders in
// one array discriminated by `order_type` ("limit" | "trigger"). ReScript surface:
// log in, then read `Order.Client.getUserOrders`; the compiled UserOrders.res.mjs is the
// JS example.
let main = async () => {
  let client = Common__Example.client()
  await Client.useNativeSigner(client, Common__Example.walletSecretKey())

  switch await Auth.Client.login(client) {
  | Error(error) => Console.error(SdkError.toMessage(error))
  | Ok(_) =>
    switch await Order.Client.getUserOrders(client, ~limit=10) {
    | Ok(snapshot) =>
      let (limitOrders, triggerOrders) = snapshot.orders->Array.reduce((0, 0), (
        (limits, triggers),
        order,
      ) =>
        switch order {
        | Order.Raw.SnapshotOrder.Trigger(_) => (limits, triggers + 1)
        | Limit(_) => (limits + 1, triggers)
        }
      )
      Console.log(
        `orders: ${Int.toString(limitOrders)} limit / ${Int.toString(triggerOrders)} trigger`,
      )
      Console.log(`market balances: ${Int.toString(Array.length(snapshot.marketBalances))}`)
      Console.log(`has more: ${snapshot.hasMore ? "yes" : "no"}`)

      switch snapshot.orders[0] {
      | Some(Order.Raw.SnapshotOrder.Trigger(order)) =>
        Console.log(
          `first trigger: ${order.triggerOrderId} ${Shared.Side.toString(order.common.side)} ` ++
          `@ ${order.common.price} (trigger ${order.triggerPrice})`,
        )
      | Some(Limit(order)) =>
        Console.log(
          `first limit: ${order.common.orderHash} ${Shared.Side.toString(
              order.common.side,
            )} @ ${order.common.price}`,
        )
      | None => ()
      }

      // Follow the cursor once, if the backend paginated the snapshot.
      switch snapshot.nextCursor {
      | Some(cursor) =>
        switch await Order.Client.getUserOrders(client, ~limit=10, ~cursor) {
        | Ok(next) => Console.log(`next page: ${Int.toString(Array.length(next.orders))} order(s)`)
        | Error(error) => Console.error(SdkError.toMessage(error))
        }
      | None => ()
      }
    | Error(error) => Console.error(SdkError.toMessage(error))
    }
  }
}

let _ = main()
