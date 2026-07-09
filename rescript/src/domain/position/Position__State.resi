// Live user balance state — a nested balance lookup fed from WS user snapshots
// / balance updates: market → deposit asset → conditional token → balance.
// Zero balances are dropped on the way in. The index types stay transparent
// Dict aliases (consumers walk the tree directly).

// conditional token → balance.
@genType
type conditionalTokenBalanceIndex = Dict.t<Order__Raw.UserOutcomeBalance.t>
// deposit asset → conditional token → balance.
@genType
type depositAssetBalanceIndex = Dict.t<conditionalTokenBalanceIndex>
// market → deposit asset → conditional token → balance.
@genType
type t = Dict.t<depositAssetBalanceIndex>

@genType
let make: unit => t
@genType
let get: (t, ~marketPubkey: Shared.pubkeyStr) => option<depositAssetBalanceIndex>
@genType
let insert: (t, ~marketPubkey: Shared.pubkeyStr, depositAssetBalanceIndex) => unit
@genType
let remove: (t, ~marketPubkey: Shared.pubkeyStr) => unit
// Merge `other` in: per market, per deposit asset, the other's entry wins wholesale.
@genType
let extend: (t, t) => unit
// Indexed market pubkeys, sorted (deterministic iteration).
@genType
let marketPubkeys: t => array<Shared.pubkeyStr>
// Index a single market balance; `None` when every outcome is zero.
@genType
let fromMarketBalance: Order__Raw.UserMarketBalance.t => option<t>
// Index a full set of market balances.
@genType
let fromMarketBalances: array<Order__Raw.UserMarketBalance.t> => t
