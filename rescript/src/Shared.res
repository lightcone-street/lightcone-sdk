// Shared newtypes and enums used across every domain module — the ReScript
// counterpart of the Rust SDK's `shared` module.
//
// Enum codec convention (see the project memo): each string enum sets BOTH `@as`
// (ReScript runtime value + the gentype TS union) and `@spice.as` (the JSON wire
// value) to the exact wire string. That keeps three representations aligned —
// ReScript constructor, JSON wire, and the TypeScript union the SDK exports.
//
// Each enum lives in its own sub-module (`type t`) so that look-alike constructors
// across the trigger enums (Created/Triggered/…) don't collide.

// ── Transparent string newtypes ──────────────────────────────────────────────
// The Rust newtypes serialize as plain JSON strings; we mirror that with string
// aliases (spice → string codec, gentype → `string`).
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
}

// ── Denominator ──────────────────────────────────────────────────────────────
module Denominator = {
  @spice
  type t = | @as("Base") @spice.as("Base") Base | @as("Quote") @spice.as("Quote") Quote
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
