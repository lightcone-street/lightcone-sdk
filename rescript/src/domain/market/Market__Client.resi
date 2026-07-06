// Market REST client — market discovery / metadata queries. The query-string
// helper is internal.

// Cursor-paginated markets. Only Active and Resolved markets are kept; markets
// that fail validation are skipped and surfaced in `validationErrors`.
let get: (
  Client.t,
  ~cursor: float=?,
  ~limit: int=?,
) => promise<result<Market__Model.MarketsResult.t, SdkError.t>>

// Featured markets. Only Active markets are returned.
let featured: Client.t => promise<result<array<Market__Raw.MarketSearchResult.t>, SdkError.t>>

// Fetch a single market by slug.
let getBySlug: (Client.t, ~slug: string) => promise<result<Market__Model.t, SdkError.t>>

// Fetch a single market by on-chain pubkey.
let getByPubkey: (Client.t, ~pubkey: string) => promise<result<Market__Model.t, SdkError.t>>

// Search markets by query string.
let search: (
  Client.t,
  ~query: string,
  ~limit: int=?,
) => promise<result<array<Market__Raw.MarketSearchResult.t>, SdkError.t>>

// The active global deposit-asset whitelist (platform-scoped). Assets that fail
// validation are skipped and surfaced in `validationErrors`.
let globalDepositAssets: Client.t => promise<
  result<Market__Model.GlobalDepositAssetsResult.t, SdkError.t>,
>

// Deposit assets registered for a specific market (with their conditional mints).
let getDepositMints: (
  Client.t,
  ~marketPubkey: string,
) => promise<result<Market__Raw.DepositMintsResponse.t, SdkError.t>>
