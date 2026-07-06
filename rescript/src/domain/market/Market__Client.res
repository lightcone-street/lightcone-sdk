// Market REST client — market discovery / metadata queries. Functions take a
// `Client.t`, return `promise<result<_, SdkError.t>>`, and convert the
// `Market__Raw` wire responses to `Market__Model` domain values (except the
// deposit-mints and search shapes, which are returned as wire types).

let optionalQuery = (query, key, value) =>
  value->Option.forEach(value => query->Array.push((key, value)))

// Cursor-paginated markets. Only Active and Resolved markets are kept; markets
// that fail validation are skipped and surfaced in `validationErrors`.
let get = async (
  client: Client.t,
  ~cursor: option<float>=?,
  ~limit: option<int>=?,
): result<Market__Model.MarketsResult.t, SdkError.t> => {
  let query: array<(string, string)> = []
  optionalQuery(query, "cursor", cursor->Option.map(value => Float.toString(value)))
  optionalQuery(query, "limit", limit->Option.map(value => Int.toString(value)))
  (
    await Http.get(
      client.http,
      ~path="/api/markets",
      ~query,
      ~decode=Market__Raw.MarketsResponse.t_decode,
    )
  )->Result.map(response => {
    let markets: array<Market__Model.t> = []
    let validationErrors: array<string> = []
    response.markets->Array.forEach(marketResponse =>
      switch Market__Raw.MarketResponse.toMarket(marketResponse) {
      | Ok(market) =>
        switch market.status {
        | Market__Model.Status.Active | Market__Model.Status.Resolved =>
          Array.push(markets, market)
        | Market__Model.Status.Pending | Market__Model.Status.Cancelled => ()
        }
      | Error(message) => Array.push(validationErrors, message)
      }
    )
    let result: Market__Model.MarketsResult.t = {markets, validationErrors}
    result
  })
}

// Featured markets. Only Active markets are returned.
let featured = async (client: Client.t): result<
  array<Market__Raw.MarketSearchResult.t>,
  SdkError.t,
> =>
  (await Http.get(
    client.http,
    ~path="/api/markets/search/featured",
    ~decode=(json => Spice.arrayFromJson(Market__Raw.MarketSearchResult.t_decode, json)),
  ))->Result.map(results =>
    results->Array.filter(result =>
      switch result.marketStatus {
      | Market__Model.Status.Active => true
      | _ => false
      }
    )
  )

// Fetch a single market by slug.
let getBySlug = async (client: Client.t, ~slug: string): result<Market__Model.t, SdkError.t> =>
  switch await Http.get(
    client.http,
    ~path=`/api/markets/by-slug/${slug}`,
    ~decode=Market__Raw.SingleMarketResponse.t_decode,
  ) {
  | Error(error) => Error(error)
  | Ok(response) =>
    Market__Raw.MarketResponse.toMarket(response.market)->Result.mapError(message => SdkError.Validation(
      message,
    ))
  }

// Fetch a single market by on-chain pubkey.
let getByPubkey = async (client: Client.t, ~pubkey: string): result<Market__Model.t, SdkError.t> =>
  switch await Http.get(
    client.http,
    ~path=`/api/markets/${pubkey}`,
    ~decode=Market__Raw.SingleMarketResponse.t_decode,
  ) {
  | Error(error) => Error(error)
  | Ok(response) =>
    Market__Raw.MarketResponse.toMarket(response.market)->Result.mapError(message => SdkError.Validation(
      message,
    ))
  }

// Search markets by query string.
let search = async (
  client: Client.t,
  ~query: string,
  ~limit: option<int>=?,
): result<array<Market__Raw.MarketSearchResult.t>, SdkError.t> => {
  let encoded = encodeURIComponent(query)
  let queryParams: array<(string, string)> = []
  optionalQuery(queryParams, "limit", limit->Option.map(value => Int.toString(value)))
  await Http.get(
    client.http,
    ~path=`/api/markets/search/by-query/${encoded}`,
    ~query=queryParams,
    ~decode=(json => Spice.arrayFromJson(Market__Raw.MarketSearchResult.t_decode, json)),
  )
}

// The active global deposit-asset whitelist (platform-scoped). Assets that fail
// validation are skipped and surfaced in `validationErrors`.
let globalDepositAssets = async (client: Client.t): result<
  Market__Model.GlobalDepositAssetsResult.t,
  SdkError.t,
> =>
  (await Http.get(
    client.http,
    ~path="/api/global-deposit-assets",
    ~decode=Market__Raw.GlobalDepositAssetsListResponse.t_decode,
  ))->Result.map(response => {
    let assets: array<Market__Model.GlobalDepositAsset.t> = []
    let validationErrors: array<string> = []
    response.assets->Array.forEach(assetResponse =>
      switch Market__Raw.GlobalDepositAssetResponse.toGlobalDepositAsset(assetResponse) {
      | Ok(asset) => Array.push(assets, asset)
      | Error(message) => Array.push(validationErrors, message)
      }
    )
    let result: Market__Model.GlobalDepositAssetsResult.t = {assets, validationErrors}
    result
  })

// Deposit assets registered for a specific market (with their conditional mints).
let getDepositMints = async (
  client: Client.t,
  ~marketPubkey: string,
): result<Market__Raw.DepositMintsResponse.t, SdkError.t> =>
  await Http.get(
    client.http,
    ~path=`/api/markets/${marketPubkey}/deposit-assets`,
    ~decode=Market__Raw.DepositMintsResponse.t_decode,
  )
