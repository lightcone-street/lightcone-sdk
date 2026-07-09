// Referral domain — beta-access status and referral-code redemption. Pure
// namespace: each alias re-exports its implementation module, whose `.resi` is
// the real interface.

module Code = Referral__Model.Code
module Status = Referral__Model.Status
module RedeemResult = Referral__Model.RedeemResult
module Raw = Referral__Raw
module Client = Referral__Client
