// Position REST client — portfolio-wide and per-market position queries plus
// the authenticated user's SPL deposit-token balances. The read endpoints
// return the `Position__Raw` wire responses directly.
//
// Note: the on-chain position ops live elsewhere — the deposit / withdraw /
// merge / redeem flows and position-token builders in
// `Position__Builders.res`, and the on-chain account reads in `Rpc.res`.

// All positions for a user across every market. Public path-based endpoint.
let get: (
  Client.t,
  ~userPubkey: string,
) => promise<result<Position__Raw.PositionsResponse.t, SdkError.t>>

// Positions for a user in a specific market. Public path-based endpoint.
let getForMarket: (
  Client.t,
  ~userPubkey: string,
  ~marketPubkey: string,
) => promise<result<Position__Raw.MarketPositionsResponse.t, SdkError.t>>

// All positions for the authenticated user (wallet resolved server-side from the
// auth cookie). Pass `~cookieHeader` to forward a per-request cookie (SSR).
let positions: (
  Client.t,
  ~cookieHeader: string=?,
) => promise<result<Position__Raw.PositionsResponse.t, SdkError.t>>

// The authenticated user's positions in a specific market.
let positionsForMarket: (
  Client.t,
  ~marketPubkey: string,
  ~cookieHeader: string=?,
) => promise<result<Position__Raw.MarketPositionsResponse.t, SdkError.t>>

// SPL deposit-token balances for the authenticated user, keyed by mint pubkey.
// An empty map means the user holds none of the tracked balances (not an error).
let depositTokenBalances: (
  Client.t,
  ~cookieHeader: string=?,
) => promise<result<dict<Position__Raw.DepositTokenBalance.t>, SdkError.t>>
