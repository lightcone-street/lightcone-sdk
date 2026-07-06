// A rolling, capped trade-history buffer per orderbook, maintained from WS
// `Trades` frames and REST backfills. Newest-first: WS trades carry a monotonic
// `sequence` and are inserted in descending sequence order (duplicated
// sequences insert before the equal entry); REST trades have `sequence == 0`
// and are simply prepended. A `maxSize` of 0 disables the buffer entirely.

type t = {
  orderbookId: Shared.orderBookId,
  mutable trades: array<Trade__Model.t>,
  maxSize: int,
}

let make = (~orderbookId: Shared.orderBookId, ~maxSize: int): t => {
  orderbookId,
  trades: [],
  maxSize,
}

// Push a new trade, keeping the buffer sorted newest-first and capped. Trades
// older than everything in a full buffer are dropped.
let push = (state: t, trade: Trade__Model.t): unit =>
  if state.maxSize > 0 {
    if trade.sequence == 0.0 {
      // REST trades carry no ordering metadata: prepend, evict the oldest.
      let updated = [trade]->Array.concat(state.trades)
      state.trades =
        Array.length(updated) > state.maxSize
          ? updated->Array.slice(~start=0, ~end=state.maxSize)
          : updated
    } else {
      // First retained trade older than the incoming sequence — inserting there
      // keeps the buffer sorted newest-first.
      let position =
        state.trades
        ->Array.findIndex(existing => existing.sequence < trade.sequence)
        ->(index => index == -1 ? Array.length(state.trades) : index)
      if !(Array.length(state.trades) >= state.maxSize && position == Array.length(state.trades)) {
        let updated = Array.concatMany(
          state.trades->Array.slice(~start=0, ~end=position),
          [[trade], state.trades->Array.slice(~start=position, ~end=Array.length(state.trades))],
        )
        state.trades =
          Array.length(updated) > state.maxSize
            ? updated->Array.slice(~start=0, ~end=state.maxSize)
            : updated
      }
    }
  }

// Replace all trades (e.g. from a REST fetch), truncated to capacity.
let replace = (state: t, trades: array<Trade__Model.t>): unit =>
  state.trades = trades->Array.slice(~start=0, ~end=state.maxSize)

// Newest-first view of the buffer.
let trades = (state: t): array<Trade__Model.t> => state.trades

// The most recent trade, if any.
let latest = (state: t): option<Trade__Model.t> => state.trades[0]

let clear = (state: t): unit => state.trades = []

let size = (state: t): int => Array.length(state.trades)

let isEmpty = (state: t): bool => Array.length(state.trades) == 0
