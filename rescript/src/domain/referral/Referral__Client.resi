// Referral client — beta-access status and referral-code redemption
// (authenticated session required).

// Current user's beta-access status plus any referral codes they own.
let getStatus: (
  Client.t,
  ~cookieHeader: string=?,
) => promise<result<Referral__Model.Status.t, SdkError.t>>

// Redeem a referral code to gain beta access.
let redeem: (
  Client.t,
  ~code: string,
) => promise<result<Referral__Model.RedeemResult.t, SdkError.t>>
