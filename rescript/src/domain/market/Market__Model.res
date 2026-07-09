// Market domain types — lifecycle status, resolution payouts, the validated
// token structures (deposit assets, conditional tokens, metadata), tradeable
// orderbook pairs, price-impact math, and the primary market type.
//
// Decimal-valued fields (prices, sizes, min order sizes) stay as wire strings
// (no precision loss, gentype-clean — wrap in `Decimal` for math). Ids / counts
// / indices are floats. Timestamps are unix-ms floats (parsed from the ISO-8601
// wire strings in `Market__Raw`).

// ── Domain-local enums ────────────────────────────────────────────────────────

// Market lifecycle status. Wire value = the PascalCase variant name.
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

// A market's resolution outcome. Wire-shaped (`@spice`, snake_case keys): the
// backend payload is embedded in the market domain type unchanged.
module Resolution = {
  // Resolution kind. Wire value = the snake_case variant name.
  module Kind = {
    @spice
    type t =
      | @as("single_winner") @spice.as("single_winner") SingleWinner
      | @as("scalar") @spice.as("scalar") Scalar
  }

  @spice
  type payout = {
    @spice.key("outcome_index") outcomeIndex: float,
    @spice.key("payout_numerator") payoutNumerator: float,
  }

  @spice
  type t = {
    kind: Kind.t,
    @spice.key("payout_denominator") payoutDenominator: float,
    payouts: array<payout>,
    @spice.key("single_winning_outcome") singleWinningOutcome?: float,
  }
}

// ── Token helpers ─────────────────────────────────────────────────────────────
// USD-stablecoin detection + display sorting for token-ish values.
let usdcMainnet: Shared.pubkeyStr = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"
let usdtMainnet: Shared.pubkeyStr = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB"
let usdcDevnetLc: Shared.pubkeyStr = "7SrxsoXjNR7Y8T3koJCt1yV4FrNUumoAUrJExDt6tQez"

let isUsdStablecoin = (pubkey: Shared.pubkeyStr): bool =>
  pubkey == usdcMainnet || pubkey == usdtMainnet || pubkey == usdcDevnetLc

// "$" for USD stablecoins, "" otherwise.
let currencySymbol = (pubkey: Shared.pubkeyStr): string => isUsdStablecoin(pubkey) ? "$" : ""

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
// alphabetically by symbol. Pass a `~symbolOf` accessor — `token => token.symbol`
// for tokens, or `pair => pair.base.symbol` for composite pairs.
let sortByDisplayPriority = (items: array<'a>, ~symbolOf: 'a => string): array<'a> =>
  items->Array.toSorted((left, right) => {
    let leftSymbol = symbolOf(left)
    let rightSymbol = symbolOf(right)
    let byPriority = Int.compare(displayPriority(leftSymbol), displayPriority(rightSymbol))
    Ordering.isEqual(byPriority) ? String.compare(leftSymbol, rightSymbol) : byPriority
  })

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

// ── Token types ───────────────────────────────────────────────────────────────
module Outcome = {
  type t = {
    index: float,
    iconUrlLow: string,
    iconUrlMedium: string,
    iconUrlHigh: string,
    name: string,
    nameLong?: string,
  }
}

module ConditionalToken = {
  type t = {
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

  // `isUsdStableCoin` checks the backing deposit asset; `currencySymbol` checks
  // the token's own mint.
  let isUsdStableCoin = (token: t): bool => isUsdStablecoin(token.depositAsset)
  let currencySymbol = (token: t): string => currencySymbol(token.mint)
}

module DepositAsset = {
  type t = {
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

  // Both check the backing deposit asset.
  let isUsdStableCoin = (asset: t): bool => isUsdStablecoin(asset.depositAsset)
  let currencySymbol = (asset: t): string => currencySymbol(asset.depositAsset)
}

module TokenMetadata = {
  type t = {
    pubkey: Shared.pubkeyStr,
    symbol: string,
    shortSymbol: string,
    decimals: float,
    iconUrlLow: string,
    iconUrlMedium: string,
    iconUrlHigh: string,
    name: string,
  }
}

module DepositAssetPair = {
  type t = {
    // Stable identifier of the form `"{base_pubkey}-{quote_pubkey}"`.
    id: string,
    base: DepositAsset.t,
    quote: DepositAsset.t,
  }
}

module GlobalDepositAsset = {
  type t = {
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

  // Both check the backing deposit asset.
  let isUsdStableCoin = (asset: t): bool => isUsdStablecoin(asset.depositAsset)
  let currencySymbol = (asset: t): string => currencySymbol(asset.depositAsset)
}

// A tradeable conditional-token pair within a market. Owned by the market
// domain rather than Orderbook: its wire conversion needs the market's
// conditional-token list, and keeping it here avoids a module cycle.
module OrderBookPair = {
  type t = {
    id: float,
    marketPubkey: Shared.pubkeyStr,
    orderbookId: Shared.orderBookId,
    base: ConditionalToken.t,
    quote: ConditionalToken.t,
    outcomeIndex: float,
    tickSize: float,
    totalBids: float,
    totalAsks: float,
    lastTradePrice?: string,
    // Unix milliseconds.
    lastTradeTime?: float,
    active: bool,
  }

  // Scaling decimals derived from the pair's token metadata — the recommended
  // way to get `Scaling.OrderbookDecimals.t` (no REST call needed).
  let decimals = (pair: t): Scaling.OrderbookDecimals.t => {
    let baseDecimals = Float.toInt(pair.base.decimals)
    let quoteDecimals = Float.toInt(pair.quote.decimals)
    {
      baseDecimals,
      quoteDecimals,
      priceDecimals: max(6 + quoteDecimals - baseDecimals, 0),
      tickSize: Math.max(pair.tickSize, 0.0),
    }
  }

  // The conditional token a denomination refers to on this pair, plus its
  // symbols.
  let denominatorToken = (denominator: Shared.Denominator.t, pair: t): ConditionalToken.t =>
    switch denominator {
    | Base => pair.base
    | Quote => pair.quote
    }

  let denominatorSymbol = (denominator: Shared.Denominator.t, pair: t): string =>
    denominatorToken(denominator, pair).symbol

  let denominatorDepositSymbol = (denominator: Shared.Denominator.t, pair: t): string =>
    denominatorToken(denominator, pair).depositSymbol
}

// ── Price impact ──────────────────────────────────────────────────────────────
// Calculated impact of a conditional token's price vs its deposit asset.
module Impact = {
  type t = {
    sign: string,
    pct: float,
    // Absolute dollar difference, as a Decimal string.
    dollar: string,
    isPositive: bool,
  }

  let zero = {sign: "", pct: 0.0, dollar: "0", isPositive: false}

  let parseDecimalOpt = (value: string): option<Decimal.t> =>
    switch Decimal.fromString(value) {
    | decimal => Some(decimal)
    | exception JsExn(_) => None
    }

  // Price impact as a percentage relative to the deposit asset price:
  // (pct, sign). Zero / malformed inputs yield (0.0, "").
  let pct = (~depositPrice: string, ~conditionalPrice: string): (float, string) =>
    switch (parseDecimalOpt(depositPrice), parseDecimalOpt(conditionalPrice)) {
    | (Some(deposit), Some(conditional)) if !Decimal.isZero(deposit) && !Decimal.isZero(conditional) =>
      let value =
        Decimal.div(Decimal.minus(conditional, deposit), deposit)
        ->Decimal.times(Decimal.fromInt(100))
        ->Decimal.toNumber
      (value, value > 0.0 ? "+" : "")
    | _ => (0.0, "")
    }

  // Full impact calculation with sign, percentage, and absolute dollar
  // difference. A zero / malformed deposit price yields the zero impact.
  let make = (~depositAssetPrice: string, ~conditionalPrice: string): t =>
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
    | _ => zero
    }
}

// ── Market ────────────────────────────────────────────────────────────────────
// The primary market domain type.
type t = {
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
  resolution?: Resolution.t,
  description: string,
  definition: string,
  category?: string,
  tags: array<string>,
  depositAssets: array<DepositAsset.t>,
  // Unique base/quote deposit-asset pairs derived from `orderbookPairs`,
  // deduplicated by `(base, quote)` pubkey.
  depositAssetPairs: array<DepositAssetPair.t>,
  conditionalTokens: array<ConditionalToken.t>,
  outcomes: array<Outcome.t>,
  orderbookPairs: array<OrderBookPair.t>,
  orderbookIds: array<Shared.orderBookId>,
  // Keyed by token pubkey.
  tokenMetadata: dict<TokenMetadata.t>,
}

// ── Market resolution helpers ─────────────────────────────────────────────────
let isResolved = (market: t): bool => market.resolution->Option.isSome

let singleWinningOutcome = (market: t): option<float> =>
  market.resolution->Option.flatMap(resolution => resolution.singleWinningOutcome)

let hasSingleWinningOutcome = (market: t): bool => singleWinningOutcome(market)->Option.isSome

// Result of fetching multiple markets: valid markets plus any per-market
// validation errors (invalid markets are skipped, not fatal).
module MarketsResult = {
  type nonrec t = {
    markets: array<t>,
    validationErrors: array<string>,
  }
}

// Result of fetching the global deposit-asset whitelist: valid assets plus any
// per-asset validation errors.
module GlobalDepositAssetsResult = {
  type t = {
    assets: array<GlobalDepositAsset.t>,
    validationErrors: array<string>,
  }
}
