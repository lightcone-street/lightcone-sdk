// Referral domain — beta-access status and referral-code redemption. Mirrors the
// Rust `domain/referral`. All calls require an authenticated session.
//
// Counts (`max_uses`, `use_count`) are floats (JS numbers).

// ── Wire types ───────────────────────────────────────────────────────────────
@spice
type referralCodeWire = {
  code: string,
  @spice.key("max_uses") maxUses: float,
  @spice.key("use_count") useCount: float,
}

@spice
type redeemResponse = {
  success: bool,
  @spice.key("is_beta") isBeta: bool,
}

// Wire shape of `GET /api/referral/status`. Hand-decoded (not `@spice`) because
// `source` is a nullable `Option<String>` with no `skip_serializing` — the
// backend sends an explicit JSON `null`, which spice's `?` optional field
// rejects. Reading it tolerantly here mirrors serde's null-or-absent handling.
type referralStatusResponse = {
  isBeta: bool,
  source: option<string>,
  referralCodes: array<referralCodeWire>,
}

let referralStatusResponseDecode = (json: JSON.t): result<referralStatusResponse, Spice.decodeError> =>
  switch json {
  | JSON.Object(dict) =>
    switch (Dict.get(dict, "is_beta"), Dict.get(dict, "referral_codes")) {
    | (Some(JSON.Boolean(isBeta)), Some(codesJson)) =>
      switch Spice.arrayFromJson(referralCodeWire_decode, codesJson) {
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

// ── Domain types ─────────────────────────────────────────────────────────────
type referralCodeInfo = {
  code: string,
  maxUses: float,
  useCount: float,
}

type referralStatus = {
  isBeta: bool,
  // How the user gained access (e.g. a referral code); absent when unknown.
  source?: string,
  referralCodes: array<referralCodeInfo>,
}

type redeemResult = {
  success: bool,
  isBeta: bool,
}

// ── Conversions (mirror the Rust `referral_status_from_wire`) ────────────────
let referralCodeInfoOfWire = (wire: referralCodeWire): referralCodeInfo => {
  code: wire.code,
  maxUses: wire.maxUses,
  useCount: wire.useCount,
}

let referralStatusOfResponse = (response: referralStatusResponse): referralStatus => {
  isBeta: response.isBeta,
  source: ?response.source,
  referralCodes: response.referralCodes->Array.map(referralCodeInfoOfWire),
}

let redeemResultOfResponse = (response: redeemResponse): redeemResult => {
  success: response.success,
  isBeta: response.isBeta,
}

// ── Client functions ─────────────────────────────────────────────────────────
// Current user's beta-access status plus any referral codes they own.
let getStatus = async (
  client: Client.t,
  ~cookieHeader: option<string>=?,
): result<referralStatus, SdkError.t> =>
  (
    await Http.get(
      client.http,
      ~path="/api/referral/status",
      ~cookieHeader?,
      ~decode=referralStatusResponseDecode,
    )
  )->Result.map(referralStatusOfResponse)

// Redeem a referral code to gain beta access. Errors if the code is invalid,
// expired, or already at max uses.
let redeem = async (client: Client.t, ~code: string): result<redeemResult, SdkError.t> => {
  let body = JSON.Object(Dict.fromArray([("code", JSON.String(code))]))
  (
    await Http.post(client.http, ~path="/api/referral/redeem", ~body, ~decode=redeemResponse_decode)
  )->Result.map(redeemResultOfResponse)
}
