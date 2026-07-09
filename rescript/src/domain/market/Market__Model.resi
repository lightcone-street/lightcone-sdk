// Market domain types — lifecycle status, resolution payouts, validated token
// structures, tradeable orderbook pairs, price-impact math, and the primary
// market type. `@spice` is kept on the enums and resolution types so the wire
// decoding in `Market__Raw` can reference the codecs across the file boundary;
// `@genType` exports the TS shapes.

// ── Domain-local enums ────────────────────────────────────────────────────────
// Market lifecycle status. Wire value = the PascalCase variant name.
module Status: {
  @genType @spice
  type t =
    | @as("Pending") @spice.as("Pending") Pending
    | @as("Active") @spice.as("Active") Active
    | @as("Resolved") @spice.as("Resolved") Resolved
    | @as("Cancelled") @spice.as("Cancelled") Cancelled

  let toString: t => string
  let fromString: string => option<t>
}

// A market's resolution outcome. Wire-shaped (`@spice`, snake_case keys): the
// backend payload is embedded in the market domain type unchanged.
module Resolution: {
  // Resolution kind. Wire value = the snake_case variant name.
  module Kind: {
    @genType @spice
    type t =
      | @as("single_winner") @spice.as("single_winner") SingleWinner
      | @as("scalar") @spice.as("scalar") Scalar
  }

  @genType @spice
  type payout = {
    @spice.key("outcome_index") outcomeIndex: float,
    @spice.key("payout_numerator") payoutNumerator: float,
  }

  @genType @spice
  type t = {
    kind: Kind.t,
    @spice.key("payout_denominator") payoutDenominator: float,
    payouts: array<payout>,
    @spice.key("single_winning_outcome") singleWinningOutcome?: float,
  }
}

// ── Token helpers ─────────────────────────────────────────────────────────────
@genType
let usdcMainnet: Shared.pubkeyStr
@genType
let usdtMainnet: Shared.pubkeyStr
@genType
let usdcDevnetLc: Shared.pubkeyStr

@genType
let isUsdStablecoin: Shared.pubkeyStr => bool
// "$" for USD stablecoins, "" otherwise.
@genType
let currencySymbol: Shared.pubkeyStr => string

// A new array ordered for display: BTC/WBTC → ETH/WETH → SOL first, then the
// rest alphabetically by symbol. Pass `~symbolOf` (e.g. `token => token.symbol`,
// or `pair => pair.base.symbol` for composite pairs).
@genType
let sortByDisplayPriority: (array<'a>, ~symbolOf: 'a => string) => array<'a>

// Resolve three icon-url quality variants with cross-fallback. `None` only when
// all three inputs are absent.
let resolveIconUrls: (
  option<string>,
  option<string>,
  option<string>,
) => option<(string, string, string)>

// ── Token types ───────────────────────────────────────────────────────────────
module Outcome: {
  @genType
  type t = {
    index: float,
    iconUrlLow: string,
    iconUrlMedium: string,
    iconUrlHigh: string,
    name: string,
    nameLong?: string,
  }
}

module ConditionalToken: {
  @genType
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
  @genType
  let isUsdStableCoin: t => bool
  @genType
  let currencySymbol: t => string
}

module DepositAsset: {
  @genType
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
  @genType
  let isUsdStableCoin: t => bool
  @genType
  let currencySymbol: t => string
}

module TokenMetadata: {
  @genType
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

module DepositAssetPair: {
  @genType
  type t = {
    // Stable identifier of the form `"{base_pubkey}-{quote_pubkey}"`.
    id: string,
    base: DepositAsset.t,
    quote: DepositAsset.t,
  }
}

module GlobalDepositAsset: {
  @genType
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
  @genType
  let isUsdStableCoin: t => bool
  @genType
  let currencySymbol: t => string
}

// A tradeable conditional-token pair within a market. Owned by the market
// domain: its wire conversion needs the market's conditional-token list.
module OrderBookPair: {
  @genType
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

  // Scaling decimals derived from the pair's token metadata (no REST call
  // needed).
  @genType
  let decimals: t => Scaling.OrderbookDecimals.t

  // The conditional token a denomination refers to on this pair, plus its
  // symbols.
  @genType
  let denominatorToken: (Shared.Denominator.t, t) => ConditionalToken.t
  @genType
  let denominatorSymbol: (Shared.Denominator.t, t) => string
  @genType
  let denominatorDepositSymbol: (Shared.Denominator.t, t) => string
}

// ── Price impact ──────────────────────────────────────────────────────────────
// Calculated impact of a conditional token's price vs its deposit asset.
module Impact: {
  @genType
  type t = {
    sign: string,
    pct: float,
    // Absolute dollar difference, as a Decimal string.
    dollar: string,
    isPositive: bool,
  }

  let zero: t

  // Price impact as a percentage relative to the deposit asset price:
  // (pct, sign). Zero / malformed inputs yield (0.0, "").
  @genType
  let pct: (~depositPrice: string, ~conditionalPrice: string) => (float, string)

  // Full impact with sign, percentage, and absolute dollar difference. A zero /
  // malformed deposit price yields the zero impact.
  @genType
  let make: (~depositAssetPrice: string, ~conditionalPrice: string) => t
}

// ── Market ────────────────────────────────────────────────────────────────────
// The primary market domain type.
@genType
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
let isResolved: t => bool
let singleWinningOutcome: t => option<float>
let hasSingleWinningOutcome: t => bool

// Result of fetching multiple markets: valid markets plus any per-market
// validation errors (invalid markets are skipped, not fatal).
module MarketsResult: {
  @genType
  type nonrec t = {
    markets: array<t>,
    validationErrors: array<string>,
  }
}

// Result of fetching the global deposit-asset whitelist: valid assets plus any
// per-asset validation errors.
module GlobalDepositAssetsResult: {
  @genType
  type t = {
    assets: array<GlobalDepositAsset.t>,
    validationErrors: array<string>,
  }
}
