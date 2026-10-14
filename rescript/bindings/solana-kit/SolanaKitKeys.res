// `SolanaKit.Keys` — ed25519 keypairs + arbitrary-message signing (WebCrypto,
// async). `signBytes` signs raw message bytes (order hashes, the login message) —
// never a transaction.

// Wallet file is the 64-byte [seed||pubkey] Solana id.json format.
@module("@solana/kit")
external createKeyPairFromBytes: Uint8Array.t => promise<SolanaKit.cryptoKeyPair> = "createKeyPairFromBytes"
@module("@solana/kit")
external createKeyPairFromPrivateKeyBytes: Uint8Array.t => promise<SolanaKit.cryptoKeyPair> =
  "createKeyPairFromPrivateKeyBytes"
@module("@solana/kit")
external createKeyPairSignerFromBytes: Uint8Array.t => promise<SolanaKit.keyPairSigner> =
  "createKeyPairSignerFromBytes"

@module("@solana/kit")
external signBytes: (SolanaKit.cryptoKey, Uint8Array.t) => promise<Uint8Array.t> = "signBytes"
@module("@solana/kit")
external verifySignature: (SolanaKit.cryptoKey, Uint8Array.t, Uint8Array.t) => promise<bool> =
  "verifySignature"
@module("@solana/kit")
external getAddressFromPublicKey: SolanaKit.cryptoKey => promise<SolanaKit.address> = "getAddressFromPublicKey"
@get external signerAddress: SolanaKit.keyPairSigner => SolanaKit.address = "address"
