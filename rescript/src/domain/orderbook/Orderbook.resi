// Orderbook domain — live depth (REST), book aggregation, the WS snapshot
// frame, and the live sorted-book container. Pure namespace: each alias
// re-exports its implementation module, whose `.resi` is the real interface.

module Aggregation = Orderbook__Model.Aggregation
module Ticker = Orderbook__Model.Ticker
module Raw = Orderbook__Raw
module State = Orderbook__State
module Client = Orderbook__Client
