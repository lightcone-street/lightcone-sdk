// Shared newtypes + enums used across every domain. Everything here is public
// (the enum codecs are consumed cross-module), so this interface restates the full
// surface; `@spice` is kept so the ppx regenerates the JSON codecs.

@genType @spice
type orderBookId = string

@genType @spice
type pubkeyStr = string

module Side: {
  @genType @spice
  type t = | @as("bid") @spice.as("bid") Bid | @as("ask") @spice.as("ask") Ask

  let toString: t => string
  // UI label: Bid → "Buy", Ask → "Sell".
  let label: t => string

  // The price to submit with a market (IOC) order: the worst book fill price
  // padded by the impact-protection percentage in the fill direction. `None`
  // unless both inputs are positive Decimal strings.
  let applyImpactProtection: (
    t,
    ~worstFillPrice: string,
    ~protectionPercent: string,
  ) => option<string>
}

module Denominator: {
  @genType @spice
  type t = | @as("Base") @spice.as("Base") Base | @as("Quote") @spice.as("Quote") Quote

  // Display order: quote first.
  let all: array<t>

  // Convert `amount` between denominations at `basePriceInQuote` (Decimal
  // strings). Crossing denominations requires a positive price — `None`
  // otherwise (or on malformed input).
  let convertTo: (t, ~target: t, ~amount: string, ~basePriceInQuote: string) => option<string>
}

let spendDenominator: Side.t => Denominator.t
let receiveDenominator: Side.t => Denominator.t

module TimeInForce: {
  @genType @spice
  type t =
    | @as("GTC") @spice.as("GTC") Gtc
    | @as("IOC") @spice.as("IOC") Ioc
    | @as("FOK") @spice.as("FOK") Fok
    | @as("ALO") @spice.as("ALO") Alo
}

module TriggerType: {
  @genType @spice
  type t =
    | @as("TP") @spice.as("TP") TakeProfit
    | @as("SL") @spice.as("SL") StopLoss
}

// The engine's real-time order state; wire absence defaults to Open.
module OrderStatus: {
  @genType @spice
  type t =
    | @as("OPEN") @spice.as("OPEN") Open
    | @as("MATCHING") @spice.as("MATCHING") Matching
    | @as("CANCELLED") @spice.as("CANCELLED") Cancelled
    | @as("FILLED") @spice.as("FILLED") Filled
    | @as("PENDING") @spice.as("PENDING") Pending
}

module TriggerStatus: {
  @genType @spice
  type t =
    | @as("created") @spice.as("created") Created
    | @as("triggered") @spice.as("triggered") Triggered
    | @as("failed") @spice.as("failed") Failed
    | @as("expired") @spice.as("expired") Expired
    | @as("invalidated") @spice.as("invalidated") Invalidated
}

module OrderUpdateType: {
  @genType @spice
  type t =
    | @as("PLACEMENT") @spice.as("PLACEMENT") Placement
    | @as("UPDATE") @spice.as("UPDATE") Update
    | @as("CANCELLATION") @spice.as("CANCELLATION") Cancellation
}

module TriggerUpdateType: {
  @genType @spice
  type t =
    | @as("CREATED") @spice.as("CREATED") Created
    | @as("TRIGGERED") @spice.as("TRIGGERED") Triggered
    | @as("FAILED") @spice.as("FAILED") Failed
    | @as("EXPIRED") @spice.as("EXPIRED") Expired
    | @as("INVALIDATED") @spice.as("INVALIDATED") Invalidated
}

module TriggerResultStatus: {
  @genType @spice
  type t =
    | @as("filled") @spice.as("filled") Filled
    | @as("accepted") @spice.as("accepted") Accepted
    | @as("rejected") @spice.as("rejected") Rejected
}

module DepositSource: {
  @genType @spice
  type t =
    | @as("global") @spice.as("global") Global
    | @as("market") @spice.as("market") Market
}

module Resolution: {
  @genType @spice
  type t =
    | @as("1m") @spice.as("1m") Minute1
    | @as("5m") @spice.as("5m") Minute5
    | @as("15m") @spice.as("15m") Minute15
    | @as("1h") @spice.as("1h") Hour1
    | @as("4h") @spice.as("4h") Hour4
    | @as("1d") @spice.as("1d") Day1

  let toString: t => string
  // Candle duration in seconds.
  let seconds: t => int
  let fromString: string => option<t>
}

// Derive an orderbook id from base/quote token pubkeys: `{base[0:8]}_{quote[0:8]}`.
let deriveOrderbookId: (~baseToken: string, ~quoteToken: string) => orderBookId
