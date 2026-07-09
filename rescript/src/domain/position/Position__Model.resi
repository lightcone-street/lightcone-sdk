// Position domain types — the portfolio / position / wallet-holding family that
// forms the public type surface, deposit-asset metadata, the user's per-token
// balances with their computed display values, and the conditional balance
// deltas arriving on the WS user channel. `@spice` is kept on
// `DepositAssetMetadata` so its codecs stay public; `@genType` exports the TS
// shapes. The decimal-string helpers are internal.

// ── Domain types ──────────────────────────────────────────────────────────────
// (The REST read endpoints return the `Position__Raw` wire shapes directly; the
// types below are the public type surface only.)

// One outcome within a position.
module Outcome: {
  @genType
  type t = {
    conditionId: float,
    conditionName: string,
    tokenMint: Shared.pubkeyStr,
    amount: string,
    usdValue: string,
  }
}

// A non-conditional token balance held in the user's wallet.
module WalletHolding: {
  @genType
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
@genType
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
module Portfolio: {
  @genType
  type nonrec t = {
    userAddress: Shared.pubkeyStr,
    walletHoldings: array<WalletHolding.t>,
    positions: array<t>,
    totalWalletValue: string,
    totalPositionsValue: string,
  }
}

// Static metadata for a deposit asset.
module DepositAssetMetadata: {
  @genType @spice
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

// ── Token balances ────────────────────────────────────────────────────────────
// A user's balance for a specific token, with its computed display values
// (everything formats through `Fmt.display` — tiered decimals, comma-grouped
// thousands).
module TokenBalance: {
  // Classification of a token balance's source.
  @genType
  type kind =
    | DepositAsset
    | ConditionalToken({
        orderbookId: Shared.orderBookId,
        marketPubkey: Shared.pubkeyStr,
        outcomeIndex: float,
      })

  // A user's balance for a specific token.
  @genType
  type t = {
    mint: Shared.pubkeyStr,
    idle: string,
    onBook: string,
    tokenType: kind,
  }

  // Display strings for a conditional (base) balance at a given price.
  @genType
  type computedBase = {
    value: string,
    size: string,
    price: string,
  }

  @genType
  let computedBase: (t, ~conditionalPrice: string) => computedBase

  // Display string for a quote balance: idle + on-book.
  @genType
  let computedQuote: t => string
}

// ── Conditional balance delta ─────────────────────────────────────────────────
// One conditional-token balance from a WS user event, before it is folded into a
// balance index or token balance. Decimal fields are wire strings (malformed → 0).
module ConditionalBalanceDelta: {
  @genType
  type t = {
    marketPubkey: Shared.pubkeyStr,
    orderbookId?: Shared.orderBookId,
    outcomeIndex: float,
    conditionalToken: Shared.pubkeyStr,
    idle: string,
    onBook: string,
  }

  // idle + on-book, as a Decimal string.
  @genType
  let total: t => string
  // Neither idle nor on-book balance is positive.
  @genType
  let isZero: t => bool

  // Delta → token balance (classified as a conditional token; a missing
  // orderbook id becomes the empty default).
  @genType
  let toTokenBalance: t => TokenBalance.t

  // Delta → WS outcome balance (`balance` = idle + on-book).
  @genType
  let toUserOutcomeBalance: t => Order__Raw.UserOutcomeBalance.t
}
