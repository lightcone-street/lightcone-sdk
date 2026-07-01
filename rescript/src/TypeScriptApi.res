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

// The native signer's address as a base58 string (None until useNativeSigner is called).
@genType
let signerAddress = (client: Client.t): option<string> =>
  Client.signerAddress(client)->Option.map(SolanaKit.addressToString)

// Generic boundary helper — unwrap any result-promise into a throwing value-promise.
@genType
let unwrap = SdkError.unwrap

// Pull the native signer's (keypair, address) off the client, or throw (internal).
let nativeSigner = (client: Client.t): (SolanaKit.cryptoKeyPair, SolanaKit.address) =>
  switch client.signingStrategy {
  | Some(Client.NativeSigner({keypair, address})) => (keypair, address)
  | _ =>
    let error = SdkError.Signing(
      "no native signer set; call useNativeSigner(client, secretKey) first",
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
    let (keypair, maker) = nativeSigner(client)
    let decimals: Scaling.orderbookDecimals = {baseDecimals, quoteDecimals, priceDecimals, tickSize}
    await SdkError.unwrap(
      Envelope.submitLimitOrder(
        client,
        ~maker,
        ~market=SolanaKit.address(market),
        ~baseMint=SolanaKit.address(baseMint),
        ~quoteMint=SolanaKit.address(quoteMint),
        ~side,
        ~price,
        ~size,
        ~decimals,
        ~orderbookId,
        ~keypair,
        ~timeInForce?,
      ),
    )
  }

  let cancel = async (client: Client.t, ~orderHash: string): Order.cancelSuccess => {
    let (keypair, address) = nativeSigner(client)
    let body = await Order.cancelBodySigned(
      ~orderHash,
      ~maker=SolanaKit.addressToString(address),
      ~keypair,
    )
    await SdkError.unwrap(Order.cancel(client, body))
  }

  let cancelAll = async (client: Client.t, ~orderbookId: string): Order.cancelAllSuccess => {
    let (keypair, address) = nativeSigner(client)
    let body = await Order.cancelAllBodySigned(
      ~userPubkey=SolanaKit.addressToString(address),
      ~orderbookId,
      ~keypair,
    )
    await SdkError.unwrap(Order.cancelAll(client, body))
  }
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
    let (_keypair, user) = nativeSigner(client)
    await SdkError.unwrap(
      PositionBuilders.depositToGlobal(client, ~user, ~mint=SolanaKit.address(mint), ~amount),
    )
  }
  let withdrawFromGlobal = async (client: Client.t, ~mint: string, ~amount: bigint): string => {
    let (_keypair, user) = nativeSigner(client)
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
    let (_keypair, user) = nativeSigner(client)
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
    let (_keypair, user) = nativeSigner(client)
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
    let (_keypair, user) = nativeSigner(client)
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
}
