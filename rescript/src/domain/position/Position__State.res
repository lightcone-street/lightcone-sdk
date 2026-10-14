// Live user balance state — a nested balance lookup fed from WS user snapshots
// / balance updates: market → deposit asset → conditional token → balance.
// Zero balances are dropped on the way in; `extend` merges at the market level
// with the other's per-deposit-asset entries winning wholesale.

// conditional token → balance.
type conditionalTokenBalanceIndex = Dict.t<Order__Raw.UserOutcomeBalance.t>
// deposit asset → conditional token → balance.
type depositAssetBalanceIndex = Dict.t<conditionalTokenBalanceIndex>
// market → deposit asset → conditional token → balance.
type t = Dict.t<depositAssetBalanceIndex>

let make = (): t => Dict.make()

let get = (index: t, ~marketPubkey: Shared.pubkeyStr): option<depositAssetBalanceIndex> =>
  index->Dict.get(marketPubkey)

let insert = (index: t, ~marketPubkey: Shared.pubkeyStr, entry: depositAssetBalanceIndex): unit =>
  index->Dict.set(marketPubkey, entry)

let remove = (index: t, ~marketPubkey: Shared.pubkeyStr): unit => index->Dict.delete(marketPubkey)

// Merge `other` in: per market, per deposit asset, the other's entry wins wholesale.
let extend = (index: t, other: t): unit =>
  other
  ->Dict.toArray
  ->Array.forEach(((marketPubkey, marketEntry)) => {
    let target = switch index->Dict.get(marketPubkey) {
    | Some(existing) => existing
    | None =>
      let created = Dict.make()
      index->Dict.set(marketPubkey, created)
      created
    }
    marketEntry
    ->Dict.toArray
    ->Array.forEach(((depositAsset, outcomes)) => target->Dict.set(depositAsset, outcomes))
  })

// Indexed market pubkeys, sorted (deterministic iteration).
let marketPubkeys = (index: t): array<Shared.pubkeyStr> =>
  index->Dict.keysToArray->Array.toSorted(String.compare)

// Index a single market balance; `None` when every outcome is zero.
let fromMarketBalance = (marketBalance: Order__Raw.UserMarketBalance.t): option<t> => {
  let marketEntry: depositAssetBalanceIndex = Dict.make()
  marketBalance.depositAssets->Array.forEach(depositAssetBalance => {
    let outcomes: conditionalTokenBalanceIndex = Dict.make()
    depositAssetBalance.outcomes->Array.forEach(outcome =>
      if !Order.Raw.UserOutcomeBalance.isZero(outcome) {
        outcomes->Dict.set(outcome.conditionalToken, outcome)
      }
    )
    if Dict.keysToArray(outcomes)->Array.length > 0 {
      marketEntry->Dict.set(depositAssetBalance.depositAsset, outcomes)
    }
  })
  switch Dict.keysToArray(marketEntry)->Array.length {
  | 0 => None
  | _ =>
    let index = make()
    index->Dict.set(marketBalance.marketPubkey, marketEntry)
    Some(index)
  }
}

// Index a full set of market balances.
let fromMarketBalances = (marketBalances: array<Order__Raw.UserMarketBalance.t>): t => {
  let index = make()
  marketBalances->Array.forEach(marketBalance =>
    switch fromMarketBalance(marketBalance) {
    | Some(marketIndex) => extend(index, marketIndex)
    | None => ()
    }
  )
  index
}
