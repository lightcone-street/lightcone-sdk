// Auth wire decoding + request bodies. `Identity`/`User`/`Session` are
// internally tagged (`type`) or composite shapes spice can't auto-derive, so
// they are hand-decoded here into the `Auth__Model` types; the leaf account
// payloads decode via their `Auth__Model` spice codecs.

// The GET /api/auth/nonce payload.
module Nonce = {
  @spice
  type t = {nonce: string}
}

// ── Decode helpers ────────────────────────────────────────────────────────────
let field = (dict, key) => Dict.get(dict, key)

// ── Identity (internally tagged) ──────────────────────────────────────────────
let decodeIdentity = (json: JSON.t): result<Auth__Model.Identity.t, Spice.decodeError> =>
  switch json {
  | JSON.Object(dict) =>
    switch field(dict, "type") {
    | Some(JSON.String("google")) =>
      switch (field(dict, "account"), field(dict, "privy")) {
      | (Some(account), Some(privy)) =>
        switch (
          Auth__Model.GoogleAccount.t_decode(account),
          Auth__Model.PrivyData.t_decode(privy),
        ) {
        | (Ok(account), Ok(privy)) => Ok(Auth__Model.Identity.Google({account, privy}))
        | (Error(error), _) | (_, Error(error)) => Error(error)
        }
      | _ => Spice.error("google identity missing account/privy", json)
      }
    | Some(JSON.String("x")) =>
      switch (field(dict, "account"), field(dict, "privy")) {
      | (Some(account), Some(privy)) =>
        switch (Auth__Model.XAccount.t_decode(account), Auth__Model.PrivyData.t_decode(privy)) {
        | (Ok(account), Ok(privy)) => Ok(Auth__Model.Identity.X({account, privy}))
        | (Error(error), _) | (_, Error(error)) => Error(error)
        }
      | _ => Spice.error("x identity missing account/privy", json)
      }
    | Some(JSON.String("wallet")) =>
      switch (field(dict, "address"), field(dict, "chain")) {
      | (Some(JSON.String(address)), Some(chainJson)) =>
        switch Auth__Model.ChainType.t_decode(chainJson) {
        | Ok(chain) =>
          let privy =
            field(dict, "privy")->Option.flatMap(json =>
              Auth__Model.PrivyData.t_decode(json)->Result.mapOr(None, value => Some(value))
            )
          Ok(Auth__Model.Identity.Wallet({address, chain, ?privy}))
        | Error(error) => Error(error)
        }
      | _ => Spice.error("wallet identity missing address/chain", json)
      }
    | _ => Spice.error("unknown identity type", json)
    }
  | _ => Spice.error("identity is not an object", json)
  }

// ── User + session (composite) ────────────────────────────────────────────────
let decodeUser = (json: JSON.t): result<Auth__Model.User.t, Spice.decodeError> =>
  switch json {
  | JSON.Object(dict) =>
    switch (field(dict, "user_id"), field(dict, "identity")) {
    | (Some(JSON.String(userId)), Some(identityJson)) =>
      switch decodeIdentity(identityJson) {
      | Ok(identity) =>
        let connectedX =
          field(dict, "connected_x")->Option.flatMap(json =>
            Auth__Model.XAccount.t_decode(json)->Result.mapOr(None, value => Some(value))
          )
        Ok({userId, identity, ?connectedX})
      | Error(error) => Error(error)
      }
    | _ => Spice.error("user missing user_id/identity", json)
    }
  | _ => Spice.error("user is not an object", json)
  }

let decodeSession = (json: JSON.t): result<Auth__Model.Session.t, Spice.decodeError> =>
  switch json {
  | JSON.Object(dict) =>
    switch (
      field(dict, "user"),
      field(dict, "expires_at"),
      field(dict, "auth_method"),
      field(dict, "is_beta"),
    ) {
    | (
        Some(userJson),
        Some(JSON.Number(expiresAt)),
        Some(authMethodJson),
        Some(JSON.Boolean(isBeta)),
      ) =>
      switch (decodeUser(userJson), Auth__Model.Method.t_decode(authMethodJson)) {
      | (Ok(user), Ok(authMethod)) => Ok({user, expiresAt, authMethod, isBeta})
      | (Error(error), _) | (_, Error(error)) => Error(error)
      }
    | _ => Spice.error("session response missing fields", json)
    }
  | _ => Spice.error("session response is not an object", json)
  }

// ── Request bodies ────────────────────────────────────────────────────────────
module SignedLogin = {
  // The login_or_register_with_message request body.
  let toJson = (signed: Auth__Model.SignedLogin.t, ~useEmbeddedWallet: option<bool>): JSON.t => {
    let fields = [
      ("message", JSON.String(signed.message)),
      ("signature_bs58", JSON.String(signed.signatureBs58)),
      (
        "pubkey_bytes",
        JSON.Array(signed.pubkeyBytes->Array.map(byte => JSON.Number(Int.toFloat(byte)))),
      ),
    ]
    useEmbeddedWallet->Option.forEach(value =>
      fields->Array.push(("use_embedded_wallet", JSON.Boolean(value)))
    )
    JSON.Object(Dict.fromArray(fields))
  }
}
