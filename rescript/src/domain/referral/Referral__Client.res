// Referral client — beta-access status and referral-code redemption. All calls
// require an authenticated session.

// Current user's beta-access status plus any referral codes they own.
let getStatus = async (
  client: Client.t,
  ~cookieHeader: option<string>=?,
): result<Referral__Model.Status.t, SdkError.t> =>
  (
    await Http.get(
      client.http,
      ~path="/api/referral/status",
      ~cookieHeader?,
      ~decode=Referral__Raw.StatusResponse.t_decode,
    )
  )->Result.map(Referral__Raw.StatusResponse.toStatus)

// Redeem a referral code to gain beta access. Errors if the code is invalid,
// expired, or already at max uses.
let redeem = async (
  client: Client.t,
  ~code: string,
): result<Referral__Model.RedeemResult.t, SdkError.t> => {
  let body = JSON.Object(Dict.fromArray([("code", JSON.String(code))]))
  (
    await Http.post(
      client.http,
      ~path="/api/referral/redeem",
      ~body,
      ~decode=Referral__Raw.RedeemResponse.t_decode,
    )
  )->Result.map(Referral__Raw.RedeemResponse.toRedeemResult)
}
