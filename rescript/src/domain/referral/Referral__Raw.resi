// Referral wire types + wire→domain conversions. `@spice` is kept so the codecs
// are public (the client decodes across the file boundary).

// Wire row of a referral code.
module Code: {
  @genType @spice
  type t = {
    code: string,
    @spice.key("max_uses") maxUses: float,
    @spice.key("use_count") useCount: float,
  }

  let toCode: t => Referral__Model.Code.t
}

// Wire shape of `GET /api/referral/status` (hand-decoded: `source` is nullable).
module StatusResponse: {
  @genType
  type t = {
    isBeta: bool,
    source: option<string>,
    referralCodes: array<Code.t>,
  }

  let t_decode: JSON.t => result<t, Spice.decodeError>
  let toStatus: t => Referral__Model.Status.t
}

// Wire shape of `POST /api/referral/redeem`.
module RedeemResponse: {
  @genType @spice
  type t = {
    success: bool,
    @spice.key("is_beta") isBeta: bool,
  }

  let toRedeemResult: t => Referral__Model.RedeemResult.t
}
