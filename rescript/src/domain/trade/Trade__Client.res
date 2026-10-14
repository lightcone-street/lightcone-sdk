// Trade REST client — trade-history queries. Functions take a `Client.t`,
// return `promise<result<_, SdkError.t>>`, and convert the `Trade__Raw` wire
// responses to `Trade__Model.Page.t`.

let optionalQuery = (query, key, value) =>
  value->Option.forEach(value => query->Array.push((key, value)))

// Trades for one orderbook. `cursor` is a numeric REST row id (pass a prior
// `nextCursor` to page).
let get = async (
  client: Client.t,
  ~orderbookId: string,
  ~limit: option<int>=?,
  ~cursor: option<float>=?,
): result<Trade__Model.Page.t, SdkError.t> => {
  let query = [("orderbook_id", orderbookId)]
  optionalQuery(query, "limit", limit->Option.map(value => Int.toString(value)))
  optionalQuery(query, "cursor", cursor->Option.map(value => Float.toString(value)))
  (
    await Http.get(client.http, ~path="/api/trades", ~query, ~decode=Trade__Raw.TradesResponse.t_decode)
  )->Result.map(Trade__Raw.TradesResponse.toPage)
}

// Trades across every orderbook in a market, interleaved by time.
let getByMarket = async (
  client: Client.t,
  ~marketPubkey: string,
  ~limit: option<int>=?,
  ~cursor: option<float>=?,
): result<Trade__Model.Page.t, SdkError.t> => {
  let query = [("market_pubkey", marketPubkey)]
  optionalQuery(query, "limit", limit->Option.map(value => Int.toString(value)))
  optionalQuery(query, "cursor", cursor->Option.map(value => Float.toString(value)))
  (
    await Http.get(
      client.http,
      ~path="/api/trades/market",
      ~query,
      ~decode=Trade__Raw.MarketTradesResponse.t_decode,
    )
  )->Result.map(Trade__Raw.MarketTradesResponse.toPage)
}
