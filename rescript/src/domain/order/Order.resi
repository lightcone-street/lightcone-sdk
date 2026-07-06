// Order domain — submit/cancel + user order/fill read paths (REST), the order
// wire types shared by REST and WS, and the live open-limit / trigger order
// containers. Pure namespace: each alias re-exports its implementation module,
// whose `.resi` is the real interface.

module Type = Order__Model.Type
module Limit = Order__Model.Limit
module Trigger = Order__Model.Trigger
module Raw = Order__Raw
module State = Order__State
module Client = Order__Client
