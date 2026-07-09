// Price-history domain — orderbook OHLCV / candle queries (REST), deposit-token
// price candles, the deposit-asset price snapshot, and the live WS-driven
// series containers. Pure namespace over the PriceHistory__* files.

module LineData = PriceHistory__Model.LineData
module OrderbookQuery = PriceHistory__Model.OrderbookQuery
module DepositQuery = PriceHistory__Model.DepositQuery
module Raw = PriceHistory__Raw
module State = PriceHistory__State
module DepositState = PriceHistory__DepositState
module Client = PriceHistory__Client
