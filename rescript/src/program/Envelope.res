// Order envelopes — build + sign an order off-chain into a
// `Order.Raw.SubmitRequest.t`: align price to tick → scale price/size to raw
// u64 amounts (Scaling) → pack + keccak256 + ed25519-sign (OrderPayload) →
// request. A trigger order is the same signed payload with the trigger fields
// (`triggerPrice` / `triggerType`) added to the request.

let randomSalt: unit => bigint = %raw(`() => BigInt(Math.floor(Math.random() * Number.MAX_SAFE_INTEGER))`)

let scalingErrorToMessage = (error: Scaling.Error.t): string =>
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
  ~decimals: Scaling.OrderbookDecimals.t,
  ~orderbookId: string,
  ~keypair: SolanaKit.cryptoKeyPair,
  ~nonce: bigint=0n,
  ~salt: option<bigint>=?,
  ~expiration: bigint=0n,
  ~timeInForce: option<Shared.TimeInForce.t>=?,
  ~depositSource: option<Shared.DepositSource.t>=?,
): result<Order.Raw.SubmitRequest.t, SdkError.t> => {
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

// Build a signed trigger (take-profit / stop-loss) order: the limit-order sign
// flow plus the required trigger price + type on the request.
let buildTriggerOrder = async (
  ~maker: SolanaKit.address,
  ~market: SolanaKit.address,
  ~baseMint: SolanaKit.address,
  ~quoteMint: SolanaKit.address,
  ~side: int,
  ~price: string,
  ~size: string,
  ~decimals: Scaling.OrderbookDecimals.t,
  ~orderbookId: string,
  ~keypair: SolanaKit.cryptoKeyPair,
  ~triggerPrice: float,
  ~triggerType: Shared.TriggerType.t,
  ~nonce: bigint=0n,
  ~salt: option<bigint>=?,
  ~expiration: bigint=0n,
  ~timeInForce: option<Shared.TimeInForce.t>=?,
  ~depositSource: option<Shared.DepositSource.t>=?,
): result<Order.Raw.SubmitRequest.t, SdkError.t> =>
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
  | Ok(request) => Ok({...request, triggerPrice, triggerType})
  | Error(error) => Error(error)
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
): result<Order.Raw.SubmitResponse.t, SdkError.t> =>
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
  | Ok(request) => await Order.Client.submit(client, request)
  | Error(error) => Error(error)
  }

// Build + sign + submit a trigger order in one step.
let submitTriggerOrder = async (
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
  ~triggerPrice: float,
  ~triggerType: Shared.TriggerType.t,
  ~nonce: bigint=0n,
  ~salt: option<bigint>=?,
  ~expiration: bigint=0n,
  ~timeInForce: option<Shared.TimeInForce.t>=?,
  ~depositSource: option<Shared.DepositSource.t>=?,
): result<Order.Raw.TriggerResponse.t, SdkError.t> =>
  switch await buildTriggerOrder(
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
    ~triggerPrice,
    ~triggerType,
    ~nonce,
    ~salt?,
    ~expiration,
    ~timeInForce?,
    ~depositSource?,
  ) {
  | Ok(request) => await Order.Client.submitTrigger(client, request)
  | Error(error) => Error(error)
  }

// ── Strategy-signed submits ────────────────────────────────────────────────────
// Build + sign with the client's configured strategy (native keypair or external
// wallet adapter): the strategy's address is the maker, and an omitted nonce
// falls back to the client's cached order nonce (an explicit one updates the
// cache). Both strategies sign the same message — the UTF-8 bytes of the
// payload's hex hash.
let buildWithClientSigner = async (
  client: Client.t,
  ~market: SolanaKit.address,
  ~baseMint: SolanaKit.address,
  ~quoteMint: SolanaKit.address,
  ~side: int,
  ~price: string,
  ~size: string,
  ~decimals: Scaling.OrderbookDecimals.t,
  ~orderbookId: string,
  ~nonce: option<bigint>=?,
  ~salt: option<bigint>=?,
  ~expiration: bigint=0n,
  ~timeInForce: option<Shared.TimeInForce.t>=?,
  ~depositSource: option<Shared.DepositSource.t>=?,
  ~triggerPrice: option<float>=?,
  ~triggerType: option<Shared.TriggerType.t>=?,
): result<Order.Raw.SubmitRequest.t, SdkError.t> =>
  switch Client.signerAddress(client) {
  | None =>
    Error(
      Signing(
        "no signing strategy configured; call Client.useNativeSigner or Client.useExternalSigner first",
      ),
    )
  | Some(maker) =>
    let resolvedNonce = switch nonce {
    | Some(value) =>
      Client.setOrderNonce(client, value)
      value
    | None => Client.orderNonce(client)->Option.getOr(0n)
    }
    let alignedPrice = Scaling.alignPriceToTick(price, decimals)
    switch Scaling.scalePriceSize(~price=alignedPrice, ~size, ~side, ~decimals) {
    | Error(scalingError) => Error(Validation(scalingErrorToMessage(scalingError)))
    | Ok({amountIn, amountOut}) =>
      let resolvedSalt = switch salt {
      | Some(value) => value
      | None => randomSalt()
      }
      let order: OrderPayload.t = {
        nonce: resolvedNonce,
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
      let message = SolanaKitCodec.encode(SolanaKitCodec.getUtf8Encoder(), OrderPayload.hashHex(order))
      switch await Client.signMessageBytes(client, message) {
      | Error(error) => Error(error)
      | Ok(signature) =>
        Ok({
          maker: SolanaKit.addressToString(maker),
          nonce: resolvedNonce,
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
          triggerPrice: ?triggerPrice,
          triggerType: ?triggerType,
        })
      }
    }
  }

// Build + sign + submit a limit order with the client's signing strategy.
let submitLimitOrderSigned = async (
  client: Client.t,
  ~market: SolanaKit.address,
  ~baseMint: SolanaKit.address,
  ~quoteMint: SolanaKit.address,
  ~side: int,
  ~price: string,
  ~size: string,
  ~decimals: Scaling.OrderbookDecimals.t,
  ~orderbookId: string,
  ~nonce: option<bigint>=?,
  ~salt: option<bigint>=?,
  ~expiration: bigint=0n,
  ~timeInForce: option<Shared.TimeInForce.t>=?,
  ~depositSource: option<Shared.DepositSource.t>=?,
): result<Order.Raw.SubmitResponse.t, SdkError.t> =>
  switch await buildWithClientSigner(
    client,
    ~market,
    ~baseMint,
    ~quoteMint,
    ~side,
    ~price,
    ~size,
    ~decimals,
    ~orderbookId,
    ~nonce?,
    ~salt?,
    ~expiration,
    ~timeInForce?,
    ~depositSource?,
  ) {
  | Ok(request) => await Order.Client.submit(client, request)
  | Error(error) => Error(error)
  }

// Build + sign + submit a trigger order with the client's signing strategy.
let submitTriggerOrderSigned = async (
  client: Client.t,
  ~market: SolanaKit.address,
  ~baseMint: SolanaKit.address,
  ~quoteMint: SolanaKit.address,
  ~side: int,
  ~price: string,
  ~size: string,
  ~decimals: Scaling.OrderbookDecimals.t,
  ~orderbookId: string,
  ~triggerPrice: float,
  ~triggerType: Shared.TriggerType.t,
  ~nonce: option<bigint>=?,
  ~salt: option<bigint>=?,
  ~expiration: bigint=0n,
  ~timeInForce: option<Shared.TimeInForce.t>=?,
  ~depositSource: option<Shared.DepositSource.t>=?,
): result<Order.Raw.TriggerResponse.t, SdkError.t> =>
  switch await buildWithClientSigner(
    client,
    ~market,
    ~baseMint,
    ~quoteMint,
    ~side,
    ~price,
    ~size,
    ~decimals,
    ~orderbookId,
    ~nonce?,
    ~salt?,
    ~expiration,
    ~timeInForce?,
    ~depositSource?,
    ~triggerPrice,
    ~triggerType,
  ) {
  | Ok(request) => await Order.Client.submitTrigger(client, request)
  | Error(error) => Error(error)
  }
