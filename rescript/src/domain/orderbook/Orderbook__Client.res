// Orderbook depth client — live REST depth, optionally aggregated. Decodes
// straight into the `Orderbook__Raw` wire types (no domain conversion).

let optionalQuery = (query, key, value) =>
  value->Option.forEach(value => query->Array.push((key, value)))

// Live orderbook depth, optionally aggregated. `depth` is capped server-side at
// 20 levels per side. Invalid aggregation combinations are rejected client-side
// before any request is made. Only `depth`, `nSigFigs`, and `mantissa` are sent.
let get = async (
  client: Client.t,
  ~orderbookId: string,
  ~depth: option<int>=?,
  ~aggregation: Orderbook__Model.Aggregation.t=Orderbook__Model.Aggregation.full,
  ~cookieHeader: option<string>=?,
): result<Orderbook__Raw.DepthResponse.t, SdkError.t> =>
  switch Orderbook__Model.Aggregation.validate(aggregation.nSigFigs, aggregation.mantissa) {
  | Error(message) => Error(SdkError.Validation(message))
  | Ok(validated) => {
      let query: array<(string, string)> = []
      optionalQuery(query, "depth", depth->Option.map(value => Int.toString(value)))
      optionalQuery(query, "nSigFigs", validated.nSigFigs->Option.map(value => Int.toString(value)))
      optionalQuery(query, "mantissa", validated.mantissa->Option.map(value => Int.toString(value)))
      await Http.get(
        client.http,
        ~path=`/api/orderbook/${orderbookId}`,
        ~query,
        ~cookieHeader?,
        ~decode=Orderbook__Raw.DepthResponse.t_decode,
      )
    }
  }

// Convenience: depth with an explicit aggregation view.
let getWithAggregation = (
  client: Client.t,
  ~orderbookId: string,
  ~aggregation: Orderbook__Model.Aggregation.t,
  ~depth: option<int>=?,
  ~cookieHeader: option<string>=?,
): promise<result<Orderbook__Raw.DepthResponse.t, SdkError.t>> =>
  get(client, ~orderbookId, ~depth?, ~aggregation, ~cookieHeader?)

// Convenience: depth with an explicit auth cookie (Node/Bun, no cookie jar).
let getWithCookies = (
  client: Client.t,
  ~orderbookId: string,
  ~cookieHeader: string,
  ~depth: option<int>=?,
  ~aggregation: Orderbook__Model.Aggregation.t=Orderbook__Model.Aggregation.full,
): promise<result<Orderbook__Raw.DepthResponse.t, SdkError.t>> =>
  get(client, ~orderbookId, ~depth?, ~aggregation, ~cookieHeader=?Some(cookieHeader))
