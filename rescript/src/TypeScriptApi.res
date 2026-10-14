// The TypeScript-facing SDK surface (gentype → TypeScriptApi.gen.ts; the package
// entry point). ReScript consumers use the result-returning domain modules directly
// (`Market.Client.featured(client)` → `promise<result<_, SdkError.t>>`); this module wraps
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
  ~transactionResources: option<V1Transaction.Resources.t>=?,
  ~sponsorshipEnabled=false,
  (),
): Client.t =>
  Client.make(
    ~env,
    ~baseUrl?,
    ~wsUrl?,
    ~rpcUrl?,
    ~backupRpcUrl?,
    ~programId?,
    ~depositSource,
    ~transactionResources?,
    ~sponsorshipEnabled,
    (),
  )

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
  Client.useExternalSigner(
    client,
    ~address=SolanaKit.address(address),
    ~signMessage,
    ~signTransaction,
  )

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

// Cache the maker's order nonce (fetch it with `RpcClient.nonce`); submits that
// don't carry an explicit nonce reuse the cached value.
@genType
let setOrderNonce = (client: Client.t, nonce: bigint): unit => Client.setOrderNonce(client, nonce)

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
  let credentials = Auth__Client.credentials
  let updateMaxSlippagePreference = (client, value) =>
    SdkError.unwrap(Auth__Client.updateMaxSlippagePreference(client, value))
  let setCredentialRestorer = (client: Client.t, restorer: option<unit => promise<bool>>) =>
    Http.setCredentialRestorer(client.http, restorer)
  let clearAuth = Client.clearAuth
  let tradingWallet = Auth__Model.tradingWallet
  let displayName = Auth__Model.displayName
  let walletDisplayName = Auth__Model.walletDisplayName
  let avatarUrl = Auth__Model.avatarUrl
  let classifyRegisterPrivyConflict = Auth__Model.classifyRegisterPrivyConflict
  let getNonce = (client: Client.t): promise<string> =>
    SdkError.unwrap(Auth.Client.getNonce(client))
  let login = (client: Client.t, ~useEmbeddedWallet: option<bool>=?): promise<
    Auth__Model.Session.t,
  > => SdkError.unwrap(Auth.Client.login(client, ~useEmbeddedWallet?))
  let checkSession = (client: Client.t, ~cookieHeader: option<string>=?): promise<
    Auth__Model.Session.t,
  > => SdkError.unwrap(Auth.Client.checkSession(client, ~cookieHeader?))
  let logout = (client: Client.t): promise<unit> => SdkError.unwrap(Auth.Client.logout(client))
  let isAuthenticated = (client: Client.t): bool => Auth.Client.isAuthenticated(client)
  let registerPrivy = (client: Client.t, request: Auth__Model.RegisterPrivyRequest.t) =>
    SdkError.unwrap(Auth.Client.registerPrivy(client, request))
  let disconnectX = (client: Client.t): promise<unit> =>
    SdkError.unwrap(Auth.Client.disconnectX(client))
  let connectXUrl = (client: Client.t): string => Auth.Client.connectXUrl(client)
}

// ── Markets ───────────────────────────────────────────────────────────────────
@genType
module MarketClient = {
  let favoriteMarkets = (client, ~limit=?, ~cursor=?, ~cookieHeader=?) =>
    SdkError.unwrap(Market__Client.favoriteMarkets(client, ~limit?, ~cursor?, ~cookieHeader?))
  let addFavoriteMarket = (client, ~marketPubkey, ~cookieHeader=?) =>
    SdkError.unwrap(Market__Client.addFavoriteMarket(client, ~marketPubkey, ~cookieHeader?))
  let removeFavoriteMarket = (client, ~marketPubkey, ~cookieHeader=?) =>
    SdkError.unwrap(Market__Client.removeFavoriteMarket(client, ~marketPubkey, ~cookieHeader?))
  let depositSymbol = Market__Model.depositSymbol
  let get = (client: Client.t, ~cursor: option<float>=?, ~limit: option<int>=?): promise<
    Market__Model.MarketsResult.t,
  > => SdkError.unwrap(Market.Client.get(client, ~cursor?, ~limit?))
  let featured = (client: Client.t): promise<array<Market__Raw.MarketSearchResult.t>> =>
    SdkError.unwrap(Market.Client.featured(client))
  let getBySlug = (client: Client.t, slug: string): promise<Market__Model.t> =>
    SdkError.unwrap(Market.Client.getBySlug(client, ~slug))
  let getByPubkey = (client: Client.t, pubkey: string): promise<Market__Model.t> =>
    SdkError.unwrap(Market.Client.getByPubkey(client, ~pubkey))
  let search = (client: Client.t, query: string, ~limit: option<int>=?): promise<
    array<Market__Raw.MarketSearchResult.t>,
  > => SdkError.unwrap(Market.Client.search(client, ~query, ~limit?))
  let globalDepositAssets = (client: Client.t): promise<
    Market__Model.GlobalDepositAssetsResult.t,
  > => SdkError.unwrap(Market.Client.globalDepositAssets(client))
  let depositMints = (client: Client.t, marketPubkey: string): promise<
    Market__Raw.DepositMintsResponse.t,
  > => SdkError.unwrap(Market.Client.getDepositMints(client, ~marketPubkey))
}

// ── Orderbook ─────────────────────────────────────────────────────────────────
@genType
module OrderbookClient = {
  let decimals = (client, orderbookId) =>
    SdkError.unwrap(Orderbook__Client.decimals(client, ~orderbookId))
  let invalidateDecimals = (client, orderbookId) =>
    Orderbook__Client.invalidateDecimals(client, ~orderbookId)
  let clearDecimalsCache = Orderbook__Client.clearDecimalsCache
  let getWithAggregation = (client, orderbookId, aggregation, ~depth=?, ~cookieHeader=?) =>
    SdkError.unwrap(
      Orderbook__Client.getWithAggregation(
        client,
        ~orderbookId,
        ~aggregation,
        ~depth?,
        ~cookieHeader?,
      ),
    )
  let get = (client: Client.t, orderbookId: string, ~depth: option<int>=?): promise<
    Orderbook__Raw.DepthResponse.t,
  > => SdkError.unwrap(Orderbook.Client.get(client, ~orderbookId, ~depth?))
}

// ── Trades ────────────────────────────────────────────────────────────────────
@genType
module TradeClient = {
  let forOrderbook = (
    client: Client.t,
    orderbookId: string,
    ~limit: option<int>=?,
    ~cursor: option<float>=?,
  ): promise<Trade__Model.Page.t> =>
    SdkError.unwrap(Trade.Client.get(client, ~orderbookId, ~limit?, ~cursor?))
  let forMarket = (
    client: Client.t,
    marketPubkey: string,
    ~limit: option<int>=?,
    ~cursor: option<float>=?,
  ): promise<Trade__Model.Page.t> =>
    SdkError.unwrap(Trade.Client.getByMarket(client, ~marketPubkey, ~limit?, ~cursor?))
}

// ── Orders ────────────────────────────────────────────────────────────────────
@genType
module OrderClient = {
  let forUser = (client: Client.t, ~limit: option<int>=?, ~cursor: option<string>=?): promise<
    Order__Raw.UserOrdersResponse.t,
  > => SdkError.unwrap(Order.Client.getUserOrders(client, ~limit?, ~cursor?))

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
    ~orderbookId: string,
    ~timeInForce: option<Shared.TimeInForce.t>=?,
  ): Order__Raw.SubmitResponse.t => {
    await SdkError.unwrap(
      Envelope.submitLimitOrderSigned(
        client,
        ~market=SolanaKit.address(market),
        ~baseMint=SolanaKit.address(baseMint),
        ~quoteMint=SolanaKit.address(quoteMint),
        ~side,
        ~price,
        ~size,
        ~orderbookId,
        ~timeInForce?,
      ),
    )
  }

  let cancel = (client: Client.t, ~orderHash: string): promise<Order__Raw.CancelSuccess.t> =>
    SdkError.unwrap(Order.Client.cancelSigned(client, ~orderHash))

  let cancelAll = (client: Client.t, ~orderbookId: string): promise<
    Order__Raw.CancelAllSuccess.t,
  > => SdkError.unwrap(Order.Client.cancelAllSigned(client, ~orderbookId))

  // The authenticated user's filled orders with nested fill events.
  let fills = (
    client: Client.t,
    ~marketPubkey: option<string>=?,
    ~limit: option<int>=?,
    ~cursor: option<string>=?,
  ): promise<Order__Raw.UserFillsResponse.t> =>
    SdkError.unwrap(Order.Client.getUserOrderFills(client, ~marketPubkey?, ~limit?, ~cursor?))

  // Public variant: any wallet's fills via the URL path (no auth).
  let fillsByWallet = (
    client: Client.t,
    ~walletAddress: string,
    ~marketPubkey: option<string>=?,
    ~limit: option<int>=?,
    ~cursor: option<string>=?,
  ): promise<Order__Raw.UserFillsResponse.t> =>
    SdkError.unwrap(
      Order.Client.getUserOrderFillsByWallet(
        client,
        ~walletAddress,
        ~marketPubkey?,
        ~limit?,
        ~cursor?,
      ),
    )
}

// ── Positions ─────────────────────────────────────────────────────────────────
@genType
module PositionClient = {
  let initPositionTokens = (client, ~payer, ~user, ~market, ~depositMints) =>
    SdkError.unwrap(
      Position__Builders.initPositionTokens(
        client,
        ~payer=SolanaKit.address(payer),
        ~user=SolanaKit.address(user),
        ~market=SolanaKit.address(market),
        ~depositMints=depositMints->Array.map(SolanaKit.address),
      ),
    )
  let forUser = (client: Client.t, userPubkey: string): promise<
    Position__Raw.PositionsResponse.t,
  > => SdkError.unwrap(Position.Client.get(client, ~userPubkey))
  let forMarket = (client: Client.t, ~userPubkey: string, ~marketPubkey: string): promise<
    Position__Raw.MarketPositionsResponse.t,
  > => SdkError.unwrap(Position.Client.getForMarket(client, ~userPubkey, ~marketPubkey))
  let mine = (client: Client.t): promise<Position__Raw.PositionsResponse.t> =>
    SdkError.unwrap(Position.Client.positions(client))
  let depositTokenBalances = (client: Client.t): promise<
    Position__Raw.DepositTokenBalancesSnapshot.t,
  > => SdkError.unwrap(Position.Client.depositTokenBalances(client))

  // Each builds + signs + sends a Solana transaction and returns the tx signature.
  let depositToGlobal = async (client: Client.t, ~mint: string, ~amount: bigint): string => {
    let user = signerAddressOrThrow(client)
    await SdkError.unwrap(
      Position.Builders.depositToGlobal(client, ~user, ~mint=SolanaKit.address(mint), ~amount),
    )
  }
  let withdrawFromGlobal = async (client: Client.t, ~mint: string, ~amount: bigint): string => {
    let user = signerAddressOrThrow(client)
    await SdkError.unwrap(
      Position.Builders.withdrawFromGlobal(client, ~user, ~mint=SolanaKit.address(mint), ~amount),
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
      Position.Builders.globalToMarketDeposit(
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
      Position.Builders.merge(
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
      Position.Builders.redeemWinnings(
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
      Position.Builders.deposit(
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
      Position.Builders.withdrawFromPosition(
        client,
        ~user,
        ~market=SolanaKit.address(market),
        ~depositMint=SolanaKit.address(mint),
        ~amount,
        ~outcomeIndex,
      ),
    )
  }

  let closePositionTokenAccounts = async (
    client: Client.t,
    ~market: string,
    ~position: string,
    ~depositMints: array<string>,
    ~numOutcomes: int,
  ): string => {
    let operator = signerAddressOrThrow(client)
    await SdkError.unwrap(
      Position.Builders.closePositionTokenAccounts(
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
  let platform = (client: Client.t): promise<Metrics__Raw.Platform.t> =>
    SdkError.unwrap(Metrics.Client.platform(client))
  let markets = (client: Client.t): promise<Metrics__Raw.Markets.t> =>
    SdkError.unwrap(Metrics.Client.markets(client))
  let market = (client: Client.t, marketPubkey: string): promise<Metrics__Raw.MarketDetail.t> =>
    SdkError.unwrap(Metrics.Client.market(client, ~marketPubkey))
  let orderbookTickers = (client: Client.t, ~depositAsset: option<string>=?): promise<
    Metrics__Raw.OrderbookTickersResponse.t,
  > => SdkError.unwrap(Metrics.Client.orderbookTickers(client, ~depositAsset?))
  let categories = (client: Client.t): promise<Metrics__Raw.Categories.t> =>
    SdkError.unwrap(Metrics.Client.categories(client))
  let depositTokens = (client: Client.t): promise<Metrics__Raw.DepositTokens.t> =>
    SdkError.unwrap(Metrics.Client.depositTokens(client))
  let leaderboard = (client: Client.t, ~limit: option<int>=?): promise<
    Metrics__Raw.Leaderboard.t,
  > => SdkError.unwrap(Metrics.Client.leaderboard(client, ~limit?))
  let orderbook = (client: Client.t, ~orderbookId: string): promise<
    Metrics__Raw.OrderbookVolume.t,
  > => SdkError.unwrap(Metrics.Client.orderbook(client, ~orderbookId))
  let depositTokensVolumeHistory = (
    client: Client.t,
    ~fromMs: option<float>=?,
    ~toMs: option<float>=?,
    ~limit: option<int>=?,
  ): promise<Metrics__Raw.DepositTokenVolumeHistory.t> =>
    SdkError.unwrap(Metrics.Client.depositTokensVolumeHistory(client, ~fromMs?, ~toMs?, ~limit?))
  let openInterestHistory = (
    client: Client.t,
    ~fromMs: option<float>=?,
    ~toMs: option<float>=?,
    ~limit: option<int>=?,
  ): promise<Metrics__Raw.OpenInterestHistory.t> =>
    SdkError.unwrap(Metrics.Client.openInterestHistory(client, ~fromMs?, ~toMs?, ~limit?))
  let uniqueTradersHistory = (
    client: Client.t,
    ~scope: option<Metrics__Raw.UniqueTradersHistoryScope.t>=?,
    ~scopeKey: option<string>=?,
    ~fromMs: option<float>=?,
    ~toMs: option<float>=?,
    ~limit: option<int>=?,
  ): promise<Metrics__Raw.UniqueTradersHistory.t> =>
    SdkError.unwrap(
      Metrics.Client.uniqueTradersHistory(client, ~scope?, ~scopeKey?, ~fromMs?, ~toMs?, ~limit?),
    )
  let history = (
    client: Client.t,
    ~scope: string,
    ~scopeKey: string,
    ~resolution: Shared.Resolution.t=Shared.Resolution.Hour1,
    ~fromMs: option<float>=?,
    ~toMs: option<float>=?,
    ~limit: option<int>=?,
  ): promise<Metrics__Raw.History.t> =>
    SdkError.unwrap(
      Metrics.Client.history(client, ~scope, ~scopeKey, ~resolution, ~fromMs?, ~toMs?, ~limit?),
    )
  let user = (client: Client.t): promise<Metrics__Raw.User.t> =>
    SdkError.unwrap(Metrics.Client.user(client))
  let userByWallet = (client: Client.t, ~walletAddress: string): promise<Metrics__Raw.User.t> =>
    SdkError.unwrap(Metrics.Client.userByWallet(client, ~walletAddress))
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
  ): promise<PriceHistory__Raw.OrderbookResponse.t> =>
    SdkError.unwrap(PriceHistory.Client.get(client, ~orderbookId, ~resolution, ~fromMs?, ~toMs?))
  let lineData = (
    client: Client.t,
    ~orderbookId: string,
    ~resolution: Shared.Resolution.t,
    ~fromMs: option<float>=?,
    ~toMs: option<float>=?,
    ~cursor: option<float>=?,
    ~limit: option<float>=?,
  ): promise<array<PriceHistory__Model.LineData.t>> =>
    SdkError.unwrap(
      PriceHistory.Client.getLineData(
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
    PriceHistory__Raw.DepositPricesSnapshotResponse.t,
  > => SdkError.unwrap(PriceHistory.Client.getDepositAssetPricesSnapshot(client))
}

// ── Notifications ─────────────────────────────────────────────────────────────
@genType
module NotificationClient = {
  let list = (client: Client.t): promise<array<Notification__Model.t>> =>
    SdkError.unwrap(Notification.Client.fetch(client))
  let dismiss = (client: Client.t, ~notificationId: string): promise<unit> =>
    SdkError.unwrap(Notification.Client.dismiss(client, ~notificationId))
}

// ── Referrals ─────────────────────────────────────────────────────────────────
@genType
module ReferralClient = {
  let status = (client: Client.t): promise<Referral__Model.Status.t> =>
    SdkError.unwrap(Referral.Client.getStatus(client))
  let redeem = (client: Client.t, code: string): promise<Referral__Model.RedeemResult.t> =>
    SdkError.unwrap(Referral.Client.redeem(client, ~code))
}

// ── Faucet ────────────────────────────────────────────────────────────────────
@genType
module FaucetClient = {
  let claim = (client: Client.t, walletAddress: string): promise<Faucet__Raw.Response.t> =>
    SdkError.unwrap(Faucet.Client.claim(client, ~walletAddress))
}

// ── Onchain reads + PDA derivation (over the @solana/kit RPC; pubkeys as base58) ──
@genType
module RpcClient = {
  // Which endpoint is currently serving reads ("primary" until a failover flips it).
  let activeRpc = (client: Client.t): RpcFailover.Active.t => Rpc.activeRpc(client)
  let latestBlockhash = (client: Client.t): promise<string> =>
    SdkError.unwrap(Rpc.getLatestBlockhash(client))
  let exchange = (client: Client.t): promise<Accounts.Exchange.t> =>
    SdkError.unwrap(Rpc.getExchange(client))
  let market = (client: Client.t, marketPubkey: string): promise<Accounts.Market.t> =>
    SdkError.unwrap(Rpc.getMarket(client, SolanaKit.address(marketPubkey)))
  let orderbook = (client: Client.t, baseMint: string, quoteMint: string): promise<
    Accounts.Orderbook.t,
  > =>
    SdkError.unwrap(
      Rpc.getOrderbook(
        client,
        ~mintA=SolanaKit.address(baseMint),
        ~mintB=SolanaKit.address(quoteMint),
      ),
    )
  let position = (client: Client.t, userPubkey: string, marketPubkey: string): promise<
    option<Accounts.Position.t>,
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
  @genType
  type wsConnection = Ws.t
  @genType
  type wsMessage = Messages.t
  @genType
  type wsSubscription = Subscriptions.SubscribeParams.t
  @genType
  type wsUnsubscription = Subscriptions.UnsubscribeParams.t

  let connect = (
    client: Client.t,
    ~onMessage: Messages.t => unit,
    ~onConnected: unit => unit=() => (),
    ~onError: SdkError.t => unit=_ => (),
  ): wsConnection => Ws.connect(~url=client.wsUrl, ~onMessage, ~onConnected, ~onError, ())
  let subscribe = (connection: wsConnection, subscription: Subscriptions.SubscribeParams.t): unit =>
    Ws.subscribe(connection, subscription)->ignore
  let unsubscribe = (
    connection: wsConnection,
    subscription: Subscriptions.UnsubscribeParams.t,
  ): unit => Ws.unsubscribe(connection, subscription)->ignore
  let stop = Ws.stop
  let restartConnection = Ws.restartConnection
  let isStopped = Ws.isStopped
  let disconnect = (connection: wsConnection): unit => Ws.disconnect(connection)
  let isConnected = (connection: wsConnection): bool => Ws.isConnected(connection)
  let readyState = (connection: wsConnection): Ws.ReadyState.t => Ws.readyState(connection)
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
  let withAggregation = Orderbook__State.withAggregation
  let beginGeneration = Orderbook__State.beginGeneration
  let bidsRaw = Orderbook__State.bidLevels
  let asksRaw = Orderbook__State.askLevels
  // Implementation-module paths (not `Orderbook.State`): these un-annotated lets
  // are genType-visible, and genType cannot follow namespace aliases in types.
  let make = Orderbook__State.make
  let apply = Orderbook__State.apply
  let bestBid = Orderbook__State.bestBid
  let bestAsk = Orderbook__State.bestAsk
  let midPrice = Orderbook__State.midPrice
  let spread = Orderbook__State.spread
  let bids = Orderbook__State.bids
  let asks = Orderbook__State.asks
  let isEmpty = Orderbook__State.isEmpty
  let seq = Orderbook__State.seq
  let orderbookId = Orderbook__State.orderbookId
  let clear = Orderbook__State.clear
}

@genType
module LivePriceHistory = {
  let make = PriceHistory__State.make
  let applySnapshot = PriceHistory__State.applySnapshot
  let applyUpdate = PriceHistory__State.applyUpdate
  let get = PriceHistory__State.get
  let clear = PriceHistory__State.clear
}

@genType
module LiveDepositPrice = {
  let make = PriceHistory__DepositState.make
  let applySnapshot = PriceHistory__DepositState.applySnapshot
  let applyCandle = PriceHistory__DepositState.applyCandle
  let applyPriceTick = PriceHistory__DepositState.applyPriceTick
  let applyAssetSnapshot = PriceHistory__DepositState.applyAssetSnapshot
  let getCandles = PriceHistory__DepositState.getCandles
  let getLatestPrice = PriceHistory__DepositState.getLatestPrice
  let clear = PriceHistory__DepositState.clear
}

@genType
module LiveOpenLimitOrders = {
  let make = Order__State.Limits.make
  let get = Order__State.Limits.get
  let getByMarket = Order__State.Limits.getByMarket
  let insert = Order__State.Limits.insert
  let upsert = Order__State.Limits.upsert
  let remove = Order__State.Limits.remove
  let clear = Order__State.Limits.clear
  let isEmpty = Order__State.Limits.isEmpty
  let limitOrderOfUpdate = Order__Raw.Update.toLimit
  let ofSnapshotOrders = Order__State.fromSnapshotOrders
}

@genType @genType
module // A rolling, capped trade history per orderbook, fed from `Trades` frames and
// REST backfills.

LiveTrades = {
  let make = Trade__State.make
  let push = Trade__State.push
  let replace = Trade__State.replace
  let trades = Trade__State.trades
  let latest = Trade__State.latest
  let clear = Trade__State.clear
  let size = Trade__State.size
  let isEmpty = Trade__State.isEmpty
}

// The user's balance index (market → deposit asset → conditional token), fed
// from `User(Snapshot(_))` market balances and `User(BalanceUpdate(_))` events.
@genType
module LiveUserBalances = {
  let make = Position__State.make
  let get = Position__State.get
  let insert = Position__State.insert
  let remove = Position__State.remove
  let extend = Position__State.extend
  let marketPubkeys = Position__State.marketPubkeys
  let ofMarketBalance = Position__State.fromMarketBalance
  let ofMarketBalances = Position__State.fromMarketBalances
}

@genType
module LiveWalletBalances = {
  let make = Position__WalletState.make
  let applyRestSnapshot = Position__WalletState.applyRestSnapshot
  let applyEvent = Position__WalletState.applyEvent
  let balances = Position__WalletState.balances
  let contextSlot = Position__WalletState.contextSlot
  let walletAddress = Position__WalletState.walletAddress
  let nativeSolBalance = Position__WalletState.nativeSolBalance
  let combinedSolBalance = Position__WalletState.combinedSolBalance
  let solBalanceBreakdown = Position__WalletState.solBalanceBreakdown
  let clear = Position__WalletState.clear
}
@genType
module TransactionClient = {
  let setResources = (client: Client.t, resources: V1Transaction.Resources.t) => {
    switch V1Transaction.Resources.validate(resources) {
    | Error(error) => SdkError.throwAsJsError(SdkError.toMessage(error), error)
    | Ok() => client.transactionResources = Some(resources)
    }
  }
  let context = client => SdkError.unwrap(Transactions.context(client))
  let build = (client, ~payer, ~instructions) =>
    SdkError.unwrap(Transactions.build(client, ~payer, ~instructions))
  let importBytes = (bytes, context) =>
    SdkError.unwrap(Promise.resolve(V1Transaction.fromWireBytes(bytes, context)))
  let toWireBytes = V1Transaction.toWireBytes
  let messageBytes = V1Transaction.messageBytes
  let requiredSigners = V1Transaction.requiredSigners
  let estimateFee = (client, tx) => SdkError.unwrap(Transactions.estimateFee(client, tx))
  let simulate = (client, tx) => SdkError.unwrap(Transactions.simulate(client, tx))
  let sign = (client, tx) => SdkError.unwrap(Transactions.signPrepared(client, tx))
  let submitSigned = (client, tx) => SdkError.unwrap(Transactions.submitSigned(client, tx))
  let signAndSubmit = (client, tx) =>
    SdkError.unwrap(Transactions.signAndSubmitPrepared(client, tx))
  let signAndSubmitConfirmed = (client, tx) =>
    SdkError.unwrap(Transactions.signAndSubmitConfirmed(client, tx))
  let confirm = (client, signature, ~lastValidBlockHeight=?) =>
    SdkError.unwrap(Transactions.confirm(client, signature, ~lastValidBlockHeight?))
}
@genType
module SolClient = {
  let availability = Position__Sol.availability
  let canonicalAccount = (client, wallet) =>
    SdkError.unwrap(Position__Sol.canonicalAccount(client, SolanaKit.address(wallet)))
  let planWrap = (client, amount, state) =>
    SdkError.unwrap(Position__Sol.planWrap(client, ~amount, ~state))
  let planUnwrapAll = (client, state) =>
    SdkError.unwrap(Position__Sol.planUnwrapAll(client, ~state))
  let planSplit = (client, market, numOutcomes, amount, state) =>
    SdkError.unwrap(
      Position__Sol.planSplit(
        client,
        ~market=SolanaKit.address(market),
        ~numOutcomes,
        ~amount,
        ~state,
      ),
    )
  let planMerge = (client, market, numOutcomes, amount, state) =>
    SdkError.unwrap(
      Position__Sol.planMerge(
        client,
        ~market=SolanaKit.address(market),
        ~numOutcomes,
        ~amount,
        ~state,
      ),
    )
  let planRedeem = (client, market, numOutcomes, outcomeIndex, amount, state) =>
    SdkError.unwrap(
      Position__Sol.planRedeem(
        client,
        ~market=SolanaKit.address(market),
        ~numOutcomes,
        ~outcomeIndex,
        ~amount,
        ~state,
      ),
    )
  let planNativeWithdrawal = (client, recipient, amount, state) =>
    SdkError.unwrap(
      Position__Sol.planNativeWithdrawal(
        client,
        ~recipient=SolanaKit.address(recipient),
        ~amount,
        ~state,
      ),
    )
  let submit = (client, plan) => SdkError.unwrap(Position__Sol.submit(client, plan))
}
