// Auth wire decoding + request bodies. Public surface: the nonce payload codec,
// the session decoder, and the signed-login request body (what `Auth__Client`
// needs); the identity/user decoders are internal to the session decoder.

// The GET /api/auth/nonce payload.
module Nonce: {
  @spice
  type t = {nonce: string}
}

// Decode the session envelope (hand-decoded composite).
let decodeSession: JSON.t => result<Auth__Model.Session.t, Spice.decodeError>

module SignedLogin: {
  // The login_or_register_with_message request body.
  let toJson: (Auth__Model.SignedLogin.t, ~useEmbeddedWallet: option<bool>) => JSON.t
}
