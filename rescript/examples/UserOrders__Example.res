let main = async () => {
  let client = Common__Example.client()
  await Client.useNativeSigner(client, Common__Example.walletSecretKey())

  switch await Auth.Client.login(client) {
  | Error(error) => Console.error(SdkError.toMessage(error))
  | Ok(_) =>
    switch await Order.Client.getUserOrders(client, ~limit=10) {
    | Ok(snapshot) =>
      Console.log(`orders: ${Int.toString(Array.length(snapshot.orders))}`)
      Console.log(`market balances: ${Int.toString(Array.length(snapshot.marketBalances))}`)
      Console.log(`has more: ${snapshot.hasMore ? "yes" : "no"}`)

      switch snapshot.orders[0] {
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
