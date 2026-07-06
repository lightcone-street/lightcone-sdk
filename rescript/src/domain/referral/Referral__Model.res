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
