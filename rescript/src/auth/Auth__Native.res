// Native-keypair login signing — build the sign-in message, sign it with a
// local ed25519 keypair (kit `signBytes` over the message's UTF-8 bytes), and
// package the result as the signed-login material `Auth__Client.loginWithMessage`
// takes.

// The message the wallet signs for login.
let signinMessage = (nonce: string): string => `Sign in to Lightcone\nNonce: ${nonce}`

let bytesToIntArray: Uint8Array.t => array<int> = %raw(`(bytes) => Array.from(bytes)`)

// Package signed-login material from raw ed25519 signature bytes + the signer
// address: base58-encode the signature and expand the address into its 32
// public-key bytes.
let signedLogin = (
  ~message: string,
  ~signature: Uint8Array.t,
  ~address: SolanaKit.address,
): Auth__Model.SignedLogin.t => {
  message,
  signatureBs58: SolanaKitCodec.decode(SolanaKitCodec.getBase58Decoder(), signature),
  pubkeyBytes: bytesToIntArray(SolanaKitCodec.encode(SolanaKitCodec.getAddressEncoder(), address)),
}

// Sign the login message with a wallet keypair (ed25519 over the message's UTF-8
// bytes); returns the base58 signature + 32-byte public key.
let signLoginMessage = async (
  keypair: SolanaKit.cryptoKeyPair,
  nonce: string,
): Auth__Model.SignedLogin.t => {
  let message = signinMessage(nonce)
  let messageBytes = SolanaKitCodec.encode(SolanaKitCodec.getUtf8Encoder(), message)
  let signature = await SolanaKitKeys.signBytes(keypair.privateKey, messageBytes)
  let address = await SolanaKitKeys.getAddressFromPublicKey(keypair.publicKey)
  signedLogin(~message, ~signature, ~address)
}
