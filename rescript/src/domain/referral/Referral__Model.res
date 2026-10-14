// Referral domain types — beta-access status and the referral codes a user
// owns. Counts (`maxUses`, `useCount`) are floats (JS numbers).

// A referral code the user owns.
module Code = {
  type t = {
    code: string,
    maxUses: float,
    useCount: float,
  }
}

// The user's beta-access status.
module Status = {
  type t = {
    isBeta: bool,
    // How the user gained access (e.g. a referral code); absent when unknown.
    source?: string,
    referralCodes: array<Code.t>,
  }
}

// Outcome of redeeming a referral code.
module RedeemResult = {
  type t = {
    success: bool,
    isBeta: bool,
  }
}

module RedeemErrorCode = {
  @genType
  type t =
    | InvalidCode
    | CodeFullyRedeemed
    | AlreadyBeta
    | SelfReferral
    | AlreadyRedeemedByUser
    | RateLimited
    | InternalError
    | Unknown(string)
  let fromWire = raw =>
    switch raw {
    | "INVALID_CODE" => InvalidCode
    | "CODE_FULLY_REDEEMED" => CodeFullyRedeemed
    | "ALREADY_BETA" => AlreadyBeta
    | "SELF_REFERRAL" => SelfReferral
    | "ALREADY_REDEEMED_BY_USER" => AlreadyRedeemedByUser
    | "RATE_LIMITED" => RateLimited
    | "INTERNAL_ERROR" => InternalError
    | _ => Unknown(raw)
    }
  let toWire = code =>
    switch code {
    | InvalidCode => "INVALID_CODE"
    | CodeFullyRedeemed => "CODE_FULLY_REDEEMED"
    | AlreadyBeta => "ALREADY_BETA"
    | SelfReferral => "SELF_REFERRAL"
    | AlreadyRedeemedByUser => "ALREADY_REDEEMED_BY_USER"
    | RateLimited => "RATE_LIMITED"
    | InternalError => "INTERNAL_ERROR"
    | Unknown(raw) => raw
    }
}
