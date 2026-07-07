// Submit a signed limit order (bid) on the first market's first orderbook.
// Mirrors rust/examples/submit_order.rs: authenticate, deposit the order's quote
// collateral into the global pool, build the orderbook scaling decimals from the
// pair, sign the order off-chain, and POST it. ReScript surface (result core);
// the compiled .res.mjs is the JS example.

// Quote needed for the bid below (price * size, scaled to the deposit asset's
// 6 decimals). Must stay in sync with the same constant in CancelOrder__Example,
// which withdraws this amount back out of the global pool after cancelling —
// keeping the deposit/submit/cancel/withdraw cycle net-neutral across runs.
let orderQuoteAmount = 1100000n // 0.55 * 2 USDC

let delay: int => promise<unit> = %raw(`(ms) => new Promise((resolve) => setTimeout(resolve, ms))`)

// Poll the tracked deposit-token balances until `mint` shows at least `minimum`
// idle — the deposit must land before the order can rest (15 × 2s cap).
let waitForGlobalBalance = async (client, ~mint: string, ~minimum: string): result<
  unit,
  string,
> => {
  let target = Decimal.fromString(minimum)
  Console.log(`waiting for global balance: mint=${mint} required=${minimum}`)
  let rec attempt = async (n: int): result<unit, string> =>
    switch await Position.Client.depositTokenBalances(client) {
    | Error(error) => Error(SdkError.toMessage(error))
    | Ok(balances) => {
        let idle = switch balances->Dict.get(mint) {
        | Some(balance) => balance.idle
        | None => "0"
        }
        if Decimal.gte(Decimal.fromString(idle), target) {
          Console.log(`global balance ready: idle=${idle} (attempt ${Int.toString(n)})`)
          Ok()
        } else if n >= 15 {
          Error(`global balance for ${mint} did not reach ${minimum} within 30s`)
        } else {
          Console.log(`global balance not ready: idle=${idle}/${minimum} (attempt ${Int.toString(n)})`)
          await delay(2000)
          await attempt(n + 1)
        }
      }
    }
  await attempt(1)
}

// Derive the scaling decimals, sign with the maker's on-chain nonce, and submit
// the bid.
let submitOrder = async (client, ~pair: Market.OrderBookPair.t, ~maker, ~keypair) => {
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
            // Fund the global pool first: submit uses the client's default
            // Global deposit source, so the pool must cover `price * size` in
            // the quote deposit asset before the order can be placed.
            let mintAddress = SolanaKit.address(pair.quote.depositAsset)
            switch await Position.Builders.depositToGlobal(
              client,
              ~user=maker,
              ~mint=mintAddress,
              ~amount=orderQuoteAmount,
            ) {
            | Error(error) => Console.error(`deposit_to_global: ${SdkError.toMessage(error)}`)
            | Ok(signature) => {
                Console.log(`deposit_to_global: confirmed ${signature}`)
                switch await waitForGlobalBalance(
                  client,
                  ~mint=pair.quote.depositAsset,
                  ~minimum="1.1",
                ) {
                | Error(message) => Console.error(message)
                | Ok() => await submitOrder(client, ~pair, ~maker, ~keypair)
                }
              }
            }
          }
        }
      }
    }
  }
}

let _ = main()
