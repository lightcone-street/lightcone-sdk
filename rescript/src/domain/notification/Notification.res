// Notification domain — user notifications for market / order events. Pure
// namespace over the Notification__* files.

type t = Notification__Model.t
module Kind = Notification__Model.Kind
module MarketResolution = Notification__Model.MarketResolution
module MarketResolved = Notification__Model.MarketResolved
module OrderFilled = Notification__Model.OrderFilled
module MarketData = Notification__Model.MarketData
module Raw = Notification__Raw
module Client = Notification__Client

let isGlobal = Notification__Model.isGlobal
let marketSlug = Notification__Model.marketSlug
