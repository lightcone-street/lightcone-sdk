// Orderbook wire types — the WS book levels + snapshot frame and the REST depth
// response. `@spice` is kept so the codecs are public (the depth client and the
// WS message layer decode across the file boundary); `@genType` exports the TS
// shapes.

// ── Levels ────────────────────────────────────────────────────────────────────
// A single WS price level — `side` is explicit in WS frames.
module WsLevel: {
  @genType @spice
  type t = {
    side: Shared.Side.t,
    price: string,
    size: string,
  }
}

// A single REST depth level — side is implicit from the bids/asks array.
module RestLevel: {
  @genType @spice
  type t = {
    price: string,
    size: string,
    orders?: float,
  }
}

// ── REST depth response ───────────────────────────────────────────────────────
// Price/size display decimals returned by the depth endpoint.
module DepthDecimals: {
  @genType @spice
  type t = {
    price: float,
    size: float,
  }
}

// Live orderbook depth (returned directly; no domain conversion).
module DepthResponse: {
  @genType @spice
  type t = {
    @spice.key("orderbook_id") orderbookId: Shared.orderBookId,
    @spice.key("market_pubkey") marketPubkey?: string,
    @spice.key("best_bid") bestBid?: string,
    @spice.key("best_ask") bestAsk?: string,
    spread?: string,
    @spice.key("tick_size") tickSize?: string,
    bids: array<RestLevel.t>,
    asks: array<RestLevel.t>,
    decimals?: DepthDecimals.t,
  }
}

// ── WS snapshot frame ─────────────────────────────────────────────────────────
// Snapshot-only stream: every data frame carries the full top-20 levels per side
// and replaces the previous book wholesale. `seq` is strictly increasing but
// non-contiguous; the initial snapshot after each (re)subscribe is `seq: 0`.
module Book: {
  @genType @spice
  type t = {
    @spice.key("orderbook_id") id: Shared.orderBookId,
    @spice.default(false) @spice.key("is_snapshot") isSnapshot: bool,
    @spice.default(0.0) seq: float,
    @spice.default(false) resync: bool,
    @spice.default([]) bids: array<WsLevel.t>,
    @spice.default([]) asks: array<WsLevel.t>,
    @spice.key("n_sig_figs") nSigFigs?: int,
    mantissa?: int,
  }

  // The aggregation view a frame belongs to (untagged = full precision).
  let toAggregation: t => Orderbook__Model.Aggregation.t
}
