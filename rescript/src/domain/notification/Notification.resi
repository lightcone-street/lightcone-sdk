// Notification domain — user notifications for market / order events. Pure
// namespace: each alias re-exports its implementation module, whose `.resi` is
// the real interface.

type t = Notification__Model.t
module Kind = Notification__Model.Kind
module MarketResolution = Notification__Model.MarketResolution
module MarketResolved = Notification__Model.MarketResolved
module OrderFilled = Notification__Model.OrderFilled
module MarketData = Notification__Model.MarketData
module Raw = Notification__Raw
module Client = Notification__Client

let isGlobal: t => bool

// The market slug associated with this notification, if any.
let marketSlug: t => option<string>
