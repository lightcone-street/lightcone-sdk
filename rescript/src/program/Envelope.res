// Offline order construction validates exact server rules before requesting a
// signature. Client submission fetches those rules for the selected orderbook.
let randomSalt: unit => bigint = %raw(`() => {
  const bytes = crypto.getRandomValues(new Uint8Array(8));
  return new DataView(bytes.buffer).getBigUint64(0, true) & 0x7fffffffffffffffn;
}`)
let signedRequest = async (
  ~maker: SolanaKit.address,
  ~market: SolanaKit.address,
  ~baseMint: SolanaKit.address,
  ~quoteMint: SolanaKit.address,
  ~side: int,
  ~price: string,
  ~size: string,
  ~rules: Scaling.OrderbookRules.t,
  ~orderbookId: string,
  ~nonce: bigint,
  ~salt: bigint,
  ~expiration: bigint,
  ~timeInForce: option<Shared.TimeInForce.t>,
  ~depositSource: option<Shared.DepositSource.t>,
  ~sign: Uint8Array.t => promise<result<Uint8Array.t, SdkError.t>>,
): result<Order__Raw.SubmitRequest.t, SdkError.t> => {
  let admission =
    Scaling.validateForOrderbook(rules, orderbookId)->Result.flatMap(() =>
      Scaling.scalePriceSize(~price, ~size, ~side, ~rules)->Result.flatMap(amounts =>
        Scaling.validateSignedFields(
          ~amountIn=amounts.amountIn,
          ~amountOut=amounts.amountOut,
          ~salt,
          ~nonce,
        )->Result.map(() => amounts)
      )
    )
  switch admission {
  | Error(error) => Error(Scaling(error))
  | Ok({amountIn, amountOut}) =>
    if expiration < -9223372036854775808n || expiration > Scaling.i64Max {
      Error(Validation("expiration must fit signed 64-bit seconds"))
    } else {
      let order: OrderPayload.t = {
        nonce,
        salt,
        maker,
        market,
        baseMint,
        quoteMint,
        side,
        amountIn,
        amountOut,
        expiration,
      }
      let message = SolanaKitCodec.encode(
        SolanaKitCodec.getUtf8Encoder(),
        OrderPayload.hashHex(order),
      )
      switch await sign(message) {
      | Error(error) => Error(error)
      | Ok(signature) =>
        Ok({
          maker: SolanaKit.addressToString(maker),
          nonce,
          salt,
          marketPubkey: SolanaKit.addressToString(market),
          baseToken: SolanaKit.addressToString(baseMint),
          quoteToken: SolanaKit.addressToString(quoteMint),
          side,
          amountIn,
          amountOut,
          expiration,
          signatureHex: OrderPayload.signatureHex(signature),
          orderbookId,
          ?timeInForce,
          ?depositSource,
        })
      }
    }
  }
}

// Native offline signing requires the supplied maker to control the keypair.
let buildLimitOrder = async (
  ~maker: SolanaKit.address,
  ~market: SolanaKit.address,
  ~baseMint: SolanaKit.address,
  ~quoteMint: SolanaKit.address,
  ~side: int,
  ~price: string,
  ~size: string,
  ~rules: Scaling.OrderbookRules.t,
  ~orderbookId: string,
  ~keypair: SolanaKit.cryptoKeyPair,
  ~nonce: bigint=0n,
  ~salt: option<bigint>=?,
  ~expiration: bigint=0n,
  ~timeInForce: option<Shared.TimeInForce.t>=?,
  ~depositSource: option<Shared.DepositSource.t>=?,
): result<Order__Raw.SubmitRequest.t, SdkError.t> => {
  let sign = async message => {
    switch await SolanaKitKeys.getAddressFromPublicKey(keypair.publicKey) {
    | address if address != maker => Error(SdkError.Signing("maker does not match signing key"))
    | _ =>
      switch await SolanaKitKeys.signBytes(keypair.privateKey, message) {
      | signature => Ok(signature)
      | exception _ => Error(SdkError.Signing("order signing failed"))
      }
    | exception _ => Error(SdkError.Signing("invalid signing key"))
    }
  }
  await signedRequest(
    ~maker,
    ~market,
    ~baseMint,
    ~quoteMint,
    ~side,
    ~price,
    ~size,
    ~rules,
    ~orderbookId,
    ~nonce,
    ~salt=salt->Option.getOr(randomSalt()),
    ~expiration,
    ~timeInForce,
    ~depositSource,
    ~sign,
  )
}

let submitLimitOrder = async (
  client: Client.t,
  ~maker: SolanaKit.address,
  ~market: SolanaKit.address,
  ~baseMint: SolanaKit.address,
  ~quoteMint: SolanaKit.address,
  ~side: int,
  ~price: string,
  ~size: string,
  ~rules: Scaling.OrderbookRules.t,
  ~orderbookId: string,
  ~keypair: SolanaKit.cryptoKeyPair,
  ~nonce: bigint=0n,
  ~salt: option<bigint>=?,
  ~expiration: bigint=0n,
  ~timeInForce: option<Shared.TimeInForce.t>=?,
  ~depositSource: option<Shared.DepositSource.t>=?,
): result<Order__Raw.SubmitResponse.t, SdkError.t> =>
  switch await buildLimitOrder(
    ~maker,
    ~market,
    ~baseMint,
    ~quoteMint,
    ~side,
    ~price,
    ~size,
    ~rules,
    ~orderbookId,
    ~keypair,
    ~nonce,
    ~salt?,
    ~expiration,
    ~timeInForce?,
    ~depositSource?,
  ) {
  | Error(error) => Error(error)
  | Ok(request) => await Order__Client.submit(client, request)
  }

// Fetch current rules before wallet interaction. Resolve omitted nonces from the
// client's cache, loading the on-chain nonce when the cache is empty.
let submitLimitOrderSigned = async (
  client: Client.t,
  ~market: SolanaKit.address,
  ~baseMint: SolanaKit.address,
  ~quoteMint: SolanaKit.address,
  ~side: int,
  ~price: string,
  ~size: string,
  ~orderbookId: string,
  ~nonce: option<bigint>=?,
  ~salt: option<bigint>=?,
  ~expiration: bigint=0n,
  ~timeInForce: option<Shared.TimeInForce.t>=?,
  ~depositSource: option<Shared.DepositSource.t>=?,
): result<Order__Raw.SubmitResponse.t, SdkError.t> => {
  switch Client.signerAddress(client) {
  | None => Error(Signing("no signing strategy configured"))
  | Some(maker) =>
    let nonceResult = switch nonce->Option.orElse(Client.orderNonce(client)) {
    | Some(value) => Ok(value)
    | None =>
      switch await Rpc.getNonce(client, ~user=maker) {
      | Error(error) => Error(error)
      | Ok(value) =>
        switch BigInt.fromFloat(value) {
        | Some(value) => Ok(value)
        | None => Error(SdkError.Validation("invalid on-chain nonce"))
        }
      }
    }
    switch nonceResult {
    | Error(error) => Error(error)
    | Ok(nonce) =>
      switch await Orderbook__Client.decimals(client, ~orderbookId) {
      | Error(error) => Error(error)
      | Ok(rules) =>
        switch await signedRequest(
          ~maker,
          ~market,
          ~baseMint,
          ~quoteMint,
          ~side,
          ~price,
          ~size,
          ~rules,
          ~orderbookId,
          ~nonce,
          ~salt=salt->Option.getOr(randomSalt()),
          ~expiration,
          ~timeInForce,
          ~depositSource=depositSource->Option.orElse(Some(Client.depositSource(client))),
          ~sign=message =>
            if Client.signerAddress(client) != Some(maker) {
              Promise.resolve(Error(SdkError.Signing("active wallet changed before order signing")))
            } else {
              Client.signMessageBytes(client, message)
            },
        ) {
        | Error(error) => Error(error)
        | Ok(request) =>
          Client.setOrderNonce(client, nonce)
          await Order__Client.submit(client, request)
        }
      }
    }
  }
}
