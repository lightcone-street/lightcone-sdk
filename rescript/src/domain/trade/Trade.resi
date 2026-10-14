// Trade domain — trade-history queries (REST) and the rolling live trade
// buffer. Pure namespace: each alias re-exports its implementation module,
// whose `.resi` is the real interface.

type t = Trade__Model.t
module Page = Trade__Model.Page
module Raw = Trade__Raw
module State = Trade__State
module Client = Trade__Client
