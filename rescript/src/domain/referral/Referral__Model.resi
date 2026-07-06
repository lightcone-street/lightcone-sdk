// Referral domain types — beta-access status and owned referral codes.

// A referral code the user owns.
module Code: {
  @genType
  type t = {
    code: string,
    maxUses: float,
    useCount: float,
  }
}

// The user's beta-access status.
module Status: {
  @genType
  type t = {
    isBeta: bool,
    // How the user gained access (e.g. a referral code); absent when unknown.
    source?: string,
    referralCodes: array<Code.t>,
  }
}

// Outcome of redeeming a referral code.
module RedeemResult: {
  @genType
  type t = {
    success: bool,
    isBeta: bool,
  }
}
