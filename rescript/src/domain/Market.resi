// Markets domain — market discovery / metadata queries. Public surface: the
// market enums (`Status`, `ResolutionKind`), the gentype-exported resolution /
// deposit-asset / orderbook-pair / market types, the resolution helpers, and the
// client query functions. The raw wire `*Response` types, the `*OfResponse`
// conversions, and the spice codecs are internal.

// ── Domain-local enums ────────────────────────────────────────────────────────
module Status: {
  @genType
  type t =
    | @as("Pending") Pending
    | @as("Active") Active
    | @as("Resolved") Resolved
    | @as("Cancelled") Cancelled

  let toString: t => string
  let fromString: string => option<t>
}

module ResolutionKind: {
  @genType
  type t =
    | @as("single_winner") SingleWinner
    | @as("scalar") Scalar
}

// ── Resolution types ──────────────────────────────────────────────────────────
@genType
type marketResolutionPayout = {
  outcomeIndex: float,
  payoutNumerator: float,
}

@genType
type marketResolutionResponse = {
  kind: ResolutionKind.t,
  payoutDenominator: float,
  payouts: array<marketResolutionPayout>,
  singleWinningOutcome?: float,
}

// ── Deposit-asset / search response types (returned directly) ──────────────────
@genType
type conditionalTokenResponse = {
  id: float,
  outcomeIndex: float,
  tokenAddress: string,
  symbol?: string,
  uri?: string,
  outcome?: string,
  depositSymbol?: string,
  shortSymbol?: string,
  description?: string,
  iconUrlLow?: string,
  iconUrlMedium?: string,
  iconUrlHigh?: string,
  metadataUri?: string,
  decimals?: float,
  createdAt: string,
}

@genType
type depositAssetResponse = {
  displayName?: string,
  tokenSymbol?: string,
  symbol?: string,
  depositAsset: Shared.pubkeyStr,
  id: float,
  marketPubkey: Shared.pubkeyStr,
  vault: string,
  numOutcomes: float,
  description?: string,
  iconUrlLow?: string,
  iconUrlMedium?: string,
  iconUrlHigh?: string,
  metadataUri?: string,
  decimals?: float,
  minOrderSize?: string,
  conditionalMints: array<conditionalTokenResponse>,
  createdAt: string,
}

@genType
type depositMintsResponse = {
  marketPubkey: Shared.pubkeyStr,
  depositAssets: array<depositAssetResponse>,
  total: float,
}

@genType
type searchOrderbook = {
  orderbookId: Shared.orderBookId,
  outcomeName: string,
  outcomeNameLong?: string,
  outcomeIndex: float,
  depositBaseAsset: Shared.pubkeyStr,
  depositQuoteAsset: Shared.pubkeyStr,
  depositBaseSymbol: string,
  depositQuoteSymbol: string,
  baseIconUrlLow?: string,
  baseIconUrlMedium?: string,
  baseIconUrlHigh?: string,
  quoteIconUrlLow?: string,
  quoteIconUrlMedium?: string,
  quoteIconUrlHigh?: string,
  conditionalBaseMint: Shared.pubkeyStr,
  conditionalQuoteMint: Shared.pubkeyStr,
  outcomeIconUrlLow?: string,
  outcomeIconUrlMedium?: string,
  outcomeIconUrlHigh?: string,
  conditionalBaseSymbol?: string,
  conditionalQuoteSymbol?: string,
  latestMidPrice?: string,
}

@genType
type marketSearchResult = {
  slug: string,
  marketName: string,
  marketStatus: Status.t,
  category?: string,
  tags: array<string>,
  featuredRank: float,
  description?: string,
  iconUrlLow?: string,
  iconUrlMedium?: string,
  iconUrlHigh?: string,
  orderbooks: array<searchOrderbook>,
}

// ── Domain types ──────────────────────────────────────────────────────────────
@genType
type outcome = {
  index: float,
  iconUrlLow: string,
  iconUrlMedium: string,
  iconUrlHigh: string,
  name: string,
  nameLong?: string,
}

@genType
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

@genType
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

@genType
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

@genType
type depositAssetPair = {
  // Stable identifier of the form `"{base_pubkey}-{quote_pubkey}"`.
  id: string,
  base: depositAsset,
  quote: depositAsset,
}

@genType
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
@genType
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

@genType
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
@genType
type marketsResult = {
  markets: array<market>,
  validationErrors: array<string>,
}

// Result of fetching the global deposit-asset whitelist: valid assets plus any
// per-asset validation errors.
@genType
type globalDepositAssetsResult = {
  assets: array<globalDepositAsset>,
  validationErrors: array<string>,
}

// ── Market resolution helpers (mirror the Rust `Market` impl) ──────────────────
let isResolved: market => bool
let singleWinningOutcome: market => option<float>
let hasSingleWinningOutcome: market => bool

// ── Client functions ──────────────────────────────────────────────────────────
// Cursor-paginated markets. Only Active and Resolved markets are kept; markets
// that fail validation are skipped and surfaced in `validationErrors`.
let get: (
  Client.t,
  ~cursor: float=?,
  ~limit: int=?,
) => promise<result<marketsResult, SdkError.t>>

// Featured markets. Only Active markets are returned.
let featured: Client.t => promise<result<array<marketSearchResult>, SdkError.t>>

// Fetch a single market by slug.
let getBySlug: (Client.t, ~slug: string) => promise<result<market, SdkError.t>>

// Fetch a single market by on-chain pubkey.
let getByPubkey: (Client.t, ~pubkey: string) => promise<result<market, SdkError.t>>

// Search markets by query string.
let search: (
  Client.t,
  ~query: string,
  ~limit: int=?,
) => promise<result<array<marketSearchResult>, SdkError.t>>

// The active global deposit-asset whitelist (platform-scoped). Assets that fail
// validation are skipped and surfaced in `validationErrors`.
let globalDepositAssets: Client.t => promise<result<globalDepositAssetsResult, SdkError.t>>

// Deposit assets registered for a specific market (with their conditional mints).
let getDepositMints: (
  Client.t,
  ~marketPubkey: string,
) => promise<result<depositMintsResponse, SdkError.t>>
