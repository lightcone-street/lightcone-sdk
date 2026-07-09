// Shared newtypes and enums used across every domain module.
//
// Enum codec convention (see the project memo): each string enum sets BOTH `@as`
// (ReScript runtime value + the gentype TS union) and `@spice.as` (the JSON wire
// value) to the exact wire string. That keeps three representations aligned —
// ReScript constructor, JSON wire, and the TypeScript union the SDK exports.
//
// Each enum lives in its own sub-module (`type t`) so that look-alike constructors
// across the trigger enums (Created/Triggered/…) don't collide.

// ── Transparent string newtypes ──────────────────────────────────────────────
// The wire carries these as plain JSON strings, so they stay string aliases
// (spice → string codec, gentype → `string`).
//
// `pubkeyStr` deliberately stays a plain string rather than kit's branded
// `SolanaKit.address` (decision 2026-07: the wire treats pubkeys as unvalidated
// strings; validation + branding happen at the program boundary, via
// `SolanaKit.address(...)`).
// Using the branded type wire-wide would cost per-field base58 validation on hot
// WS paths, break `Dict` keys / URL interpolation across the state containers,
// and turn every TS-facing pubkey field opaque.
@spice
type orderBookId = string

@spice
type pubkeyStr = string

// ── Side ─────────────────────────────────────────────────────────────────────
module Side = {
  @spice
  type t = | @as("bid") @spice.as("bid") Bid | @as("ask") @spice.as("ask") Ask

  let toString = (side: t) =>
    switch side {
    | Bid => "bid"
    | Ask => "ask"
    }

  // UI label: Bid → "Buy", Ask → "Sell".
  let label = (side: t) =>
    switch side {
    | Bid => "Buy"
    | Ask => "Sell"
    }

  // The price to submit with a market (IOC) order: the worst book fill price
  // padded by the impact-protection percentage in the direction that lets the
  // order fill (bids pay more, asks receive less). `None` unless both inputs
  // are positive Decimal strings.
  let applyImpactProtection = (side: t, ~worstFillPrice: string, ~protectionPercent: string): option<string> =>
    switch (Decimal.fromString(worstFillPrice), Decimal.fromString(protectionPercent)) {
    | (price, percent) =>
      let zero = Decimal.fromInt(0)
      if Decimal.lte(price, zero) || Decimal.lte(percent, zero) {
        None
      } else {
        let factor = Decimal.div(percent, Decimal.fromInt(100))
        let one = Decimal.fromInt(1)
        let padded = switch side {
        | Bid => Decimal.times(price, Decimal.plus(one, factor))
        | Ask => Decimal.times(price, Decimal.minus(one, factor))
        }
        Some(Decimal.toString(padded))
      }
    | exception JsExn(_) => None
    }
}

// ── Denominator ──────────────────────────────────────────────────────────────
module Denominator = {
  @spice
  type t = | @as("Base") @spice.as("Base") Base | @as("Quote") @spice.as("Quote") Quote

  // Display order: quote first.
  let all: array<t> = [Quote, Base]

  // Convert `amount` from this denomination into `target` at the given price
  // (quote per one base, both Decimal strings). Same-denomination conversion is
  // the identity and never needs a price; crossing denominations requires a
  // positive price — `None` otherwise (or on malformed input).
  let convertTo = (
    denominator: t,
    ~target: t,
    ~amount: string,
    ~basePriceInQuote: string,
  ): option<string> =>
    switch (Decimal.fromString(amount), Decimal.fromString(basePriceInQuote)) {
    | (amount, price) =>
      let priceIsUsable = Decimal.gt(price, Decimal.fromInt(0))
      switch (denominator, target) {
      | (Base, Base) | (Quote, Quote) => Some(Decimal.toString(amount))
      | (Base, Quote) => priceIsUsable ? Some(Decimal.times(amount, price)->Decimal.toString) : None
      | (Quote, Base) => priceIsUsable ? Some(Decimal.div(amount, price)->Decimal.toString) : None
      }
    | exception JsExn(_) => None
    }
}

// Bid spends quote / receives base; Ask spends base / receives quote.
let spendDenominator = (side: Side.t): Denominator.t =>
  switch side {
  | Bid => Quote
  | Ask => Base
  }

let receiveDenominator = (side: Side.t): Denominator.t =>
  switch side {
  | Bid => Base
  | Ask => Quote
  }

// ── TimeInForce ──────────────────────────────────────────────────────────────
module TimeInForce = {
  @spice
  type t =
    | @as("GTC") @spice.as("GTC") Gtc
    | @as("IOC") @spice.as("IOC") Ioc
    | @as("FOK") @spice.as("FOK") Fok
    | @as("ALO") @spice.as("ALO") Alo
}

// ── TriggerType ──────────────────────────────────────────────────────────────
module TriggerType = {
  @spice
  type t =
    | @as("TP") @spice.as("TP") TakeProfit
    | @as("SL") @spice.as("SL") StopLoss
}

// ── OrderStatus (UPPERCASE) ──────────────────────────────────────────────────
// The engine's real-time order state, following the shared enum-codec
// convention; wire absence defaults to Open (the wire default).
module OrderStatus = {
  @spice
  type t =
    | @as("OPEN") @spice.as("OPEN") Open
    | @as("MATCHING") @spice.as("MATCHING") Matching
    | @as("CANCELLED") @spice.as("CANCELLED") Cancelled
    | @as("FILLED") @spice.as("FILLED") Filled
    | @as("PENDING") @spice.as("PENDING") Pending
}

// ── TriggerStatus (WS, lowercase) ────────────────────────────────────────────
module TriggerStatus = {
  @spice
  type t =
    | @as("created") @spice.as("created") Created
    | @as("triggered") @spice.as("triggered") Triggered
    | @as("failed") @spice.as("failed") Failed
    | @as("expired") @spice.as("expired") Expired
    | @as("invalidated") @spice.as("invalidated") Invalidated
}

// ── OrderUpdateType (WS, UPPERCASE) ──────────────────────────────────────────
module OrderUpdateType = {
  @spice
  type t =
    | @as("PLACEMENT") @spice.as("PLACEMENT") Placement
    | @as("UPDATE") @spice.as("UPDATE") Update
    | @as("CANCELLATION") @spice.as("CANCELLATION") Cancellation
}

// ── TriggerUpdateType (WS, UPPERCASE) ────────────────────────────────────────
module TriggerUpdateType = {
  @spice
  type t =
    | @as("CREATED") @spice.as("CREATED") Created
    | @as("TRIGGERED") @spice.as("TRIGGERED") Triggered
    | @as("FAILED") @spice.as("FAILED") Failed
    | @as("EXPIRED") @spice.as("EXPIRED") Expired
    | @as("INVALIDATED") @spice.as("INVALIDATED") Invalidated
}

// ── TriggerResultStatus ──────────────────────────────────────────────────────
module TriggerResultStatus = {
  @spice
  type t =
    | @as("filled") @spice.as("filled") Filled
    | @as("accepted") @spice.as("accepted") Accepted
    | @as("rejected") @spice.as("rejected") Rejected
}

// ── DepositSource ────────────────────────────────────────────────────────────
module DepositSource = {
  @spice
  type t =
    | @as("global") @spice.as("global") Global
    | @as("market") @spice.as("market") Market
}

// ── Resolution ───────────────────────────────────────────────────────────────
module Resolution = {
  @spice
  type t =
    | @as("1m") @spice.as("1m") Minute1
    | @as("5m") @spice.as("5m") Minute5
    | @as("15m") @spice.as("15m") Minute15
    | @as("1h") @spice.as("1h") Hour1
    | @as("4h") @spice.as("4h") Hour4
    | @as("1d") @spice.as("1d") Day1

  let toString = (resolution: t) =>
    switch resolution {
    | Minute1 => "1m"
    | Minute5 => "5m"
    | Minute15 => "15m"
    | Hour1 => "1h"
    | Hour4 => "4h"
    | Day1 => "1d"
    }

  // Candle duration in seconds.
  let seconds = (resolution: t) =>
    switch resolution {
    | Minute1 => 60
    | Minute5 => 300
    | Minute15 => 900
    | Hour1 => 3600
    | Hour4 => 14400
    | Day1 => 86400
    }

  let fromString = (s: string): option<t> =>
    switch s {
    | "1m" => Some(Minute1)
    | "5m" => Some(Minute5)
    | "15m" => Some(Minute15)
    | "1h" => Some(Hour1)
    | "4h" => Some(Hour4)
    | "1d" => Some(Day1)
    | _ => None
    }
}

// ── Utilities ────────────────────────────────────────────────────────────────
// Derive an orderbook id from base/quote token pubkeys: `{base[0:8]}_{quote[0:8]}`.
let deriveOrderbookId = (~baseToken: string, ~quoteToken: string): orderBookId => {
  let prefix = s => String.slice(s, ~start=0, ~end=min(8, String.length(s)))
  `${prefix(baseToken)}_${prefix(quoteToken)}`
}
