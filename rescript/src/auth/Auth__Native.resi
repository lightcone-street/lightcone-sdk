// Native-keypair login signing — the sign-in message builder, the signed-login
// packager, and keypair signing (kit `signBytes`). `Auth__Client.login` composes
// these over the client's signing strategy; call `signLoginMessage` directly to
// drive `Auth__Client.loginWithMessage` with a raw keypair.

// The message the wallet signs for login: "Sign in to Lightcone\nNonce: {nonce}".
let signinMessage: string => string

// Package signed-login material from raw ed25519 signature bytes + the signer
// address (base58 signature + 32 public-key bytes).
let signedLogin: (
  ~message: string,
  ~signature: Uint8Array.t,
  ~address: SolanaKit.address,
) => Auth__Model.SignedLogin.t

// Sign the login message with a wallet keypair (ed25519 over the message's
// UTF-8 bytes); returns the base58 signature + 32-byte public key.
let signLoginMessage: (SolanaKit.cryptoKeyPair, string) => promise<Auth__Model.SignedLogin.t>
