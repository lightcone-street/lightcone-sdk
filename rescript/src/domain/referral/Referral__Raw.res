// Referral wire types + wire→domain conversions.

// Wire row of a referral code.
module Code = {
  @spice
  type t = {
    code: string,
    @spice.key("max_uses") maxUses: float,
    @spice.key("use_count") useCount: float,
  }

  let toCode = (wire: t): Referral__Model.Code.t => {
    code: wire.code,
    maxUses: wire.maxUses,
    useCount: wire.useCount,
  }
}

// Wire shape of `GET /api/referral/status`. Hand-decoded (not `@spice`) because
// `source` is nullable on the wire — the backend sends an explicit JSON `null`,
// which spice's `?` optional field rejects; reading it tolerantly here handles
// null-or-absent uniformly.
module StatusResponse = {
  type t = {
    isBeta: bool,
    source: option<string>,
    referralCodes: array<Code.t>,
  }

  let t_decode = (json: JSON.t): result<t, Spice.decodeError> =>
    switch json {
    | JSON.Object(dict) =>
      switch (Dict.get(dict, "is_beta"), Dict.get(dict, "referral_codes")) {
      | (Some(JSON.Boolean(isBeta)), Some(codesJson)) =>
        switch Spice.arrayFromJson(Code.t_decode, codesJson) {
        | Ok(referralCodes) =>
          let source = switch Dict.get(dict, "source") {
          | Some(JSON.String(value)) => Some(value)
          | _ => None
          }
          Ok({isBeta, source, referralCodes})
        | Error(error) => Error(error)
        }
      | _ => Spice.error("referral status missing is_beta/referral_codes", json)
      }
    | _ => Spice.error("referral status is not an object", json)
    }

  let toStatus = (response: t): Referral__Model.Status.t => {
    isBeta: response.isBeta,
    source: ?response.source,
    referralCodes: response.referralCodes->Array.map(Code.toCode),
  }
}

// Wire shape of `POST /api/referral/redeem`.
module RedeemResponse = {
  @spice
  type t = {
    success: bool,
    @spice.key("is_beta") isBeta: bool,
  }

  let toRedeemResult = (response: t): Referral__Model.RedeemResult.t => {
    success: response.success,
    isBeta: response.isBeta,
  }
}
