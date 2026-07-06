// Submit a signed limit order (bid) on the first market's first orderbook.
// Mirrors rust/examples/submit_order.rs: authenticate, build the orderbook
// scaling decimals from the pair, sign the order off-chain, and POST it.
//
// The global pool must already hold `price * size` of the quote deposit asset as
// collateral before the order can rest — see GlobalDepositWithdrawal.res for the
// deposit flow. ReScript surface (result core); the compiled .res.mjs is the JS
// example.
//
// No TS facade equivalent: constructing the order keypair + market/mint addresses
// needs SolanaKit (@solana/kit) address/keypair helpers not surfaced on TypeScriptApi.gen.ts.
let main = async () => {
  let client = Common__Example.client()
  let secretKey = Common__Example.walletSecretKey()
  await Client.useNativeSigner(client, secretKey)

  // The keypair signs the order envelope off-chain; `maker` is its base58 address.
  let keypair = await SolanaKitKeys.createKeyPairFromBytes(secretKey)
  let maker = await SolanaKitKeys.getAddressFromPublicKey(keypair.publicKey)

  switch await Auth.Client.login(client) {
  | Error(error) => Console.error(SdkError.toMessage(error))
  | Ok(_) =>
    switch await Market.Client.get(client, ~limit=1) {
    | Error(error) => Console.error(SdkError.toMessage(error))
    | Ok({markets}) =>
      switch markets[0] {
      | None => Console.log("no markets found")
      | Some(market) =>
        switch market.orderbookPairs
        ->Array.find(pair => pair.active)
        ->Option.orElse(market.orderbookPairs[0]) {
        | None => Console.log("selected market has no orderbooks")
        | Some(pair) => {
            // Derive scaling decimals from the pair's token metadata (no REST call):
            // price_decimals = max(0, 6 + quote_decimals - base_decimals).
            let baseDecimals = Float.toInt(pair.base.decimals)
            let quoteDecimals = Float.toInt(pair.quote.decimals)
            let priceDecimals = 6 + quoteDecimals - baseDecimals
            let decimals: Scaling.OrderbookDecimals.t = {
              baseDecimals,
              quoteDecimals,
              priceDecimals: priceDecimals > 0 ? priceDecimals : 0,
              tickSize: pair.tickSize > 0.0 ? pair.tickSize : 0.0,
            }

            // Sign with the maker's current on-chain nonce so the order is valid.
            let nonce = switch await Order.Client.getNonce(client, ~user=maker) {
            | Ok(value) => BigInt.fromFloat(value)->Option.getOr(0n)
            | Error(_) => 0n
            }

            switch await Envelope.submitLimitOrder(
              client,
              ~maker,
              ~market=SolanaKit.address(pair.marketPubkey),
              ~baseMint=SolanaKit.address(pair.base.mint),
              ~quoteMint=SolanaKit.address(pair.quote.mint),
              ~side=0, // 0 = bid, 1 = ask
              ~price="0.55",
              ~size="2",
              ~decimals,
              ~orderbookId=pair.orderbookId,
              ~keypair,
              ~nonce,
            ) {
            | Ok(response) => {
                let status = switch response.status {
                | Order.Raw.SubmitStatus.Accepted => "accepted"
                | PartialFill => "partial_fill"
                | Filled => "filled"
                }
                Console.log(
                  `submitted: ${response.orderHash} status=${status} filled=${response.filled} remaining=${response.remaining} fills=${Int.toString(
                      Array.length(response.fills),
                    )}`,
                )
              }
            | Error(error) => Console.error(SdkError.toMessage(error))
            }
          }
        }
      }
    }
  }
}

let _ = main()
