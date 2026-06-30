// Referral domain — beta-access status and referral-code redemption. Public
// surface: the gentype domain types (`referralCodeInfo`, `referralStatus`,
// `redeemResult`) and the `getStatus` / `redeem` client calls. The wire types,
// their decoders, and the wire→domain conversions are internal.

@genType
type referralCodeInfo = {
  code: string,
  maxUses: float,
  useCount: float,
}

@genType
type referralStatus = {
  isBeta: bool,
  // How the user gained access (e.g. a referral code); absent when unknown.
  source?: string,
  referralCodes: array<referralCodeInfo>,
}

@genType
type redeemResult = {
  success: bool,
  isBeta: bool,
}

// ── Client functions ─────────────────────────────────────────────────────────
// Current user's beta-access status plus any referral codes they own.
let getStatus: (Client.t, ~cookieHeader: string=?) => promise<result<referralStatus, SdkError.t>>

// Redeem a referral code to gain beta access. Errors if the code is invalid,
// expired, or already at max uses.
let redeem: (Client.t, ~code: string) => promise<result<redeemResult, SdkError.t>>
