// Positions domain — portfolio, token-balance, and market-position queries
// (mirrors the Rust `domain/position`).
//
// Reference shape for every read domain (see `Trade.res`):
//   1. wire types (`@spice`)       — exact JSON the backend sends
//   2. domain types (`@genType`)   — the clean shape exported to TypeScript
//   3. `…OfResponse` conversions   — mirror the Rust `From<Wire> for Domain`
//   4. client functions taking a `Client.t`, returning `promise<result<_, sdkError>>`
//
// Balances/amounts stay as wire strings (Rust `Decimal`, `serde-str` → JSON
// string; no precision loss, gentype-clean). Ids / counts / decimals are floats
// (JS numbers). Note: the REST read endpoints below return the wire `*Response`
// shapes directly (exactly as the Rust client does). The `Portfolio` / `Position`
// / `WalletHolding` / `TokenBalance` family below are the domain-level types
// re-exported by Rust's `domain::position` (the SDK prelude); they are NOT
// produced by these endpoints — they are the public type surface only.

// ── Wire types ────────────────────────────────────────────────────────────────
@spice
type outcomeBalance = {
  @spice.key("outcome_index") outcomeIndex: float,
  @spice.key("conditional_token") conditionalToken: Shared.pubkeyStr,
  balance: string,
  @spice.key("balance_idle") balanceIdle: string,
  @spice.key("balance_on_book") balanceOnBook: string,
}

@spice
type vaultBalance = {
  @spice.key("deposit_mint") depositMint: Shared.pubkeyStr,
  vault: Shared.pubkeyStr,
  balance: string,
}

@spice
type globalDeposit = {
  @spice.key("deposit_mint") depositMint: Shared.pubkeyStr,
  symbol: string,
  balance: string,
}

@spice
type positionEntry = {
  id: float,
  @spice.key("position_pubkey") positionPubkey: Shared.pubkeyStr,
  owner: Shared.pubkeyStr,
  @spice.key("market_pubkey") marketPubkey: Shared.pubkeyStr,
  outcomes: array<outcomeBalance>,
  @spice.default([]) @spice.key("vault_balances") vaultBalances: array<vaultBalance>,
  // ISO-8601 timestamps (the wire ships these as strings).
  @spice.key("created_at") createdAt: string,
  @spice.key("updated_at") updatedAt: string,
}

// Response for `GET /api/users/{user_pubkey}/positions`.
@spice
type positionsResponse = {
  owner: Shared.pubkeyStr,
  @spice.key("total_markets") totalMarkets: float,
  positions: array<positionEntry>,
  @spice.default([]) @spice.key("global_deposits") globalDeposits: array<globalDeposit>,
  // Mint pubkey → token decimals.
  decimals: dict<float>,
}

// Response for `GET /api/users/{user_pubkey}/markets/{market_pubkey}/positions`.
@spice
type marketPositionsResponse = {
  owner: Shared.pubkeyStr,
  @spice.key("market_pubkey") marketPubkey: Shared.pubkeyStr,
  positions: array<positionEntry>,
  @spice.default([]) @spice.key("global_deposits") globalDeposits: array<globalDeposit>,
  decimals: dict<float>,
}

// Combined balance + metadata for a deposit token. Both the wire row AND the
// domain shape — `depositTokenBalances` returns these directly (keyed by mint).
@spice
type depositTokenBalance = {
  mint: Shared.pubkeyStr,
  idle: string,
  symbol: string,
  name: string,
  @spice.key("icon_url_low") iconUrlLow?: string,
  @spice.key("icon_url_medium") iconUrlMedium?: string,
  @spice.key("icon_url_high") iconUrlHigh?: string,
}

// ── Domain types ──────────────────────────────────────────────────────────────
// (Re-exported public surface of Rust's `domain::position`; see header note.)

// One outcome within a position.
type positionOutcome = {
  conditionId: float,
  conditionName: string,
  tokenMint: Shared.pubkeyStr,
  amount: string,
  usdValue: string,
}

// A non-conditional token balance held in the user's wallet.
type walletHolding = {
  tokenMint: Shared.pubkeyStr,
  symbol: string,
  amount: string,
  decimals: float,
  usdValue: string,
  imgSrc: string,
}

// A user's position in a single market.
type position = {
  eventPubkey: Shared.pubkeyStr,
  eventName: string,
  eventImgSrc: string,
  outcomes: array<positionOutcome>,
  totalValue: string,
  // Unix milliseconds (Rust `DateTime<Utc>`).
  createdAt: float,
}

// A user's full portfolio across all markets.
type portfolio = {
  userAddress: Shared.pubkeyStr,
  walletHoldings: array<walletHolding>,
  positions: array<position>,
  totalWalletValue: string,
  totalPositionsValue: string,
}

// Classification of a token balance's source.
type tokenBalanceTokenType =
  | DepositAsset
  | ConditionalToken({
      orderbookId: Shared.orderBookId,
      marketPubkey: Shared.pubkeyStr,
      outcomeIndex: float,
    })

// A user's balance for a specific token.
type tokenBalance = {
  mint: Shared.pubkeyStr,
  idle: string,
  onBook: string,
  tokenType: tokenBalanceTokenType,
}

// Static metadata for a deposit asset (Rust `DepositAssetMetadata`).
@spice
type depositAssetMetadata = {
  symbol: string,
  @spice.key("short_symbol") shortSymbol: string,
  name: string,
  @spice.key("deposit_asset") depositAsset: Shared.pubkeyStr,
  @spice.key("icon_url_low") iconUrlLow: string,
  @spice.key("icon_url_medium") iconUrlMedium: string,
  @spice.key("icon_url_high") iconUrlHigh: string,
  description?: string,
  decimals: float,
}

// ── Decimal-string helpers (tolerant: malformed → zero) ──────────────────────
let decimalOrZero = (value: string): Decimal.t =>
  switch Decimal.fromString(value) {
  | decimal => decimal
  | exception JsExn(_) => Decimal.fromInt(0)
  }

let decimalIsPositive = (value: string): bool => Decimal.gt(decimalOrZero(value), Decimal.fromInt(0))

// ── Conditional balance delta ─────────────────────────────────────────────────
// One conditional-token balance from a WS user event, before it is folded into
// a balance index or token balance (Rust `ConditionalBalanceDelta`).
module ConditionalBalanceDelta = {
  type t = {
    marketPubkey: Shared.pubkeyStr,
    orderbookId?: Shared.orderBookId,
    outcomeIndex: float,
    conditionalToken: Shared.pubkeyStr,
    idle: string,
    onBook: string,
  }

  // idle + on-book, as a Decimal string.
  let total = (delta: t): string =>
    Decimal.plus(decimalOrZero(delta.idle), decimalOrZero(delta.onBook))->Decimal.toString

  // Neither idle nor on-book balance is positive.
  let isZero = (delta: t): bool =>
    !(decimalIsPositive(delta.idle) || decimalIsPositive(delta.onBook))
}

// ── Conversions ───────────────────────────────────────────────────────────────
// Mirrors Rust `impl From<DepositTokenBalance> for TokenBalance` (on_book = 0,
// classified as a deposit asset).
let tokenBalanceOfDepositTokenBalance = (value: depositTokenBalance): tokenBalance => {
  mint: value.mint,
  idle: value.idle,
  onBook: "0",
  tokenType: DepositAsset,
}

// Mirrors Rust `impl From<ConditionalBalanceDelta> for TokenBalance` (classified
// as a conditional token; a missing orderbook id becomes the empty default).
let tokenBalanceOfConditionalBalanceDelta = (delta: ConditionalBalanceDelta.t): tokenBalance => {
  mint: delta.conditionalToken,
  idle: delta.idle,
  onBook: delta.onBook,
  tokenType: ConditionalToken({
    orderbookId: delta.orderbookId->Option.getOr(""),
    marketPubkey: delta.marketPubkey,
    outcomeIndex: delta.outcomeIndex,
  }),
}

// Mirrors Rust `impl From<ConditionalBalanceDelta> for UserOutcomeBalance`
// (`balance` = idle + on-book).
let userOutcomeBalanceOfConditionalBalanceDelta = (
  delta: ConditionalBalanceDelta.t,
): Order.userOutcomeBalance => {
  outcomeIndex: delta.outcomeIndex,
  conditionalToken: delta.conditionalToken,
  balance: ConditionalBalanceDelta.total(delta),
  balanceIdle: delta.idle,
  balanceOnBook: delta.onBook,
}

// ── Computed display values ───────────────────────────────────────────────────
// Display strings for a conditional (base) balance at a given price (Rust
// `TokenBalance::computed_base`); everything formats through `Fmt.display`.
type tokenBalanceComputedBase = {
  value: string,
  size: string,
  price: string,
}

let computedBase = (balance: tokenBalance, ~conditionalPrice: string): tokenBalanceComputedBase => {
  let price = decimalOrZero(conditionalPrice)
  let size = Decimal.plus(decimalOrZero(balance.idle), decimalOrZero(balance.onBook))
  {
    value: Fmt.Decimal.display(Decimal.times(size, price)),
    size: Fmt.Decimal.display(size),
    price: Fmt.Decimal.display(price),
  }
}

// Display string for a quote balance: idle + on-book (Rust `computed_quote`).
let computedQuote = (balance: tokenBalance): string =>
  Fmt.Decimal.display(Decimal.plus(decimalOrZero(balance.idle), decimalOrZero(balance.onBook)))

// ── UserMarketBalanceIndex ────────────────────────────────────────────────────
// Nested balance lookup fed from WS user snapshots / balance updates (Rust
// `UserMarketBalanceIndex`): market → deposit asset → conditional token → balance.
// Zero balances are dropped on the way in; `extend` merges at the market level
// with the other's per-deposit-asset entries winning wholesale.
module UserMarketBalanceIndex = {
  // conditional token → balance.
  type conditionalTokenBalanceIndex = Dict.t<Order.userOutcomeBalance>
  // deposit asset → conditional token → balance.
  type depositAssetBalanceIndex = Dict.t<conditionalTokenBalanceIndex>
  // market → deposit asset → conditional token → balance.
  type t = Dict.t<depositAssetBalanceIndex>

  let make = (): t => Dict.make()

  let get = (index: t, ~marketPubkey: Shared.pubkeyStr): option<depositAssetBalanceIndex> =>
    index->Dict.get(marketPubkey)

  let insert = (index: t, ~marketPubkey: Shared.pubkeyStr, entry: depositAssetBalanceIndex): unit =>
    index->Dict.set(marketPubkey, entry)

  let remove = (index: t, ~marketPubkey: Shared.pubkeyStr): unit => index->Dict.delete(marketPubkey)

  // Merge `other` in: per market, per deposit asset, the other's entry wins wholesale.
  let extend = (index: t, other: t): unit =>
    other
    ->Dict.toArray
    ->Array.forEach(((marketPubkey, marketEntry)) => {
      let target = switch index->Dict.get(marketPubkey) {
      | Some(existing) => existing
      | None =>
        let created = Dict.make()
        index->Dict.set(marketPubkey, created)
        created
      }
      marketEntry
      ->Dict.toArray
      ->Array.forEach(((depositAsset, outcomes)) => target->Dict.set(depositAsset, outcomes))
    })

  // Indexed market pubkeys, sorted (deterministic iteration).
  let marketPubkeys = (index: t): array<Shared.pubkeyStr> =>
    index->Dict.keysToArray->Array.toSorted(String.compare)

  // Index a single market balance; `None` when every outcome is zero (Rust
  // `From<UserMarketBalance> for Option<UserMarketBalanceIndex>`).
  let ofMarketBalance = (marketBalance: Order.userMarketBalance): option<t> => {
    let marketEntry: depositAssetBalanceIndex = Dict.make()
    marketBalance.depositAssets->Array.forEach(depositAssetBalance => {
      let outcomes: conditionalTokenBalanceIndex = Dict.make()
      depositAssetBalance.outcomes->Array.forEach(outcome =>
        if !Order.userOutcomeBalanceIsZero(outcome) {
          outcomes->Dict.set(outcome.conditionalToken, outcome)
        }
      )
      if Dict.keysToArray(outcomes)->Array.length > 0 {
        marketEntry->Dict.set(depositAssetBalance.depositAsset, outcomes)
      }
    })
    switch Dict.keysToArray(marketEntry)->Array.length {
    | 0 => None
    | _ =>
      let index = make()
      index->Dict.set(marketBalance.marketPubkey, marketEntry)
      Some(index)
    }
  }

  // Index a full set of market balances (Rust `From<Vec<UserMarketBalance>>`).
  let ofMarketBalances = (marketBalances: array<Order.userMarketBalance>): t => {
    let index = make()
    marketBalances->Array.forEach(marketBalance =>
      switch ofMarketBalance(marketBalance) {
      | Some(marketIndex) => extend(index, marketIndex)
      | None => ()
      }
    )
    index
  }
}

// ── Client functions ──────────────────────────────────────────────────────────

// All positions for a user across every market. Public path-based endpoint.
let get = async (client: Client.t, ~userPubkey: string): result<positionsResponse, SdkError.t> =>
  await Http.get(
    client.http,
    ~path=`/api/users/${userPubkey}/positions`,
    ~decode=positionsResponse_decode,
  )

// Positions for a user in a specific market. Public path-based endpoint.
let getForMarket = async (
  client: Client.t,
  ~userPubkey: string,
  ~marketPubkey: string,
): result<marketPositionsResponse, SdkError.t> =>
  await Http.get(
    client.http,
    ~path=`/api/users/${userPubkey}/markets/${marketPubkey}/positions`,
    ~decode=marketPositionsResponse_decode,
  )

// All positions for the authenticated user (wallet resolved server-side from the
// auth cookie). Pass `~cookieHeader` to forward a per-request cookie (SSR).
let positions = async (
  client: Client.t,
  ~cookieHeader: option<string>=?,
): result<positionsResponse, SdkError.t> =>
  await Http.get(
    client.http,
    ~path="/api/users/positions",
    ~cookieHeader?,
    ~decode=positionsResponse_decode,
  )

// The authenticated user's positions in a specific market.
let positionsForMarket = async (
  client: Client.t,
  ~marketPubkey: string,
  ~cookieHeader: option<string>=?,
): result<marketPositionsResponse, SdkError.t> =>
  await Http.get(
    client.http,
    ~path=`/api/users/markets/${marketPubkey}/positions`,
    ~cookieHeader?,
    ~decode=marketPositionsResponse_decode,
  )

// SPL deposit-token balances for the authenticated user, keyed by mint pubkey.
// An empty map means the user holds none of the tracked balances (not an error).
let depositTokenBalances = async (
  client: Client.t,
  ~cookieHeader: option<string>=?,
): result<dict<depositTokenBalance>, SdkError.t> =>
  await Http.get(
    client.http,
    ~path="/api/users/deposit-token-balances",
    ~cookieHeader?,
    ~decode=json => Spice.dictFromJson(depositTokenBalance_decode, json),
  )

// Note: the on-chain builders live in `program/PositionBuilders.res` (every
// deposit / withdraw / merge / redeem flow, the position-token init/extend/close
// ops, plus the unsigned-transaction assembler) over the instruction builders in
// `program/Instructions.res`; the on-chain account reads (`get_onchain`) live in
// `Rpc.res` (getExchange / getMarket / getOrderbook / getPosition + PDA helpers).
