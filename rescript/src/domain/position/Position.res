// Position domain — portfolio / position / token-balance types, the position
// and deposit-token-balance read paths (REST), the on-chain position
// transaction builders, and the live user balance index fed from the WS user
// channel. Pure namespace over the Position__* files.

type t = Position__Model.t
module Outcome = Position__Model.Outcome
module WalletHolding = Position__Model.WalletHolding
module Portfolio = Position__Model.Portfolio
module TokenBalance = Position__Model.TokenBalance
module ConditionalBalanceDelta = Position__Model.ConditionalBalanceDelta
module DepositAssetMetadata = Position__Model.DepositAssetMetadata
module Builders = Position__Builders
module Raw = Position__Raw
module State = Position__State
module Client = Position__Client
