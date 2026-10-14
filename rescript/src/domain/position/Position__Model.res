// Position domain types — the portfolio / position / wallet-holding family that
// forms the public type surface, deposit-asset metadata, the user's per-token
// balances with their computed display values, and the conditional balance
// deltas arriving on the WS user channel.
//
// Balances/amounts stay as wire strings (no precision loss, gentype-clean).
// Ids / counts / token decimals are floats (JS numbers). Note: the REST read
// endpoints return the `Position__Raw` wire shapes directly; the domain types
// here are NOT produced by those endpoints — they are the public type surface
// only.

// ── Domain types ──────────────────────────────────────────────────────────────

// One outcome within a position.
module Outcome = {
  type t = {
    conditionId: float,
    conditionName: string,
    tokenMint: Shared.pubkeyStr,
    amount: string,
    usdValue: string,
  }
}

// A non-conditional token balance held in the user's wallet.
module WalletHolding = {
  type t = {
    tokenMint: Shared.pubkeyStr,
    symbol: string,
    amount: string,
    decimals: float,
    usdValue: string,
    imgSrc: string,
  }
}

// A user's position in a single market.
type t = {
  eventPubkey: Shared.pubkeyStr,
  eventName: string,
  eventImgSrc: string,
  outcomes: array<Outcome.t>,
  totalValue: string,
  // Unix milliseconds.
  createdAt: float,
}

// A user's full portfolio across all markets.
module Portfolio = {
  type nonrec t = {
    userAddress: Shared.pubkeyStr,
    walletHoldings: array<WalletHolding.t>,
    positions: array<t>,
    totalWalletValue: string,
    totalPositionsValue: string,
  }
}

// Static metadata for a deposit asset.
module DepositAssetMetadata = {
  @spice
  type t = {
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
}

// ── Decimal-string helpers (tolerant: malformed → zero) ──────────────────────
let decimalOrZero = (value: string): Decimal.t =>
  switch Decimal.fromString(value) {
  | decimal => decimal
  | exception JsExn(_) => Decimal.fromInt(0)
  }

let decimalIsPositive = (value: string): bool => Decimal.gt(decimalOrZero(value), Decimal.fromInt(0))

// ── Token balances ────────────────────────────────────────────────────────────
// A user's balance for a specific token, with its computed display values
// (everything formats through `Fmt.display`).
module TokenBalance = {
  // Classification of a token balance's source.
  type kind =
    | DepositAsset
    | ConditionalToken({
        orderbookId: Shared.orderBookId,
        marketPubkey: Shared.pubkeyStr,
        outcomeIndex: float,
      })

  // A user's balance for a specific token.
  type t = {
    mint: Shared.pubkeyStr,
    idle: string,
    onBook: string,
    tokenType: kind,
  }

  // Display strings for a conditional (base) balance at a given price.
  type computedBase = {
    value: string,
    size: string,
    price: string,
  }

  let computedBase = (balance: t, ~conditionalPrice: string): computedBase => {
    let price = decimalOrZero(conditionalPrice)
    let size = Decimal.plus(decimalOrZero(balance.idle), decimalOrZero(balance.onBook))
    {
      value: Fmt.Decimal.display(Decimal.times(size, price)),
      size: Fmt.Decimal.display(size),
      price: Fmt.Decimal.display(price),
    }
  }

  // Display string for a quote balance: idle + on-book.
  let computedQuote = (balance: t): string =>
    Fmt.Decimal.display(Decimal.plus(decimalOrZero(balance.idle), decimalOrZero(balance.onBook)))
}

// ── Conditional balance delta ─────────────────────────────────────────────────
// One conditional-token balance from a WS user event, before it is folded into
// a balance index or token balance.
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

  // Delta → token balance (classified as a conditional token; a missing
  // orderbook id becomes the empty default).
  let toTokenBalance = (delta: t): TokenBalance.t => {
    mint: delta.conditionalToken,
    idle: delta.idle,
    onBook: delta.onBook,
    tokenType: TokenBalance.ConditionalToken({
      orderbookId: delta.orderbookId->Option.getOr(""),
      marketPubkey: delta.marketPubkey,
      outcomeIndex: delta.outcomeIndex,
    }),
  }

  // Delta → WS outcome balance (`balance` = idle + on-book).
  let toUserOutcomeBalance = (delta: t): Order.Raw.UserOutcomeBalance.t => {
    outcomeIndex: delta.outcomeIndex,
    conditionalToken: delta.conditionalToken,
    balance: total(delta),
    balanceIdle: delta.idle,
    balanceOnBook: delta.onBook,
  }
}
