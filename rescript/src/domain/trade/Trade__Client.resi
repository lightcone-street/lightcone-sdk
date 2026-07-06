// Trade REST client — trade-history queries. The query-string helper is
// internal.

// Trades for one orderbook. `cursor` is a numeric REST row id (pass a prior
// `nextCursor` to page).
let get: (
  Client.t,
  ~orderbookId: string,
  ~limit: int=?,
  ~cursor: float=?,
) => promise<result<Trade__Model.Page.t, SdkError.t>>

// Trades across every orderbook in a market, interleaved by time.
let getByMarket: (
  Client.t,
  ~marketPubkey: string,
  ~limit: int=?,
  ~cursor: float=?,
) => promise<result<Trade__Model.Page.t, SdkError.t>>
