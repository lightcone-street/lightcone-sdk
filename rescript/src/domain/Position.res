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

// ── Conversions ───────────────────────────────────────────────────────────────
// Mirrors Rust `impl From<DepositTokenBalance> for TokenBalance` (on_book = 0,
// classified as a deposit asset).
let tokenBalanceOfDepositTokenBalance = (value: depositTokenBalance): tokenBalance => {
  mint: value.mint,
  idle: value.idle,
  onBook: "0",
  tokenType: DepositAsset,
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

// Note: the core on-chain builders ARE ported in `program/PositionBuilders.res`
// (depositToGlobal, withdrawFromGlobal, globalToMarketDeposit, merge,
// redeemWinnings — each builds + signs + sends a Solana transaction) and
// `program/Instructions.res` (initPositionTokens, incrementNonce instructions);
// the on-chain account reads (`get_onchain`) live in `Rpc.res`
// (getExchange / getMarket / getOrderbook / getPosition + the PDA helpers).
//
// TODO(program-layer): the less-common flows remain deferred — the market-level
// deposit/withdraw, extendPositionTokens, closePositionAlt /
// closePositionTokenAccounts / withdrawFromPosition, and the low-level `_ix` /
// `_tx` variants.
//
// Also deferred from `domain::position` (WS / balance-index + Decimal-math
// layer, handled separately): the `From<ConditionalBalanceDelta>` conversions,
// `ConditionalBalanceDelta`, `UserMarketBalanceIndex`, `DepositAssetMetadata`,
// and `TokenBalance::computed_base` / `computed_quote`.
