// Orderbook depth client — live REST depth, optionally aggregated. The
// query-string helper is internal.

// Live orderbook depth, optionally aggregated. `depth` is capped server-side at
// 20 levels per side. Invalid aggregation combinations are rejected client-side
// before any request is made.
let get: (
  Client.t,
  ~orderbookId: string,
  ~depth: int=?,
  ~aggregation: Orderbook__Model.Aggregation.t=?,
  ~cookieHeader: string=?,
) => promise<result<Orderbook__Raw.DepthResponse.t, SdkError.t>>

// Convenience: depth with an explicit aggregation view.
let getWithAggregation: (
  Client.t,
  ~orderbookId: string,
  ~aggregation: Orderbook__Model.Aggregation.t,
  ~depth: int=?,
  ~cookieHeader: string=?,
) => promise<result<Orderbook__Raw.DepthResponse.t, SdkError.t>>

// Convenience: depth with an explicit auth cookie (Node/Bun, no cookie jar).
let getWithCookies: (
  Client.t,
  ~orderbookId: string,
  ~cookieHeader: string,
  ~depth: int=?,
  ~aggregation: Orderbook__Model.Aggregation.t=?,
) => promise<result<Orderbook__Raw.DepthResponse.t, SdkError.t>>
