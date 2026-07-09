// Metrics domain — platform / market / orderbook / category / deposit-token
// volume metrics, history series, the market leaderboard, and per-wallet
// aggregates. Pure namespace: each alias re-exports its Metrics__*
// implementation file, whose `.resi` is the real interface.

module UniqueTradersHistoryScope = Metrics__Raw.UniqueTradersHistoryScope
module Raw = Metrics__Raw
module Client = Metrics__Client
