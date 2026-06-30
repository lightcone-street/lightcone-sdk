// Limit-order envelope — build + sign a limit order off-chain into a
// `Order.submitOrderRequest`. Mirrors the Rust LimitOrderEnvelope sign flow:
// align price to tick → scale price/size to raw u64 amounts (Scaling) → pack +
// keccak256 + ed25519-sign (OrderPayload) → request.

let randomSalt: unit => bigint = %raw(`() => BigInt(Math.floor(Math.random() * Number.MAX_SAFE_INTEGER))`)

let scalingErrorToMessage = (error: Scaling.scalingError): string =>
  switch error {
  | NonPositivePrice(value) => `price must be positive, got ${value}`
  | NonPositiveSize(value) => `size must be positive, got ${value}`
  | Overflow(context) => `overflow: ${context}`
  | ZeroAmount => "computed amount is zero"
  }

// Build a signed limit order. `decimals` + the mints/market come from the
// orderbook pair (the caller extracts them). `side` is 0 = Bid, 1 = Ask. `nonce`
// defaults to 0; `salt` is random when omitted.
let buildLimitOrder = async (
  ~maker: SolanaKit.address,
  ~market: SolanaKit.address,
  ~baseMint: SolanaKit.address,
  ~quoteMint: SolanaKit.address,
  ~side: int,
  ~price: string,
  ~size: string,
  ~decimals: Scaling.orderbookDecimals,
  ~orderbookId: string,
  ~keypair: SolanaKit.cryptoKeyPair,
  ~nonce: bigint=0n,
  ~salt: option<bigint>=?,
  ~expiration: bigint=0n,
  ~timeInForce: option<Shared.TimeInForce.t>=?,
  ~depositSource: option<Shared.DepositSource.t>=?,
): result<Order.submitOrderRequest, SdkError.t> => {
  let alignedPrice = Scaling.alignPriceToTick(price, decimals)
  switch Scaling.scalePriceSize(~price=alignedPrice, ~size, ~side, ~decimals) {
  | Error(scalingError) => Error(Validation(scalingErrorToMessage(scalingError)))
  | Ok({amountIn, amountOut}) =>
    let resolvedSalt = switch salt {
    | Some(value) => value
    | None => randomSalt()
    }
    let order: OrderPayload.t = {
      nonce,
      salt: resolvedSalt,
      maker,
      market,
      baseMint,
      quoteMint,
      side,
      amountIn,
      amountOut,
      expiration,
    }
    let signature = await OrderPayload.sign(order, keypair)
    Ok({
      maker: SolanaKit.addressToString(maker),
      nonce,
      salt: resolvedSalt,
      marketPubkey: SolanaKit.addressToString(market),
      baseToken: SolanaKit.addressToString(baseMint),
      quoteToken: SolanaKit.addressToString(quoteMint),
      side,
      amountIn,
      amountOut,
      expiration,
      signatureHex: OrderPayload.signatureHex(signature),
      orderbookId,
      timeInForce: ?timeInForce,
      depositSource: ?depositSource,
    })
  }
}

// Build + sign + submit in one step.
let submitLimitOrder = async (
  client: Client.t,
  ~maker,
  ~market,
  ~baseMint,
  ~quoteMint,
  ~side,
  ~price,
  ~size,
  ~decimals,
  ~orderbookId,
  ~keypair,
  ~nonce: bigint=0n,
  ~salt: option<bigint>=?,
  ~expiration: bigint=0n,
  ~timeInForce: option<Shared.TimeInForce.t>=?,
  ~depositSource: option<Shared.DepositSource.t>=?,
): result<Order.submitOrderResponse, SdkError.t> =>
  switch await buildLimitOrder(
    ~maker,
    ~market,
    ~baseMint,
    ~quoteMint,
    ~side,
    ~price,
    ~size,
    ~decimals,
    ~orderbookId,
    ~keypair,
    ~nonce,
    ~salt?,
    ~expiration,
    ~timeInForce?,
    ~depositSource?,
  ) {
  | Ok(request) => await Order.submit(client, request)
  | Error(error) => Error(error)
  }
