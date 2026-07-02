// The TypeScript-facing SDK surface (gentype → TypeScriptApi.gen.ts; the package
// entry point). ReScript consumers use the result-returning domain modules directly
// (`Market.featured(client)` → `promise<result<_, SdkError.t>>`); this module wraps
// each call with `SdkError.unwrap` so TypeScript gets throwing `Promise<T>` ergonomics,
// adapts string pubkeys + the client's stored keypair into the `@solana/kit` values the
// core needs, and groups everything under `@genType module` namespaces so TS consumers
// write `MarketClient.featured(client)`, `OrderClient.submitLimit(...)`, etc. Each facade
//
// namespace is named `<DomainModule>Client` so it never collides with a domain module.

// ── Client lifecycle (top-level) ──────────────────────────────────────────────
// Build a client (Prod by default; per-field overrides win, then SDK_* env vars).
@genType
let make = (
  ~env: Env.t=Prod,
  ~baseUrl: option<string>=?,
  ~wsUrl: option<string>=?,
  ~rpcUrl: option<string>=?,
  ~backupRpcUrl: option<string>=?,
  ~programId: option<string>=?,
  ~depositSource: Shared.DepositSource.t=Global,
  (),
): Client.t =>
  Client.make(~env, ~baseUrl?, ~wsUrl?, ~rpcUrl?, ~backupRpcUrl?, ~programId?, ~depositSource, ())

// Ergonomic constructor for the common case (TS: `makeForEnv("staging")`).
@genType
let makeForEnv = (env: Env.t): Client.t => Client.make(~env, ())

// Attach a native ed25519 signer from a 64-byte wallet secret key.
@genType
let useNativeSigner = (client: Client.t, secretKey: Uint8Array.t): promise<unit> =>
  Client.useNativeSigner(client, secretKey)

// Attach an external wallet signer (browser wallet adapter): the wallet's base58
// address plus its message- and transaction-signing callbacks. `signMessage`
// returns the raw 64-byte ed25519 signature; `signTransaction` takes the
// serialized unsigned wire transaction and returns the fully signed wire bytes.
@genType
let useExternalSigner = (
  client: Client.t,
  ~address: string,
  ~signMessage: Uint8Array.t => promise<Uint8Array.t>,
  ~signTransaction: Uint8Array.t => promise<Uint8Array.t>,
): unit =>
  Client.useExternalSigner(client, ~address=SolanaKit.address(address), ~signMessage, ~signTransaction)

// Drop the configured signing strategy / cached order nonce.
@genType
let clearSigningStrategy = (client: Client.t): unit => Client.clearSigningStrategy(client)
@genType
let clearOrderNonce = (client: Client.t): unit => Client.clearOrderNonce(client)

// The configured signer's address as a base58 string (None until useNativeSigner
// or useExternalSigner is called).
@genType
let signerAddress = (client: Client.t): option<string> =>
  Client.signerAddress(client)->Option.map(SolanaKit.addressToString)

// Generic boundary helper — unwrap any result-promise into a throwing value-promise.
@genType
let unwrap = SdkError.unwrap

// The configured signer's address, or throw (internal).
let signerAddressOrThrow = (client: Client.t): SolanaKit.address =>
  switch Client.signerAddress(client) {
  | Some(address) => address
  | None =>
    let error = SdkError.Signing(
      "no signing strategy set; call useNativeSigner or useExternalSigner first",
    )
    SdkError.throwAsJsError(SdkError.toMessage(error), error)
  }

// ── Auth ──────────────────────────────────────────────────────────────────────
@genType
module AuthClient = {
  let getNonce = (client: Client.t): promise<string> => SdkError.unwrap(Auth.getNonce(client))
  let login = (client: Client.t, ~useEmbeddedWallet: option<bool>=?): promise<
    Auth.sessionResponse,
  > => SdkError.unwrap(Auth.login(client, ~useEmbeddedWallet?))
  let checkSession = (client: Client.t, ~cookieHeader: option<string>=?): promise<
    Auth.sessionResponse,
  > => SdkError.unwrap(Auth.checkSession(client, ~cookieHeader?))
  let logout = (client: Client.t): promise<unit> => SdkError.unwrap(Auth.logout(client))
  let isAuthenticated = (client: Client.t): bool => Auth.isAuthenticated(client)
  let registerPrivy = (client: Client.t): promise<unit> => SdkError.unwrap(Auth.registerPrivy(client))
  let disconnectX = (client: Client.t): promise<unit> => SdkError.unwrap(Auth.disconnectX(client))
  let connectXUrl = (client: Client.t): string => Auth.connectXUrl(client)
}

// ── Markets ───────────────────────────────────────────────────────────────────
@genType
module MarketClient = {
  let get = (client: Client.t, ~cursor: option<float>=?, ~limit: option<int>=?): promise<
    Market.marketsResult,
  > => SdkError.unwrap(Market.get(client, ~cursor?, ~limit?))
  let featured = (client: Client.t): promise<array<Market.marketSearchResult>> =>
    SdkError.unwrap(Market.featured(client))
  let getBySlug = (client: Client.t, slug: string): promise<Market.market> =>
    SdkError.unwrap(Market.getBySlug(client, ~slug))
  let getByPubkey = (client: Client.t, pubkey: string): promise<Market.market> =>
    SdkError.unwrap(Market.getByPubkey(client, ~pubkey))
  let search = (client: Client.t, query: string, ~limit: option<int>=?): promise<
    array<Market.marketSearchResult>,
  > => SdkError.unwrap(Market.search(client, ~query, ~limit?))
  let globalDepositAssets = (client: Client.t): promise<Market.globalDepositAssetsResult> =>
    SdkError.unwrap(Market.globalDepositAssets(client))
  let depositMints = (client: Client.t, marketPubkey: string): promise<
    Market.depositMintsResponse,
  > => SdkError.unwrap(Market.getDepositMints(client, ~marketPubkey))
}

// ── Orderbook ─────────────────────────────────────────────────────────────────
@genType
module OrderbookClient = {
  let get = (client: Client.t, orderbookId: string, ~depth: option<int>=?): promise<
    Orderbook.orderbookDepthResponse,
  > => SdkError.unwrap(Orderbook.get(client, ~orderbookId, ~depth?))
}

// ── Trades ────────────────────────────────────────────────────────────────────
@genType
module TradeClient = {
  let forOrderbook = (
    client: Client.t,
    orderbookId: string,
    ~limit: option<int>=?,
    ~cursor: option<float>=?,
  ): promise<Trade.tradesPage> =>
    SdkError.unwrap(Trade.get(client, ~orderbookId, ~limit?, ~cursor?))
  let forMarket = (
    client: Client.t,
    marketPubkey: string,
    ~limit: option<int>=?,
    ~cursor: option<float>=?,
  ): promise<Trade.tradesPage> =>
    SdkError.unwrap(Trade.getByMarket(client, ~marketPubkey, ~limit?, ~cursor?))
}

// ── Orders ────────────────────────────────────────────────────────────────────
@genType
module OrderClient = {
  let forUser = (client: Client.t, ~limit: option<int>=?, ~cursor: option<string>=?): promise<
    Order.userOrdersResponse,
  > => SdkError.unwrap(Order.getUserOrders(client, ~limit?, ~cursor?))

  // Submit / cancel sign with the client's configured strategy (native keypair
  // or external wallet adapter).
  let submitLimit = async (
    client: Client.t,
    ~market: string,
    ~baseMint: string,
    ~quoteMint: string,
    ~side: int,
    ~price: string,
    ~size: string,
    ~baseDecimals: int,
    ~quoteDecimals: int,
    ~priceDecimals: int,
    ~tickSize: float,
    ~orderbookId: string,
    ~timeInForce: option<Shared.TimeInForce.t>=?,
  ): Order.submitOrderResponse => {
    let decimals: Scaling.orderbookDecimals = {baseDecimals, quoteDecimals, priceDecimals, tickSize}
    await SdkError.unwrap(
      Envelope.submitLimitOrderSigned(
        client,
        ~market=SolanaKit.address(market),
        ~baseMint=SolanaKit.address(baseMint),
        ~quoteMint=SolanaKit.address(quoteMint),
        ~side,
        ~price,
        ~size,
        ~decimals,
        ~orderbookId,
        ~timeInForce?,
      ),
    )
  }

  let submitTrigger = async (
    client: Client.t,
    ~market: string,
    ~baseMint: string,
    ~quoteMint: string,
    ~side: int,
    ~price: string,
    ~size: string,
    ~baseDecimals: int,
    ~quoteDecimals: int,
    ~priceDecimals: int,
    ~tickSize: float,
    ~orderbookId: string,
    ~triggerPrice: float,
    ~triggerType: Shared.TriggerType.t,
    ~timeInForce: option<Shared.TimeInForce.t>=?,
  ): Order.triggerOrderResponse => {
    let decimals: Scaling.orderbookDecimals = {baseDecimals, quoteDecimals, priceDecimals, tickSize}
    await SdkError.unwrap(
      Envelope.submitTriggerOrderSigned(
        client,
        ~market=SolanaKit.address(market),
        ~baseMint=SolanaKit.address(baseMint),
        ~quoteMint=SolanaKit.address(quoteMint),
        ~side,
        ~price,
        ~size,
        ~decimals,
        ~orderbookId,
        ~triggerPrice,
        ~triggerType,
        ~timeInForce?,
      ),
    )
  }

  let cancel = (client: Client.t, ~orderHash: string): promise<Order.cancelSuccess> =>
    SdkError.unwrap(Order.cancelSigned(client, ~orderHash))

  let cancelTrigger = (client: Client.t, ~triggerOrderId: string): promise<Order.cancelTriggerSuccess> =>
    SdkError.unwrap(Order.cancelTriggerSigned(client, ~triggerOrderId))

  let cancelAll = (client: Client.t, ~orderbookId: string): promise<Order.cancelAllSuccess> =>
    SdkError.unwrap(Order.cancelAllSigned(client, ~orderbookId))

  // The authenticated user's filled orders with nested fill events.
  let fills = (
    client: Client.t,
    ~marketPubkey: option<string>=?,
    ~limit: option<int>=?,
    ~cursor: option<string>=?,
  ): promise<Order.userOrderFillsResponse> =>
    SdkError.unwrap(Order.getUserOrderFills(client, ~marketPubkey?, ~limit?, ~cursor?))

  // Public variant: any wallet's fills via the URL path (no auth).
  let fillsByWallet = (
    client: Client.t,
    ~walletAddress: string,
    ~marketPubkey: option<string>=?,
    ~limit: option<int>=?,
    ~cursor: option<string>=?,
  ): promise<Order.userOrderFillsResponse> =>
    SdkError.unwrap(
      Order.getUserOrderFillsByWallet(client, ~walletAddress, ~marketPubkey?, ~limit?, ~cursor?),
    )
}

// ── Positions ─────────────────────────────────────────────────────────────────
@genType
module PositionClient = {
  let forUser = (client: Client.t, userPubkey: string): promise<Position.positionsResponse> =>
    SdkError.unwrap(Position.get(client, ~userPubkey))
  let forMarket = (client: Client.t, ~userPubkey: string, ~marketPubkey: string): promise<
    Position.marketPositionsResponse,
  > => SdkError.unwrap(Position.getForMarket(client, ~userPubkey, ~marketPubkey))
  let mine = (client: Client.t): promise<Position.positionsResponse> =>
    SdkError.unwrap(Position.positions(client))
  let depositTokenBalances = (client: Client.t): promise<dict<Position.depositTokenBalance>> =>
    SdkError.unwrap(Position.depositTokenBalances(client))

  // Each builds + signs + sends a Solana transaction and returns the tx signature.
  let depositToGlobal = async (client: Client.t, ~mint: string, ~amount: bigint): string => {
    let user = signerAddressOrThrow(client)
    await SdkError.unwrap(
      PositionBuilders.depositToGlobal(client, ~user, ~mint=SolanaKit.address(mint), ~amount),
    )
  }
  let withdrawFromGlobal = async (client: Client.t, ~mint: string, ~amount: bigint): string => {
    let user = signerAddressOrThrow(client)
    await SdkError.unwrap(
      PositionBuilders.withdrawFromGlobal(client, ~user, ~mint=SolanaKit.address(mint), ~amount),
    )
  }
  let globalToMarketDeposit = async (
    client: Client.t,
    ~market: string,
    ~mint: string,
    ~amount: bigint,
    ~numOutcomes: int,
  ): string => {
    let user = signerAddressOrThrow(client)
    await SdkError.unwrap(
      PositionBuilders.globalToMarketDeposit(
        client,
        ~user,
        ~market=SolanaKit.address(market),
        ~mint=SolanaKit.address(mint),
        ~amount,
        ~numOutcomes,
      ),
    )
  }
  let merge = async (
    client: Client.t,
    ~market: string,
    ~mint: string,
    ~amount: bigint,
    ~numOutcomes: int,
  ): string => {
    let user = signerAddressOrThrow(client)
    await SdkError.unwrap(
      PositionBuilders.merge(
        client,
        ~user,
        ~market=SolanaKit.address(market),
        ~mint=SolanaKit.address(mint),
        ~amount,
        ~numOutcomes,
      ),
    )
  }
  let redeemWinnings = async (
    client: Client.t,
    ~market: string,
    ~mint: string,
    ~amount: bigint,
    ~outcomeIndex: int,
  ): string => {
    let user = signerAddressOrThrow(client)
    await SdkError.unwrap(
      PositionBuilders.redeemWinnings(
        client,
        ~user,
        ~market=SolanaKit.address(market),
        ~mint=SolanaKit.address(mint),
        ~amount,
        ~outcomeIndex,
      ),
    )
  }

  // Market-level direct deposit (mint complete set): wallet ATA → market vault.
  let deposit = async (
    client: Client.t,
    ~market: string,
    ~mint: string,
    ~amount: bigint,
    ~numOutcomes: int,
  ): string => {
    let user = signerAddressOrThrow(client)
    await SdkError.unwrap(
      PositionBuilders.deposit(
        client,
        ~user,
        ~market=SolanaKit.address(market),
        ~mint=SolanaKit.address(mint),
        ~amount,
        ~numOutcomes,
      ),
    )
  }

  // Withdraw conditional tokens from the position ATA to the user's own ATA
  // (`mint` is the conditional mint).
  let withdrawFromPosition = async (
    client: Client.t,
    ~market: string,
    ~mint: string,
    ~amount: bigint,
    ~outcomeIndex: int,
  ): string => {
    let user = signerAddressOrThrow(client)
    await SdkError.unwrap(
      PositionBuilders.withdrawFromPosition(
        client,
        ~user,
        ~market=SolanaKit.address(market),
        ~mint=SolanaKit.address(mint),
        ~amount,
        ~outcomeIndex,
      ),
    )
  }

  // Append newly-added deposit mints' ATAs to an existing position ALT. The
  // signer acts as the operator; `user` is the position owner.
  let extendPositionTokens = async (
    client: Client.t,
    ~user: string,
    ~market: string,
    ~lookupTable: string,
    ~depositMints: array<string>,
    ~numOutcomes: int,
  ): string => {
    let operator = signerAddressOrThrow(client)
    await SdkError.unwrap(
      PositionBuilders.extendPositionTokens(
        client,
        ~operator,
        ~user=SolanaKit.address(user),
        ~market=SolanaKit.address(market),
        ~lookupTable=SolanaKit.address(lookupTable),
        ~depositMints=depositMints->Array.map(SolanaKit.address),
        ~numOutcomes,
      ),
    )
  }

  // Deactivate / close a position ALT. The signer acts as the operator;
  // `position` is the position PDA address itself.
  let closePositionAlt = async (
    client: Client.t,
    ~position: string,
    ~market: string,
    ~lookupTable: string,
  ): string => {
    let operator = signerAddressOrThrow(client)
    await SdkError.unwrap(
      PositionBuilders.closePositionAlt(
        client,
        ~operator,
        ~position=SolanaKit.address(position),
        ~market=SolanaKit.address(market),
        ~lookupTable=SolanaKit.address(lookupTable),
      ),
    )
  }

  // Close empty conditional ATAs owned by a position PDA after resolution. The
  // signer acts as the operator; `position` is the position PDA address itself.
  let closePositionTokenAccounts = async (
    client: Client.t,
    ~market: string,
    ~position: string,
    ~depositMints: array<string>,
    ~numOutcomes: int,
  ): string => {
    let operator = signerAddressOrThrow(client)
    await SdkError.unwrap(
      PositionBuilders.closePositionTokenAccounts(
        client,
        ~operator,
        ~market=SolanaKit.address(market),
        ~position=SolanaKit.address(position),
        ~depositMints=depositMints->Array.map(SolanaKit.address),
        ~numOutcomes,
      ),
    )
  }
}

// ── Metrics ───────────────────────────────────────────────────────────────────
@genType
module MetricsClient = {
  let platform = (client: Client.t): promise<Metrics.platformMetrics> =>
    SdkError.unwrap(Metrics.platform(client))
  let markets = (client: Client.t): promise<Metrics.marketsMetrics> =>
    SdkError.unwrap(Metrics.markets(client))
  let market = (client: Client.t, marketPubkey: string): promise<Metrics.marketDetailMetrics> =>
    SdkError.unwrap(Metrics.market(client, ~marketPubkey))
  let orderbookTickers = (client: Client.t, ~depositAsset: option<string>=?): promise<
    Metrics.orderbookTickersResponse,
  > => SdkError.unwrap(Metrics.orderbookTickers(client, ~depositAsset?))
  let categories = (client: Client.t): promise<Metrics.categoriesMetrics> =>
    SdkError.unwrap(Metrics.categories(client))
  let depositTokens = (client: Client.t): promise<Metrics.depositTokensMetrics> =>
    SdkError.unwrap(Metrics.depositTokens(client))
  let leaderboard = (client: Client.t, ~limit: option<int>=?): promise<Metrics.leaderboard> =>
    SdkError.unwrap(Metrics.leaderboard(client, ~limit?))
  let orderbook = (client: Client.t, ~orderbookId: string): promise<
    Metrics.orderbookVolumeMetrics,
  > => SdkError.unwrap(Metrics.orderbook(client, ~orderbookId))
  let depositTokensVolumeHistory = (
    client: Client.t,
    ~fromMs: option<float>=?,
    ~toMs: option<float>=?,
    ~limit: option<int>=?,
  ): promise<Metrics.depositTokenVolumeHistory> =>
    SdkError.unwrap(Metrics.depositTokensVolumeHistory(client, ~fromMs?, ~toMs?, ~limit?))
  let openInterestHistory = (
    client: Client.t,
    ~fromMs: option<float>=?,
    ~toMs: option<float>=?,
    ~limit: option<int>=?,
  ): promise<Metrics.openInterestHistory> =>
    SdkError.unwrap(Metrics.openInterestHistory(client, ~fromMs?, ~toMs?, ~limit?))
  let uniqueTradersHistory = (
    client: Client.t,
    ~scope: option<Metrics.UniqueTradersHistoryScope.t>=?,
    ~scopeKey: option<string>=?,
    ~fromMs: option<float>=?,
    ~toMs: option<float>=?,
    ~limit: option<int>=?,
  ): promise<Metrics.uniqueTradersHistory> =>
    SdkError.unwrap(
      Metrics.uniqueTradersHistory(client, ~scope?, ~scopeKey?, ~fromMs?, ~toMs?, ~limit?),
    )
  let history = (
    client: Client.t,
    ~scope: string,
    ~scopeKey: string,
    ~resolution: Shared.Resolution.t=Shared.Resolution.Hour1,
    ~fromMs: option<float>=?,
    ~toMs: option<float>=?,
    ~limit: option<int>=?,
  ): promise<Metrics.metricsHistory> =>
    SdkError.unwrap(
      Metrics.history(client, ~scope, ~scopeKey, ~resolution, ~fromMs?, ~toMs?, ~limit?),
    )
  let user = (client: Client.t): promise<Metrics.userMetrics> =>
    SdkError.unwrap(Metrics.user(client))
  let userByWallet = (client: Client.t, ~walletAddress: string): promise<Metrics.userMetrics> =>
    SdkError.unwrap(Metrics.userByWallet(client, ~walletAddress))
}

// ── PriceHistory ──────────────────────────────────────────────────────────────
@genType
module PriceHistoryClient = {
  let get = (
    client: Client.t,
    ~orderbookId: string,
    ~resolution: Shared.Resolution.t,
    ~fromMs: option<float>=?,
    ~toMs: option<float>=?,
  ): promise<PriceHistory.orderbookPriceHistoryResponse> =>
    SdkError.unwrap(PriceHistory.get(client, ~orderbookId, ~resolution, ~fromMs?, ~toMs?))
  let lineData = (
    client: Client.t,
    ~orderbookId: string,
    ~resolution: Shared.Resolution.t,
    ~fromMs: option<float>=?,
    ~toMs: option<float>=?,
    ~cursor: option<float>=?,
    ~limit: option<float>=?,
  ): promise<array<PriceHistory.lineData>> =>
    SdkError.unwrap(
      PriceHistory.getLineData(
        client,
        ~orderbookId,
        ~resolution,
        ~fromMs?,
        ~toMs?,
        ~cursor?,
        ~limit?,
      ),
    )
  let depositAssetSnapshot = (client: Client.t): promise<
    PriceHistory.depositAssetPricesSnapshotResponse,
  > => SdkError.unwrap(PriceHistory.getDepositAssetPricesSnapshot(client))
}

// ── Notifications ─────────────────────────────────────────────────────────────
@genType
module NotificationClient = {
  let list = (client: Client.t): promise<array<Notification.notification>> =>
    SdkError.unwrap(Notification.fetch(client))
  let dismiss = (client: Client.t, ~notificationId: string): promise<unit> =>
    SdkError.unwrap(Notification.dismiss(client, ~notificationId))
}

// ── Referrals ─────────────────────────────────────────────────────────────────
@genType
module ReferralClient = {
  let status = (client: Client.t): promise<Referral.referralStatus> =>
    SdkError.unwrap(Referral.getStatus(client))
  let redeem = (client: Client.t, code: string): promise<Referral.redeemResult> =>
    SdkError.unwrap(Referral.redeem(client, ~code))
}

// ── Faucet ────────────────────────────────────────────────────────────────────
@genType
module FaucetClient = {
  let claim = (client: Client.t, walletAddress: string): promise<Faucet.faucetResponse> =>
    SdkError.unwrap(Faucet.claim(client, ~walletAddress))
}

// ── Onchain reads + PDA derivation (over the @solana/kit RPC; pubkeys as base58) ──
@genType
module RpcClient = {
  // Which endpoint is currently serving reads ("primary" until a failover flips it).
  let activeRpc = (client: Client.t): RpcFailover.activeRpc => Rpc.activeRpc(client)
  let latestBlockhash = (client: Client.t): promise<string> =>
    SdkError.unwrap(Rpc.getLatestBlockhash(client))
  let exchange = (client: Client.t): promise<Accounts.exchange> =>
    SdkError.unwrap(Rpc.getExchange(client))
  let market = (client: Client.t, marketPubkey: string): promise<Accounts.market> =>
    SdkError.unwrap(Rpc.getMarket(client, SolanaKit.address(marketPubkey)))
  let orderbook = (client: Client.t, baseMint: string, quoteMint: string): promise<
    Accounts.orderbook,
  > =>
    SdkError.unwrap(
      Rpc.getOrderbook(
        client,
        ~mintA=SolanaKit.address(baseMint),
        ~mintB=SolanaKit.address(quoteMint),
      ),
    )
  let position = (client: Client.t, userPubkey: string, marketPubkey: string): promise<
    option<Accounts.position>,
  > =>
    SdkError.unwrap(
      Rpc.getPosition(
        client,
        ~owner=SolanaKit.address(userPubkey),
        ~market=SolanaKit.address(marketPubkey),
      ),
    )
  let nonce = (client: Client.t, userPubkey: string): promise<float> =>
    SdkError.unwrap(Rpc.getNonce(client, ~user=SolanaKit.address(userPubkey)))
  let exchangePda = async (client: Client.t): string =>
    SolanaKit.addressToString(await Rpc.exchangePda(client))
  let marketPda = async (client: Client.t, marketId: bigint): string =>
    SolanaKit.addressToString(await Rpc.marketPda(client, ~marketId))
  let positionPda = async (client: Client.t, userPubkey: string, marketPubkey: string): string =>
    SolanaKit.addressToString(
      await Rpc.positionPda(
        client,
        ~owner=SolanaKit.address(userPubkey),
        ~market=SolanaKit.address(marketPubkey),
      ),
    )
  let globalDepositTokenPda = async (client: Client.t, mint: string): string =>
    SolanaKit.addressToString(
      await Rpc.globalDepositTokenPda(client, ~mint=SolanaKit.address(mint)),
    )
}

// ── WebSocket ─────────────────────────────────────────────────────────────────

@genType
module WsClient = {
  @genType.opaque
  type wsConnection = Ws.connection
  @genType
  type wsMessage = Messages.messageIn
  @genType
  type wsSubscription = Subscriptions.SubscribeParams.t
  @genType
  type wsUnsubscription = Subscriptions.UnsubscribeParams.t

  let connect = (
    client: Client.t,
    ~onMessage: Messages.messageIn => unit,
    ~onConnected: unit => unit=() => (),
    ~onError: SdkError.t => unit=_ => (),
  ): wsConnection => Ws.connect(~url=client.wsUrl, ~onMessage, ~onConnected, ~onError, ())
  let subscribe = (connection: wsConnection, subscription: Subscriptions.SubscribeParams.t): unit =>
    Ws.subscribe(connection, subscription)->ignore
  let unsubscribe = (
    connection: wsConnection,
    subscription: Subscriptions.UnsubscribeParams.t,
  ): unit => Ws.unsubscribe(connection, subscription)->ignore
  let disconnect = (connection: wsConnection): unit => Ws.disconnect(connection)
  let isConnected = (connection: wsConnection): bool => Ws.isConnected(connection)
  let readyState = (connection: wsConnection): Ws.readyState => Ws.readyState(connection)
  // Drop tracked user-channel subscriptions (call after logout).
  let clearAuthedSubscriptions = (connection: wsConnection): unit =>
    Ws.clearAuthedSubscriptions(connection)
}

// ── Live WS state containers ──────────────────────────────────────────────────
// Stateful reducers a consumer feeds from a `WsClient` `~onMessage` closure (see
// `domain/*State`). Exposed as `Live*` so the facade module never shares a name with —
// and thus shadows — the domain `*State` module it re-exports.
@genType
module LiveOrderbook = {
  let make = OrderbookState.make
  let apply = OrderbookState.apply
  let bestBid = OrderbookState.bestBid
  let bestAsk = OrderbookState.bestAsk
  let midPrice = OrderbookState.midPrice
  let spread = OrderbookState.spread
  let bids = OrderbookState.bids
  let asks = OrderbookState.asks
  let isEmpty = OrderbookState.isEmpty
  let seq = OrderbookState.seq
  let orderbookId = OrderbookState.orderbookId
  let clear = OrderbookState.clear
}

@genType
module LivePriceHistory = {
  let make = PriceHistoryState.make
  let applySnapshot = PriceHistoryState.applySnapshot
  let applyUpdate = PriceHistoryState.applyUpdate
  let get = PriceHistoryState.get
  let clear = PriceHistoryState.clear
}

@genType
module LiveDepositPrice = {
  let make = DepositPriceState.make
  let applySnapshot = DepositPriceState.applySnapshot
  let applyCandle = DepositPriceState.applyCandle
  let applyPriceTick = DepositPriceState.applyPriceTick
  let applyAssetSnapshot = DepositPriceState.applyAssetSnapshot
  let getCandles = DepositPriceState.getCandles
  let getLatestPrice = DepositPriceState.getLatestPrice
  let clear = DepositPriceState.clear
}

// The user's open limit orders, fed from `User(Snapshot(_))` (via
// `ofSnapshotOrders`, which seeds BOTH this and the trigger container) and
// `User(Order(Limit(_)))` events.
@genType
module LiveOpenLimitOrders = {
  let make = OrderState.UserOpenLimitOrders.make
  let get = OrderState.UserOpenLimitOrders.get
  let getByMarket = OrderState.UserOpenLimitOrders.getByMarket
  let insert = OrderState.UserOpenLimitOrders.insert
  let upsert = OrderState.UserOpenLimitOrders.upsert
  let remove = OrderState.UserOpenLimitOrders.remove
  let clear = OrderState.UserOpenLimitOrders.clear
  let isEmpty = OrderState.UserOpenLimitOrders.isEmpty
  let limitOrderOfUpdate = OrderState.limitOrderOfUpdate
  // Seeds (open limit orders, trigger orders) from a user snapshot's orders.
  let ofSnapshotOrders = OrderState.ofSnapshotOrders
}

// The user's resting trigger orders, fed from the snapshot seeding above and
// `User(Order(Trigger(_)))` events (via `triggerOrderOfUpdate`).
@genType
module LiveTriggerOrders = {
  let make = OrderState.UserTriggerOrders.make
  let get = OrderState.UserTriggerOrders.get
  let getByMarket = OrderState.UserTriggerOrders.getByMarket
  let all = OrderState.UserTriggerOrders.all
  let getById = OrderState.UserTriggerOrders.getById
  let insert = OrderState.UserTriggerOrders.insert
  let remove = OrderState.UserTriggerOrders.remove
  let clear = OrderState.UserTriggerOrders.clear
  let isEmpty = OrderState.UserTriggerOrders.isEmpty
  let size = OrderState.UserTriggerOrders.size
  let triggerOrderOfUpdate = OrderState.triggerOrderOfUpdate
  let limitPrice = OrderState.limitPrice
}

// A rolling, capped trade history per orderbook, fed from `Trades` frames and
// REST backfills.
@genType
module LiveTrades = {
  let make = TradeState.make
  let push = TradeState.push
  let replace = TradeState.replace
  let trades = TradeState.trades
  let latest = TradeState.latest
  let clear = TradeState.clear
  let size = TradeState.size
  let isEmpty = TradeState.isEmpty
}

// The user's balance index (market → deposit asset → conditional token), fed
// from `User(Snapshot(_))` market balances and `User(BalanceUpdate(_))` events.
@genType
module LiveUserBalances = {
  let make = Position.UserMarketBalanceIndex.make
  let get = Position.UserMarketBalanceIndex.get
  let insert = Position.UserMarketBalanceIndex.insert
  let remove = Position.UserMarketBalanceIndex.remove
  let extend = Position.UserMarketBalanceIndex.extend
  let marketPubkeys = Position.UserMarketBalanceIndex.marketPubkeys
  let ofMarketBalance = Position.UserMarketBalanceIndex.ofMarketBalance
  let ofMarketBalances = Position.UserMarketBalanceIndex.ofMarketBalances
}
