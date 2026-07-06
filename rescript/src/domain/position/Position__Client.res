// Position REST client — portfolio-wide and per-market position queries plus
// the authenticated user's SPL deposit-token balances. Functions take a
// `Client.t`, return `promise<result<_, SdkError.t>>`, and return the
// `Position__Raw` wire responses directly.
//
// Note: the on-chain position ops live elsewhere — every deposit / withdraw /
// merge / redeem flow and the position-token init/extend/close builders in
// `Position__Builders.res` (over the instruction builders in
// `program/Instructions.res`), and the on-chain account reads in `Rpc.res`
// (getExchange / getMarket / getOrderbook / getPosition + PDA helpers).

// All positions for a user across every market. Public path-based endpoint.
let get = async (
  client: Client.t,
  ~userPubkey: string,
): result<Position__Raw.PositionsResponse.t, SdkError.t> =>
  await Http.get(
    client.http,
    ~path=`/api/users/${userPubkey}/positions`,
    ~decode=Position__Raw.PositionsResponse.t_decode,
  )

// Positions for a user in a specific market. Public path-based endpoint.
let getForMarket = async (
  client: Client.t,
  ~userPubkey: string,
  ~marketPubkey: string,
): result<Position__Raw.MarketPositionsResponse.t, SdkError.t> =>
  await Http.get(
    client.http,
    ~path=`/api/users/${userPubkey}/markets/${marketPubkey}/positions`,
    ~decode=Position__Raw.MarketPositionsResponse.t_decode,
  )

// All positions for the authenticated user (wallet resolved server-side from the
// auth cookie). Pass `~cookieHeader` to forward a per-request cookie (SSR).
let positions = async (
  client: Client.t,
  ~cookieHeader: option<string>=?,
): result<Position__Raw.PositionsResponse.t, SdkError.t> =>
  await Http.get(
    client.http,
    ~path="/api/users/positions",
    ~cookieHeader?,
    ~decode=Position__Raw.PositionsResponse.t_decode,
  )

// The authenticated user's positions in a specific market.
let positionsForMarket = async (
  client: Client.t,
  ~marketPubkey: string,
  ~cookieHeader: option<string>=?,
): result<Position__Raw.MarketPositionsResponse.t, SdkError.t> =>
  await Http.get(
    client.http,
    ~path=`/api/users/markets/${marketPubkey}/positions`,
    ~cookieHeader?,
    ~decode=Position__Raw.MarketPositionsResponse.t_decode,
  )

// SPL deposit-token balances for the authenticated user, keyed by mint pubkey.
// An empty map means the user holds none of the tracked balances (not an error).
let depositTokenBalances = async (
  client: Client.t,
  ~cookieHeader: option<string>=?,
): result<dict<Position__Raw.DepositTokenBalance.t>, SdkError.t> =>
  await Http.get(
    client.http,
    ~path="/api/users/deposit-token-balances",
    ~cookieHeader?,
    ~decode=json => Spice.dictFromJson(Position__Raw.DepositTokenBalance.t_decode, json),
  )
