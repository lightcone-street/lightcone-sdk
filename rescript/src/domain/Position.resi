// Positions domain — portfolio, token-balance, and market-position queries.
// Public surface: the gentype-exported wire `*Response` rows returned directly by
// the read endpoints, the domain re-exports of Rust's `domain::position`
// (`portfolio` / `position` / `walletHolding` / `tokenBalance` family), and the
// client functions. The `*OfResponse` conversion and the spice codecs are internal.

// ── Response rows (returned directly by the read endpoints) ────────────────────
@genType
type outcomeBalance = {
  outcomeIndex: float,
  conditionalToken: Shared.pubkeyStr,
  balance: string,
  balanceIdle: string,
  balanceOnBook: string,
}

@genType
type vaultBalance = {
  depositMint: Shared.pubkeyStr,
  vault: Shared.pubkeyStr,
  balance: string,
}

@genType
type globalDeposit = {
  depositMint: Shared.pubkeyStr,
  symbol: string,
  balance: string,
}

@genType
type positionEntry = {
  id: float,
  positionPubkey: Shared.pubkeyStr,
  owner: Shared.pubkeyStr,
  marketPubkey: Shared.pubkeyStr,
  outcomes: array<outcomeBalance>,
  vaultBalances: array<vaultBalance>,
  // ISO-8601 timestamps (the wire ships these as strings).
  createdAt: string,
  updatedAt: string,
}

// Response for `GET /api/users/{user_pubkey}/positions`.
@genType
type positionsResponse = {
  owner: Shared.pubkeyStr,
  totalMarkets: float,
  positions: array<positionEntry>,
  globalDeposits: array<globalDeposit>,
  // Mint pubkey → token decimals.
  decimals: dict<float>,
}

// Response for `GET /api/users/{user_pubkey}/markets/{market_pubkey}/positions`.
@genType
type marketPositionsResponse = {
  owner: Shared.pubkeyStr,
  marketPubkey: Shared.pubkeyStr,
  positions: array<positionEntry>,
  globalDeposits: array<globalDeposit>,
  decimals: dict<float>,
}

// Combined balance + metadata for a deposit token. Both the wire row AND the
// domain shape — `depositTokenBalances` returns these directly (keyed by mint).
@genType
type depositTokenBalance = {
  mint: Shared.pubkeyStr,
  idle: string,
  symbol: string,
  name: string,
  iconUrlLow?: string,
  iconUrlMedium?: string,
  iconUrlHigh?: string,
}

// ── Domain types ──────────────────────────────────────────────────────────────
// (Re-exported public surface of Rust's `domain::position`.)

// One outcome within a position.
@genType
type positionOutcome = {
  conditionId: float,
  conditionName: string,
  tokenMint: Shared.pubkeyStr,
  amount: string,
  usdValue: string,
}

// A non-conditional token balance held in the user's wallet.
@genType
type walletHolding = {
  tokenMint: Shared.pubkeyStr,
  symbol: string,
  amount: string,
  decimals: float,
  usdValue: string,
  imgSrc: string,
}

// A user's position in a single market.
@genType
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
@genType
type portfolio = {
  userAddress: Shared.pubkeyStr,
  walletHoldings: array<walletHolding>,
  positions: array<position>,
  totalWalletValue: string,
  totalPositionsValue: string,
}

// Classification of a token balance's source.
@genType
type tokenBalanceTokenType =
  | DepositAsset
  | ConditionalToken({
      orderbookId: Shared.orderBookId,
      marketPubkey: Shared.pubkeyStr,
      outcomeIndex: float,
    })

// A user's balance for a specific token.
@genType
type tokenBalance = {
  mint: Shared.pubkeyStr,
  idle: string,
  onBook: string,
  tokenType: tokenBalanceTokenType,
}

// ── Client functions ──────────────────────────────────────────────────────────
// All positions for a user across every market. Public path-based endpoint.
let get: (Client.t, ~userPubkey: string) => promise<result<positionsResponse, SdkError.t>>

// Positions for a user in a specific market. Public path-based endpoint.
let getForMarket: (
  Client.t,
  ~userPubkey: string,
  ~marketPubkey: string,
) => promise<result<marketPositionsResponse, SdkError.t>>

// All positions for the authenticated user (wallet resolved server-side from the
// auth cookie). Pass `~cookieHeader` to forward a per-request cookie (SSR).
let positions: (
  Client.t,
  ~cookieHeader: string=?,
) => promise<result<positionsResponse, SdkError.t>>

// The authenticated user's positions in a specific market.
let positionsForMarket: (
  Client.t,
  ~marketPubkey: string,
  ~cookieHeader: string=?,
) => promise<result<marketPositionsResponse, SdkError.t>>

// SPL deposit-token balances for the authenticated user, keyed by mint pubkey.
// An empty map means the user holds none of the tracked balances (not an error).
let depositTokenBalances: (
  Client.t,
  ~cookieHeader: string=?,
) => promise<result<dict<depositTokenBalance>, SdkError.t>>
