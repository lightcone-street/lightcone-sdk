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
  let registerPrivy = (client: Client.t): promise<unit> =>
    SdkError.unwrap(Auth.Client.registerPrivy(client))
  let disconnectX = (client: Client.t): promise<unit> =>
    SdkError.unwrap(Auth.Client.disconnectX(client))
  let connectXUrl = (client: Client.t): string => Auth.Client.connectXUrl(client)
}

// ── Markets ───────────────────────────────────────────────────────────────────
@genType
module MarketClient = {
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
    ~baseDecimals: int,
    ~quoteDecimals: int,
    ~priceDecimals: int,
    ~tickSize: float,
    ~orderbookId: string,
    ~timeInForce: option<Shared.TimeInForce.t>=?,
  ): Order__Raw.SubmitResponse.t => {
    let decimals: Scaling.OrderbookDecimals.t = {baseDecimals, quoteDecimals, priceDecimals, tickSize}
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
  ): Order__Raw.TriggerResponse.t => {
    let decimals: Scaling.OrderbookDecimals.t = {baseDecimals, quoteDecimals, priceDecimals, tickSize}
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

  let cancel = (client: Client.t, ~orderHash: string): promise<Order__Raw.CancelSuccess.t> =>
    SdkError.unwrap(Order.Client.cancelSigned(client, ~orderHash))

  let cancelTrigger = (client: Client.t, ~triggerOrderId: string): promise<
    Order__Raw.CancelTriggerSuccess.t,
  > => SdkError.unwrap(Order.Client.cancelTriggerSigned(client, ~triggerOrderId))

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
  let forUser = (client: Client.t, userPubkey: string): promise<
    Position__Raw.PositionsResponse.t,
  > => SdkError.unwrap(Position.Client.get(client, ~userPubkey))
  let forMarket = (client: Client.t, ~userPubkey: string, ~marketPubkey: string): promise<
    Position__Raw.MarketPositionsResponse.t,
  > => SdkError.unwrap(Position.Client.getForMarket(client, ~userPubkey, ~marketPubkey))
  let mine = (client: Client.t): promise<Position__Raw.PositionsResponse.t> =>
    SdkError.unwrap(Position.Client.positions(client))
  let depositTokenBalances = (client: Client.t): promise<
    dict<Position__Raw.DepositTokenBalance.t>,
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
      Position.Builders.extendPositionTokens(
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
      Position.Builders.closePositionAlt(
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
  @genType.opaque
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

// The user's open limit orders, fed from `User(Snapshot(_))` (via
// `ofSnapshotOrders`, which seeds BOTH this and the trigger container) and
// `User(Order(Limit(_)))` events.
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
  // Seeds (open limit orders, trigger orders) from a user snapshot's orders.
  let ofSnapshotOrders = Order__State.fromSnapshotOrders
}

// The user's resting trigger orders, fed from the snapshot seeding above and
// `User(Order(Trigger(_)))` events (converted with the update converter below).
@genType
module LiveTriggerOrders = {
  let make = Order__State.Triggers.make
  let get = Order__State.Triggers.get
  let getByMarket = Order__State.Triggers.getByMarket
  let all = Order__State.Triggers.all
  let getById = Order__State.Triggers.getById
  let insert = Order__State.Triggers.insert
  let remove = Order__State.Triggers.remove
  let clear = Order__State.Triggers.clear
  let isEmpty = Order__State.Triggers.isEmpty
  let size = Order__State.Triggers.size
  let triggerOrderOfUpdate = Order__Raw.TriggerUpdate.toTrigger
  let limitPrice = Order__Model.Trigger.limitPrice
}

// A rolling, capped trade history per orderbook, fed from `Trades` frames and
// REST backfills.
@genType
module LiveTrades = {
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
