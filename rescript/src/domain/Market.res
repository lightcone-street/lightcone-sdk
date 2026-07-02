// Markets domain — market discovery / metadata queries (mirrors the Rust
// `domain/market`, plus the orderbook-pair structure that markets own).
//
// Reference shape (same as Trade.res):
//   1. wire types (`@spice`)  — exact JSON the backend sends
//   2. domain types (`@genType`) — the clean shape exported to TypeScript
//   3. `…OfResponse` conversions (mirror the Rust `TryFrom<Wire> for Domain`)
//   4. client functions taking a `Client.t`, returning `promise<result<_, sdkError>>`
//
// Decimal-valued fields (prices, sizes, min order sizes) stay as wire strings.
// Ids / counts / indices are floats. Timestamps arrive as ISO-8601 strings
// (chrono `DateTime<Utc>`) and are parsed to unix-ms floats in the conversions.
//
// `orderBookPair` (and its `orderbookResponse` wire form) live HERE rather than
// in Orderbook.res: the Rust orderbook→pair conversion needs market token types
// (`ConditionalToken`), so keeping it in Market.res avoids a module cycle. Other
// modules reference it as `Market.orderBookPair`.

// ── Domain-local enums ────────────────────────────────────────────────────────

// Market lifecycle status. Wire value = the PascalCase variant name (serde
// default, no rename_all).
module Status = {
  @spice
  type t =
    | @as("Pending") @spice.as("Pending") Pending
    | @as("Active") @spice.as("Active") Active
    | @as("Resolved") @spice.as("Resolved") Resolved
    | @as("Cancelled") @spice.as("Cancelled") Cancelled

  let toString = (status: t) =>
    switch status {
    | Pending => "Pending"
    | Active => "Active"
    | Resolved => "Resolved"
    | Cancelled => "Cancelled"
    }

  let fromString = (value: string): option<t> =>
    switch value {
    | "Pending" => Some(Pending)
    | "Active" => Some(Active)
    | "Resolved" => Some(Resolved)
    | "Cancelled" => Some(Cancelled)
    | _ => None
    }
}

// Market resolution kind (serde `rename_all = "snake_case"`).
module ResolutionKind = {
  @spice
  type t =
    | @as("single_winner") @spice.as("single_winner") SingleWinner
    | @as("scalar") @spice.as("scalar") Scalar
}

// ── Resolution wire+domain types (passed through unchanged, so shared) ─────────
@spice
type marketResolutionPayout = {
  @spice.key("outcome_index") outcomeIndex: float,
  @spice.key("payout_numerator") payoutNumerator: float,
}

@spice
type marketResolutionResponse = {
  kind: ResolutionKind.t,
  @spice.key("payout_denominator") payoutDenominator: float,
  payouts: array<marketResolutionPayout>,
  @spice.key("single_winning_outcome") singleWinningOutcome?: float,
}

// ── Wire types ────────────────────────────────────────────────────────────────
@spice
type outcomeResponse = {
  index: float,
  name: string,
  @spice.key("name_long") nameLong?: string,
  @spice.key("icon_url_low") iconUrlLow?: string,
  @spice.key("icon_url_medium") iconUrlMedium?: string,
  @spice.key("icon_url_high") iconUrlHigh?: string,
}

@spice
type conditionalTokenResponse = {
  id: float,
  @spice.key("outcome_index") outcomeIndex: float,
  @spice.key("token_address") tokenAddress: string,
  symbol?: string,
  uri?: string,
  outcome?: string,
  @spice.key("deposit_symbol") depositSymbol?: string,
  @spice.key("short_symbol") shortSymbol?: string,
  description?: string,
  @spice.key("icon_url_low") iconUrlLow?: string,
  @spice.key("icon_url_medium") iconUrlMedium?: string,
  @spice.key("icon_url_high") iconUrlHigh?: string,
  @spice.key("metadata_uri") metadataUri?: string,
  decimals?: float,
  @spice.key("created_at") createdAt: string,
}

@spice
type depositAssetResponse = {
  @spice.key("display_name") displayName?: string,
  @spice.key("token_symbol") tokenSymbol?: string,
  symbol?: string,
  @spice.key("deposit_asset") depositAsset: Shared.pubkeyStr,
  id: float,
  @spice.key("market_pubkey") marketPubkey: Shared.pubkeyStr,
  vault: string,
  @spice.key("num_outcomes") numOutcomes: float,
  description?: string,
  @spice.key("icon_url_low") iconUrlLow?: string,
  @spice.key("icon_url_medium") iconUrlMedium?: string,
  @spice.key("icon_url_high") iconUrlHigh?: string,
  @spice.key("metadata_uri") metadataUri?: string,
  decimals?: float,
  @spice.key("min_order_size") minOrderSize?: string,
  @spice.key("conditional_mints") conditionalMints: array<conditionalTokenResponse>,
  @spice.key("created_at") createdAt: string,
}

@spice
type depositMintsResponse = {
  @spice.key("market_pubkey") marketPubkey: Shared.pubkeyStr,
  @spice.key("deposit_assets") depositAssets: array<depositAssetResponse>,
  total: float,
}

// REST single-orderbook wire type. Consumed by the market conversion to build
// `orderBookPair`; not returned directly by any client function.
@spice
type orderbookResponse = {
  id: float,
  @spice.key("market_pubkey") marketPubkey: Shared.pubkeyStr,
  @spice.key("orderbook_id") orderbookId: Shared.orderBookId,
  @spice.key("base_token") baseToken: string,
  @spice.key("quote_token") quoteToken: string,
  @spice.key("outcome_index") outcomeIndex?: float,
  @spice.key("tick_size") tickSize: float,
  @spice.key("total_bids") totalBids: float,
  @spice.key("total_asks") totalAsks: float,
  @spice.key("last_trade_price") lastTradePrice?: string,
  @spice.key("last_trade_time") lastTradeTime?: string,
  active: bool,
  @spice.key("created_at") createdAt: string,
  @spice.key("updated_at") updatedAt: string,
}

@spice
type searchOrderbook = {
  @spice.key("orderbook_id") orderbookId: Shared.orderBookId,
  @spice.key("outcome_name") outcomeName: string,
  @spice.key("outcome_name_long") outcomeNameLong?: string,
  @spice.key("outcome_index") outcomeIndex: float,
  @spice.key("deposit_base_asset") depositBaseAsset: Shared.pubkeyStr,
  @spice.key("deposit_quote_asset") depositQuoteAsset: Shared.pubkeyStr,
  @spice.key("deposit_base_symbol") depositBaseSymbol: string,
  @spice.key("deposit_quote_symbol") depositQuoteSymbol: string,
  @spice.key("base_icon_url_low") baseIconUrlLow?: string,
  @spice.key("base_icon_url_medium") baseIconUrlMedium?: string,
  @spice.key("base_icon_url_high") baseIconUrlHigh?: string,
  @spice.key("quote_icon_url_low") quoteIconUrlLow?: string,
  @spice.key("quote_icon_url_medium") quoteIconUrlMedium?: string,
  @spice.key("quote_icon_url_high") quoteIconUrlHigh?: string,
  @spice.key("conditional_base_mint") conditionalBaseMint: Shared.pubkeyStr,
  @spice.key("conditional_quote_mint") conditionalQuoteMint: Shared.pubkeyStr,
  @spice.key("outcome_icon_url_low") outcomeIconUrlLow?: string,
  @spice.key("outcome_icon_url_medium") outcomeIconUrlMedium?: string,
  @spice.key("outcome_icon_url_high") outcomeIconUrlHigh?: string,
  @spice.key("conditional_base_symbol") conditionalBaseSymbol?: string,
  @spice.key("conditional_quote_symbol") conditionalQuoteSymbol?: string,
  @spice.key("latest_mid_price") latestMidPrice?: string,
}

@spice
type marketSearchResult = {
  slug: string,
  @spice.key("market_name") marketName: string,
  @spice.key("market_status") marketStatus: Status.t,
  category?: string,
  @spice.default([]) tags: array<string>,
  @spice.key("featured_rank") featuredRank: float,
  description?: string,
  @spice.key("icon_url_low") iconUrlLow?: string,
  @spice.key("icon_url_medium") iconUrlMedium?: string,
  @spice.key("icon_url_high") iconUrlHigh?: string,
  orderbooks: array<searchOrderbook>,
}

@spice
type marketResponse = {
  @spice.key("market_name") marketName?: string,
  slug?: string,
  description?: string,
  definition?: string,
  outcomes: array<outcomeResponse>,
  @spice.key("banner_image_url_low") bannerImageUrlLow?: string,
  @spice.key("banner_image_url_medium") bannerImageUrlMedium?: string,
  @spice.key("banner_image_url_high") bannerImageUrlHigh?: string,
  @spice.key("icon_url_low") iconUrlLow?: string,
  @spice.key("icon_url_medium") iconUrlMedium?: string,
  @spice.key("icon_url_high") iconUrlHigh?: string,
  category?: string,
  tags?: array<string>,
  @spice.key("featured_rank") featuredRank?: float,
  @spice.key("market_pubkey") marketPubkey: Shared.pubkeyStr,
  @spice.key("market_id") marketId: float,
  oracle: string,
  @spice.key("question_id") questionId: string,
  @spice.key("condition_id") conditionId: string,
  @spice.key("market_status") marketStatus: string,
  resolution?: marketResolutionResponse,
  @spice.key("created_at") createdAt: string,
  @spice.key("activated_at") activatedAt?: string,
  @spice.key("settled_at") settledAt?: string,
  @spice.key("deposit_assets") depositAssets: array<depositAssetResponse>,
  orderbooks: array<orderbookResponse>,
}

@spice
type marketsResponse = {
  markets: array<marketResponse>,
  @spice.key("next_cursor") nextCursor?: float,
  @spice.key("has_more") hasMore: bool,
}

@spice
type singleMarketResponse = {market: marketResponse}

@spice
type globalDepositAssetResponse = {
  id: float,
  mint: Shared.pubkeyStr,
  @spice.key("display_name") displayName?: string,
  symbol?: string,
  description?: string,
  @spice.key("icon_url_low") iconUrlLow?: string,
  @spice.key("icon_url_medium") iconUrlMedium?: string,
  @spice.key("icon_url_high") iconUrlHigh?: string,
  decimals?: float,
  @spice.key("whitelist_index") whitelistIndex: float,
  active: bool,
}

@spice
type globalDepositAssetsListResponse = {
  assets: array<globalDepositAssetResponse>,
  total: float,
}

// ── Domain types ──────────────────────────────────────────────────────────────
type outcome = {
  index: float,
  iconUrlLow: string,
  iconUrlMedium: string,
  iconUrlHigh: string,
  name: string,
  nameLong?: string,
}

type conditionalToken = {
  id: float,
  outcomeIndex: float,
  outcome: string,
  depositAsset: Shared.pubkeyStr,
  depositSymbol: string,
  mint: Shared.pubkeyStr,
  name: string,
  symbol: string,
  shortSymbol: string,
  description?: string,
  decimals: float,
  iconUrlLow: string,
  iconUrlMedium: string,
  iconUrlHigh: string,
}

type depositAsset = {
  id: float,
  marketPda: Shared.pubkeyStr,
  depositAsset: Shared.pubkeyStr,
  numOutcomes: float,
  name: string,
  symbol: string,
  shortSymbol: string,
  description?: string,
  decimals: float,
  minOrderSize?: string,
  iconUrlLow: string,
  iconUrlMedium: string,
  iconUrlHigh: string,
}

type tokenMetadata = {
  pubkey: Shared.pubkeyStr,
  symbol: string,
  shortSymbol: string,
  decimals: float,
  iconUrlLow: string,
  iconUrlMedium: string,
  iconUrlHigh: string,
  name: string,
}

type depositAssetPair = {
  // Stable identifier of the form `"{base_pubkey}-{quote_pubkey}"`.
  id: string,
  base: depositAsset,
  quote: depositAsset,
}

type globalDepositAsset = {
  id: float,
  depositAsset: Shared.pubkeyStr,
  name: string,
  symbol: string,
  shortSymbol: string,
  description?: string,
  decimals: float,
  iconUrlLow: string,
  iconUrlMedium: string,
  iconUrlHigh: string,
  whitelistIndex: float,
  active: bool,
}

// A tradeable conditional-token pair within a market. Owned by Market because
// its conversion needs the market's `conditionalToken` list.
type orderBookPair = {
  id: float,
  marketPubkey: Shared.pubkeyStr,
  orderbookId: Shared.orderBookId,
  base: conditionalToken,
  quote: conditionalToken,
  outcomeIndex: float,
  tickSize: float,
  totalBids: float,
  totalAsks: float,
  lastTradePrice?: string,
  // Unix milliseconds.
  lastTradeTime?: float,
  active: bool,
}

type market = {
  id: float,
  pubkey: Shared.pubkeyStr,
  name: string,
  bannerImageUrlLow: string,
  bannerImageUrlMedium: string,
  bannerImageUrlHigh: string,
  iconUrlLow: string,
  iconUrlMedium: string,
  iconUrlHigh: string,
  featuredRank?: float,
  slug: string,
  status: Status.t,
  // Unix milliseconds.
  createdAt: float,
  activatedAt?: float,
  settledAt?: float,
  resolution?: marketResolutionResponse,
  description: string,
  definition: string,
  category?: string,
  tags: array<string>,
  depositAssets: array<depositAsset>,
  // Unique base/quote deposit-asset pairs derived from `orderbookPairs`,
  // deduplicated by `(base, quote)` pubkey.
  depositAssetPairs: array<depositAssetPair>,
  conditionalTokens: array<conditionalToken>,
  outcomes: array<outcome>,
  orderbookPairs: array<orderBookPair>,
  orderbookIds: array<Shared.orderBookId>,
  // Keyed by token pubkey.
  tokenMetadata: dict<tokenMetadata>,
}

// Result of fetching multiple markets: valid markets plus any per-market
// validation errors (invalid markets are skipped, not fatal).
type marketsResult = {
  markets: array<market>,
  validationErrors: array<string>,
}

// Result of fetching the global deposit-asset whitelist: valid assets plus any
// per-asset validation errors.
type globalDepositAssetsResult = {
  assets: array<globalDepositAsset>,
  validationErrors: array<string>,
}

// ── Token helpers (market/tokens.rs) ──────────────────────────────────────────
// USD-stablecoin detection + display sorting for token-ish values.
let usdcMainnet: Shared.pubkeyStr = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
let usdtMainnet: Shared.pubkeyStr = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"
let usdcDevnetLc: Shared.pubkeyStr = "7SrxsoXjNR7Y8T3koJCt1yV4FrNUumoAUrJExDt6tQez"

let isUsdStablecoin = (pubkey: Shared.pubkeyStr): bool =>
  pubkey == usdcMainnet || pubkey == usdtMainnet || pubkey == usdcDevnetLc

// "$" for USD stablecoins, "" otherwise.
let currencySymbol = (pubkey: Shared.pubkeyStr): string => isUsdStablecoin(pubkey) ? "$" : ""

// Per-type conveniences mirroring the Rust methods: `isUsdStableCoin` checks the
// backing deposit asset; `…CurrencySymbol` checks the token's own mint.
let conditionalTokenIsUsdStableCoin = (token: conditionalToken): bool =>
  isUsdStablecoin(token.depositAsset)
let conditionalTokenCurrencySymbol = (token: conditionalToken): string => currencySymbol(token.mint)
let depositAssetIsUsdStableCoin = (asset: depositAsset): bool => isUsdStablecoin(asset.depositAsset)
let depositAssetCurrencySymbol = (asset: depositAsset): string => currencySymbol(asset.depositAsset)
let globalDepositAssetIsUsdStableCoin = (asset: globalDepositAsset): bool =>
  isUsdStablecoin(asset.depositAsset)
let globalDepositAssetCurrencySymbol = (asset: globalDepositAsset): string =>
  currencySymbol(asset.depositAsset)

// Display priority for sorting (lower first): BTC/WBTC tie at 0, ETH/WETH at 1,
// SOL at 2; everything else falls to the alphabetical tail.
let displayPriority = (symbol: string): int =>
  switch String.toUpperCase(symbol) {
  | "BTC" | "WBTC" => 0
  | "ETH" | "WETH" => 1
  | "SOL" => 2
  | _ => 255
  }

// A new array ordered for display: priority groups first, then the rest
// alphabetically by symbol. The Rust `HasDisplayToken` trait becomes the
// `~symbolOf` accessor — pass `token => token.symbol` for tokens or
// `pair => pair.base.symbol` for composite pairs.
let sortByDisplayPriority = (items: array<'a>, ~symbolOf: 'a => string): array<'a> =>
  items->Array.toSorted((left, right) => {
    let leftSymbol = symbolOf(left)
    let rightSymbol = symbolOf(right)
    let byPriority = Int.compare(displayPriority(leftSymbol), displayPriority(rightSymbol))
    Ordering.isEqual(byPriority) ? String.compare(leftSymbol, rightSymbol) : byPriority
  })

// ── OrderBookPair helpers (orderbook/mod.rs) ──────────────────────────────────
// Scaling decimals derived from the pair's token metadata — the recommended way
// to get `Scaling.orderbookDecimals` (no REST call needed).
let orderBookPairDecimals = (pair: orderBookPair): Scaling.orderbookDecimals => {
  let baseDecimals = Float.toInt(pair.base.decimals)
  let quoteDecimals = Float.toInt(pair.quote.decimals)
  {
    baseDecimals,
    quoteDecimals,
    priceDecimals: max(6 + quoteDecimals - baseDecimals, 0),
    tickSize: Math.max(pair.tickSize, 0.0),
  }
}

// Calculated impact of a conditional token's price vs its deposit asset.
type outcomeImpact = {
  sign: string,
  pct: float,
  // Absolute dollar difference, as a Decimal string.
  dollar: string,
  isPositive: bool,
}

let zeroImpact = {sign: "", pct: 0.0, dollar: "0", isPositive: false}

let parseDecimalOpt = (value: string): option<Decimal.t> =>
  switch Decimal.fromString(value) {
  | decimal => Some(decimal)
  | exception JsExn(_) => None
  }

// Price impact as a percentage relative to the deposit asset price: (pct, sign).
// Zero / malformed inputs yield (0.0, "").
let impactPct = (~depositPrice: string, ~conditionalPrice: string): (float, string) =>
  switch (parseDecimalOpt(depositPrice), parseDecimalOpt(conditionalPrice)) {
  | (Some(deposit), Some(conditional)) if !Decimal.isZero(deposit) && !Decimal.isZero(conditional) =>
    let value =
      Decimal.div(Decimal.minus(conditional, deposit), deposit)
      ->Decimal.times(Decimal.fromInt(100))
      ->Decimal.toNumber
    (value, value > 0.0 ? "+" : "")
  | _ => (0.0, "")
  }

// Full impact calculation with sign, percentage, and absolute dollar difference.
// A zero / malformed deposit price yields the zero impact.
let impact = (~depositAssetPrice: string, ~conditionalPrice: string): outcomeImpact =>
  switch (parseDecimalOpt(depositAssetPrice), parseDecimalOpt(conditionalPrice)) {
  | (Some(deposit), Some(conditional)) if !Decimal.isZero(deposit) =>
    let pct =
      Decimal.div(Decimal.minus(conditional, deposit), deposit)
      ->Decimal.times(Decimal.fromInt(100))
      ->Decimal.toNumber
    {
      sign: pct > 0.0 ? "+" : "-",
      isPositive: pct > 0.0,
      pct: Math.abs(pct),
      dollar: Decimal.minus(conditional, deposit)->Decimal.abs->Decimal.toString,
    }
  | _ => zeroImpact
  }

// ── Denominator pair helpers (shared/mod.rs `impl Denominator`) ───────────────
// The conditional token a denomination refers to on this pair, plus its symbols.
let denominatorToken = (denominator: Shared.Denominator.t, pair: orderBookPair): conditionalToken =>
  switch denominator {
  | Base => pair.base
  | Quote => pair.quote
  }

let denominatorSymbol = (denominator: Shared.Denominator.t, pair: orderBookPair): string =>
  denominatorToken(denominator, pair).symbol

let denominatorDepositSymbol = (denominator: Shared.Denominator.t, pair: orderBookPair): string =>
  denominatorToken(denominator, pair).depositSymbol

// ── Conversions ───────────────────────────────────────────────────────────────
// ISO-8601 (`DateTime<Utc>`) → unix milliseconds. Invalid strings yield NaN
// rather than throwing.
let parseTimestamp = (isoTimestamp: string): float => Date.fromString(isoTimestamp)->Date.getTime
let parseTimestampOpt = (isoTimestamp: option<string>): option<float> =>
  isoTimestamp->Option.map(value => parseTimestamp(value))

let formatValidationErrors = (label: string, identifier: string, messages: array<string>): string =>
  `${label} validation errors (${identifier}):\n` ++
    messages->Array.map(message => `  - ${message}`)->Array.join("\n")

// Resolve three icon-url quality variants with cross-fallback. `None` only when
// all three inputs are absent.
let resolveIconUrls = (
  low: option<string>,
  medium: option<string>,
  high: option<string>,
): option<(string, string, string)> =>
  switch low->Option.orElse(medium)->Option.orElse(high) {
  | None => None
  | Some(fallback) =>
    Some((
      low->Option.orElse(medium)->Option.orElse(high)->Option.getOr(fallback),
      medium->Option.orElse(low)->Option.orElse(high)->Option.getOr(fallback),
      high->Option.orElse(medium)->Option.orElse(low)->Option.getOr(fallback),
    ))
  }

let outcomeOfResponse = (source: outcomeResponse): result<outcome, array<string>> =>
  switch resolveIconUrls(source.iconUrlLow, source.iconUrlMedium, source.iconUrlHigh) {
  | None => Error([`Missing icon URL for outcome: ${source.name}`])
  | Some((low, medium, high)) =>
    Ok({
      index: source.index,
      iconUrlLow: low,
      iconUrlMedium: medium,
      iconUrlHigh: high,
      name: source.name,
      nameLong: ?source.nameLong,
    })
  }

// Validated deposit asset + its conditional tokens + token metadata (mirrors the
// Rust `ValidatedTokens`). Internal: consumed by `marketOfResponse`.
type validatedTokens = {
  token: depositAsset,
  conditionals: array<conditionalToken>,
  metadata: dict<tokenMetadata>,
}

let validatedTokensOfResponse = (source: depositAssetResponse): result<validatedTokens, array<string>> => {
  let errors: array<string> = []
  let conditionals: array<conditionalToken> = []
  let metadata: dict<tokenMetadata> = Dict.make()
  let pubkey = source.depositAsset

  let (iconLow, iconMedium, iconHigh) = switch resolveIconUrls(
    source.iconUrlLow,
    source.iconUrlMedium,
    source.iconUrlHigh,
  ) {
  | Some(icons) => icons
  | None => {
      Array.push(errors, `Missing icon URL: ${source.depositAsset}`)
      ("", "", "")
    }
  }
  let shortSymbol = switch source.displayName {
  | Some(value) => value
  | None => {
      Array.push(errors, `Missing display name: ${source.depositAsset}`)
      ""
    }
  }
  let symbol = switch source.symbol {
  | Some(value) => value
  | None => {
      Array.push(errors, `Missing symbol: ${source.depositAsset}`)
      ""
    }
  }
  let decimals = switch source.decimals {
  | Some(value) => value
  | None => {
      Array.push(errors, `Missing decimals: ${source.depositAsset}`)
      0.0
    }
  }

  Dict.set(
    metadata,
    pubkey,
    {
      pubkey,
      symbol,
      shortSymbol,
      decimals,
      iconUrlLow: iconLow,
      iconUrlMedium: iconMedium,
      iconUrlHigh: iconHigh,
      name: shortSymbol,
    },
  )

  source.conditionalMints->Array.forEach(conditional => {
    let conditionalPubkey = conditional.tokenAddress
    let conditionalErrors: array<string> = []

    let conditionalDecimals = switch conditional.decimals {
    | Some(value) => value
    | None => {
        Array.push(conditionalErrors, `Missing decimals: ${conditionalPubkey}`)
        0.0
      }
    }
    let conditionalShortSymbol = switch conditional.shortSymbol->Option.orElse(conditional.symbol) {
    | Some(value) => value
    | None => {
        Array.push(conditionalErrors, `Missing short symbol: ${conditionalPubkey}`)
        ""
      }
    }
    let conditionalSymbol = conditional.symbol->Option.getOr(conditionalShortSymbol)
    let conditionalOutcome = switch conditional.outcome {
    | Some(value) => value
    | None => {
        Array.push(conditionalErrors, `Missing outcome: ${conditionalPubkey}`)
        ""
      }
    }

    if Array.length(conditionalErrors) > 0 {
      Array.push(errors, formatValidationErrors("Token", conditionalPubkey, conditionalErrors))
    } else {
      // Cross-fallback within the conditional's own icons, then fall back to the
      // parent deposit asset's icons.
      let (conditionalLow, conditionalMedium, conditionalHigh) = switch resolveIconUrls(
        conditional.iconUrlLow,
        conditional.iconUrlMedium,
        conditional.iconUrlHigh,
      ) {
      | Some(icons) => icons
      | None => (iconLow, iconMedium, iconHigh)
      }

      Dict.set(
        metadata,
        conditionalPubkey,
        {
          pubkey: conditionalPubkey,
          symbol: conditionalSymbol,
          shortSymbol: conditionalShortSymbol,
          decimals: conditionalDecimals,
          iconUrlLow: conditionalLow,
          iconUrlMedium: conditionalMedium,
          iconUrlHigh: conditionalHigh,
          name: conditionalOutcome,
        },
      )

      Array.push(
        conditionals,
        {
          id: conditional.id,
          depositSymbol: symbol,
          depositAsset: pubkey,
          outcomeIndex: conditional.outcomeIndex,
          iconUrlLow: conditionalLow,
          iconUrlMedium: conditionalMedium,
          iconUrlHigh: conditionalHigh,
          description: ?conditional.description,
          outcome: conditionalOutcome,
          mint: conditionalPubkey,
          name: conditionalOutcome,
          symbol: conditionalSymbol,
          shortSymbol: conditionalShortSymbol,
          decimals: conditionalDecimals,
        },
      )
    }
  })

  if Array.length(errors) > 0 {
    Error(errors)
  } else {
    Ok({
      token: {
        id: source.id,
        marketPda: source.marketPubkey,
        depositAsset: pubkey,
        numOutcomes: source.numOutcomes,
        name: shortSymbol,
        symbol,
        shortSymbol,
        description: ?source.description,
        decimals,
        minOrderSize: ?source.minOrderSize,
        iconUrlLow: iconLow,
        iconUrlMedium: iconMedium,
        iconUrlHigh: iconHigh,
      },
      conditionals,
      metadata,
    })
  }
}

let globalDepositAssetOfResponse = (source: globalDepositAssetResponse): result<globalDepositAsset, string> => {
  let errors: array<string> = []

  let name = switch source.displayName {
  | Some(value) => value
  | None => {
      Array.push(errors, `Missing display name: ${source.mint}`)
      ""
    }
  }
  let symbol = switch source.symbol {
  | Some(value) => value
  | None => {
      Array.push(errors, `Missing symbol: ${source.mint}`)
      ""
    }
  }
  let (low, medium, high) = switch resolveIconUrls(
    source.iconUrlLow,
    source.iconUrlMedium,
    source.iconUrlHigh,
  ) {
  | Some(icons) => icons
  | None => {
      Array.push(errors, `Missing icon URL: ${source.mint}`)
      ("", "", "")
    }
  }
  let decimals = switch source.decimals {
  | Some(value) => value
  | None => {
      Array.push(errors, `Missing decimals: ${source.mint}`)
      0.0
    }
  }

  if Array.length(errors) > 0 {
    Error(formatValidationErrors("Token", source.mint, errors))
  } else {
    Ok({
      id: source.id,
      depositAsset: source.mint,
      shortSymbol: name,
      name,
      symbol,
      description: ?source.description,
      decimals,
      iconUrlLow: low,
      iconUrlMedium: medium,
      iconUrlHigh: high,
      whitelistIndex: source.whitelistIndex,
      active: source.active,
    })
  }
}

// Match an orderbook's base/quote mints against the market's conditional tokens.
let orderBookPairOfResponse = (
  source: orderbookResponse,
  tokens: array<conditionalToken>,
): result<orderBookPair, array<string>> => {
  let errors: array<string> = []
  let baseMint = source.baseToken
  let quoteMint = source.quoteToken
  let base = tokens->Array.find(token => token.mint == baseMint)
  let quote = tokens->Array.find(token => token.mint == quoteMint)

  switch base {
  | None =>
    Array.push(errors, `Base token not found: orderbook: ${source.orderbookId}, base: ${baseMint}`)
  | Some(_) => ()
  }
  switch quote {
  | None =>
    Array.push(errors, `Quote token not found: orderbook: ${source.orderbookId}, quote: ${quoteMint}`)
  | Some(_) => ()
  }

  switch (base, quote) {
  | (Some(baseToken), Some(quoteToken)) =>
    Ok({
      id: source.id,
      outcomeIndex: source.outcomeIndex->Option.getOr(baseToken.outcomeIndex),
      marketPubkey: source.marketPubkey,
      orderbookId: source.orderbookId,
      base: baseToken,
      quote: quoteToken,
      tickSize: source.tickSize,
      totalBids: source.totalBids,
      totalAsks: source.totalAsks,
      lastTradePrice: ?source.lastTradePrice,
      lastTradeTime: ?parseTimestampOpt(source.lastTradeTime),
      active: source.active,
    })
  | _ => Error(errors)
  }
}

// Display priority for sorting: BTC/WBTC → 0, ETH/WETH → 1, SOL → 2, rest → tail.
let displayPriority = (symbol: string): int =>
  switch String.toUpperCase(symbol) {
  | "BTC" | "WBTC" => 0
  | "ETH" | "WETH" => 1
  | "SOL" => 2
  | _ => 255
  }

// Order deposit-asset pairs by base-token display priority, then alphabetically.
let sortPairsByDisplayPriority = (pairs: array<depositAssetPair>): array<depositAssetPair> =>
  pairs->Array.toSorted((left, right) => {
    let leftPriority = displayPriority(left.base.symbol)
    let rightPriority = displayPriority(right.base.symbol)
    if leftPriority == rightPriority {
      String.compare(left.base.symbol, right.base.symbol)
    } else {
      Int.compare(leftPriority, rightPriority)
    }
  })

// Unique base/quote deposit-asset pairs across the market's orderbook pairs,
// deduplicated by `(base_pubkey, quote_pubkey)`. Pairs whose base or quote
// deposit asset is absent from `depositAssets` are skipped.
let deriveDepositAssetPairs = (
  depositAssets: array<depositAsset>,
  orderbookPairs: array<orderBookPair>,
): array<depositAssetPair> => {
  let seen: dict<depositAssetPair> = Dict.make()
  orderbookPairs->Array.forEach(pair => {
    let base = depositAssets->Array.find(asset => asset.depositAsset == pair.base.depositAsset)
    let quote = depositAssets->Array.find(asset => asset.depositAsset == pair.quote.depositAsset)
    switch (base, quote) {
    | (Some(baseAsset), Some(quoteAsset)) => {
        let key = `${baseAsset.depositAsset}-${quoteAsset.depositAsset}`
        switch Dict.get(seen, key) {
        | Some(_) => ()
        | None => Dict.set(seen, key, {id: key, base: baseAsset, quote: quoteAsset})
        }
      }
    | _ => ()
    }
  })
  Dict.valuesToArray(seen)
}

let marketOfResponse = (source: marketResponse): result<market, string> => {
  let errors: array<string> = []

  // Outcomes
  let outcomes: array<outcome> = []
  source.outcomes->Array.forEach(outcomeResponse =>
    switch outcomeOfResponse(outcomeResponse) {
    | Ok(validated) => Array.push(outcomes, validated)
    | Error(messages) => messages->Array.forEach(message => Array.push(errors, `Outcome: ${message}`))
    }
  )

  // Tokens (deposit assets + conditional tokens + metadata)
  let depositAssets: array<depositAsset> = []
  let conditionalTokens: array<conditionalToken> = []
  let tokenMetadata: dict<tokenMetadata> = Dict.make()
  source.depositAssets->Array.forEach(depositAssetResponse =>
    switch validatedTokensOfResponse(depositAssetResponse) {
    | Ok(validated) => {
        Array.push(depositAssets, validated.token)
        validated.conditionals->Array.forEach(conditional =>
          Array.push(conditionalTokens, conditional)
        )
        validated.metadata
        ->Dict.toArray
        ->Array.forEach(((key, value)) => Dict.set(tokenMetadata, key, value))
      }
    | Error(messages) => messages->Array.forEach(message => Array.push(errors, `Token: ${message}`))
    }
  )

  // Sort deposit assets and conditional tokens by symbol (matches Rust).
  let depositAssets = depositAssets->Array.toSorted((left, right) =>
    String.compare(left.symbol, right.symbol)
  )
  let conditionalTokens = conditionalTokens->Array.toSorted((left, right) =>
    String.compare(left.symbol, right.symbol)
  )

  // Orderbooks (resolved against the sorted conditional tokens).
  let orderbookPairs: array<orderBookPair> = []
  source.orderbooks->Array.forEach(orderbookResponse =>
    switch orderBookPairOfResponse(orderbookResponse, conditionalTokens) {
    | Ok(pair) => Array.push(orderbookPairs, pair)
    | Error(messages) => messages->Array.forEach(message => Array.push(errors, `OrderBook: ${message}`))
    }
  )

  let slug = switch source.slug {
  | Some(value) => value
  | None => {
      Array.push(errors, "Missing slug")
      ""
    }
  }
  let name = switch source.marketName {
  | Some(value) => value
  | None => {
      Array.push(errors, "Missing name")
      ""
    }
  }
  let status = switch Status.fromString(source.marketStatus) {
  | Some(value) => value
  | None => {
      Array.push(errors, "Invalid status")
      Status.Pending
    }
  }
  let description = switch source.description {
  | Some(value) => value
  | None => {
      Array.push(errors, "Missing description")
      ""
    }
  }
  let definition = switch source.definition {
  | Some(value) => value
  | None => {
      Array.push(errors, "Missing definition")
      ""
    }
  }
  let (iconLow, iconMedium, iconHigh) = switch resolveIconUrls(
    source.iconUrlLow,
    source.iconUrlMedium,
    source.iconUrlHigh,
  ) {
  | Some(icons) => icons
  | None => {
      Array.push(errors, "Missing icon URL")
      ("", "", "")
    }
  }
  let (bannerLow, bannerMedium, bannerHigh) = switch resolveIconUrls(
    source.bannerImageUrlLow,
    source.bannerImageUrlMedium,
    source.bannerImageUrlHigh,
  ) {
  | Some(icons) => icons
  | None => {
      Array.push(errors, "Missing banner URL")
      ("", "", "")
    }
  }

  let depositAssetPairs = sortPairsByDisplayPriority(
    deriveDepositAssetPairs(depositAssets, orderbookPairs),
  )
  if Array.length(depositAssetPairs) == 0 {
    Array.push(errors, "Missing deposit asset pairs")
  }

  if Array.length(errors) > 0 {
    Error(formatValidationErrors("Market", source.marketPubkey, errors))
  } else {
    Ok({
      id: source.marketId,
      pubkey: source.marketPubkey,
      featuredRank: ?source.featuredRank,
      slug,
      name,
      status,
      createdAt: parseTimestamp(source.createdAt),
      activatedAt: ?parseTimestampOpt(source.activatedAt),
      settledAt: ?parseTimestampOpt(source.settledAt),
      resolution: ?source.resolution,
      description,
      definition,
      tags: source.tags->Option.getOr([]),
      outcomes,
      iconUrlLow: iconLow,
      iconUrlMedium: iconMedium,
      iconUrlHigh: iconHigh,
      bannerImageUrlLow: bannerLow,
      bannerImageUrlMedium: bannerMedium,
      bannerImageUrlHigh: bannerHigh,
      category: ?source.category,
      orderbookIds: orderbookPairs->Array.map(pair => pair.orderbookId),
      orderbookPairs,
      depositAssets,
      depositAssetPairs,
      conditionalTokens,
      tokenMetadata,
    })
  }
}

// ── Market resolution helpers (mirror the Rust `Market` impl) ──────────────────
let isResolved = (market: market): bool => market.resolution->Option.isSome

let singleWinningOutcome = (market: market): option<float> =>
  market.resolution->Option.flatMap(resolution => resolution.singleWinningOutcome)

let hasSingleWinningOutcome = (market: market): bool => singleWinningOutcome(market)->Option.isSome

// ── Client functions ──────────────────────────────────────────────────────────
let optionalQuery = (query, key, value) =>
  value->Option.forEach(value => query->Array.push((key, value)))

// Cursor-paginated markets. Only Active and Resolved markets are kept; markets
// that fail validation are skipped and surfaced in `validationErrors`.
let get = async (
  client: Client.t,
  ~cursor: option<float>=?,
  ~limit: option<int>=?,
): result<marketsResult, SdkError.t> => {
  let query: array<(string, string)> = []
  optionalQuery(query, "cursor", cursor->Option.map(value => Float.toString(value)))
  optionalQuery(query, "limit", limit->Option.map(value => Int.toString(value)))
  (await Http.get(client.http, ~path="/api/markets", ~query, ~decode=marketsResponse_decode))->Result.map(
    response => {
      let markets: array<market> = []
      let validationErrors: array<string> = []
      response.markets->Array.forEach(marketResponse =>
        switch marketOfResponse(marketResponse) {
        | Ok(market) =>
          switch market.status {
          | Status.Active | Status.Resolved => Array.push(markets, market)
          | Status.Pending | Status.Cancelled => ()
          }
        | Error(message) => Array.push(validationErrors, message)
        }
      )
      {markets, validationErrors}
    },
  )
}

// Featured markets. Only Active markets are returned.
let featured = async (client: Client.t): result<array<marketSearchResult>, SdkError.t> =>
  (await Http.get(
    client.http,
    ~path="/api/markets/search/featured",
    ~decode=(json => Spice.arrayFromJson(marketSearchResult_decode, json)),
  ))->Result.map(results =>
    results->Array.filter(result =>
      switch result.marketStatus {
      | Status.Active => true
      | _ => false
      }
    )
  )

// Fetch a single market by slug.
let getBySlug = async (client: Client.t, ~slug: string): result<market, SdkError.t> =>
  switch await Http.get(
    client.http,
    ~path=`/api/markets/by-slug/${slug}`,
    ~decode=singleMarketResponse_decode,
  ) {
  | Error(error) => Error(error)
  | Ok(response) =>
    marketOfResponse(response.market)->Result.mapError(message => SdkError.Validation(message))
  }

// Fetch a single market by on-chain pubkey.
let getByPubkey = async (client: Client.t, ~pubkey: string): result<market, SdkError.t> =>
  switch await Http.get(
    client.http,
    ~path=`/api/markets/${pubkey}`,
    ~decode=singleMarketResponse_decode,
  ) {
  | Error(error) => Error(error)
  | Ok(response) =>
    marketOfResponse(response.market)->Result.mapError(message => SdkError.Validation(message))
  }

// Search markets by query string.
let search = async (
  client: Client.t,
  ~query: string,
  ~limit: option<int>=?,
): result<array<marketSearchResult>, SdkError.t> => {
  let encoded = encodeURIComponent(query)
  let queryParams: array<(string, string)> = []
  optionalQuery(queryParams, "limit", limit->Option.map(value => Int.toString(value)))
  await Http.get(
    client.http,
    ~path=`/api/markets/search/by-query/${encoded}`,
    ~query=queryParams,
    ~decode=(json => Spice.arrayFromJson(marketSearchResult_decode, json)),
  )
}

// The active global deposit-asset whitelist (platform-scoped). Assets that fail
// validation are skipped and surfaced in `validationErrors`.
let globalDepositAssets = async (client: Client.t): result<globalDepositAssetsResult, SdkError.t> =>
  (await Http.get(
    client.http,
    ~path="/api/global-deposit-assets",
    ~decode=globalDepositAssetsListResponse_decode,
  ))->Result.map(response => {
    let assets: array<globalDepositAsset> = []
    let validationErrors: array<string> = []
    response.assets->Array.forEach(assetResponse =>
      switch globalDepositAssetOfResponse(assetResponse) {
      | Ok(asset) => Array.push(assets, asset)
      | Error(message) => Array.push(validationErrors, message)
      }
    )
    {assets, validationErrors}
  })

// Deposit assets registered for a specific market (with their conditional mints).
let getDepositMints = async (
  client: Client.t,
  ~marketPubkey: string,
): result<depositMintsResponse, SdkError.t> =>
  await Http.get(
    client.http,
    ~path=`/api/markets/${marketPubkey}/deposit-assets`,
    ~decode=depositMintsResponse_decode,
  )
