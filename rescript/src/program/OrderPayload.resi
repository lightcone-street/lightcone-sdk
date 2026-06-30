// Order hashing & signing. Public surface: the order record `t`, the message/
// hash/hex primitives, the `sign` flow, and the signature encoders. The LE byte
// encoders are internal. `concatBytes` is exposed because the instruction
// builders reuse it.

type t = {
  nonce: bigint,
  salt: bigint,
  maker: SolanaKit.address,
  market: SolanaKit.address,
  baseMint: SolanaKit.address,
  quoteMint: SolanaKit.address,
  // 0 = Bid, 1 = Ask.
  side: int,
  amountIn: bigint,
  amountOut: bigint,
  expiration: bigint,
}

// Concatenate byte slices into one Uint8Array (reused by Instructions).
let concatBytes: array<Uint8Array.t> => Uint8Array.t

// The 169-byte little-endian signing message at fixed offsets.
let signingMessage: t => Uint8Array.t

// keccak256 digest (32 bytes) of the signing message.
let hash: t => Uint8Array.t

// 64-char lowercase hex of the keccak digest (the bytes that get signed).
let hashHex: t => string

// ed25519 signature (64 bytes) over the UTF-8 bytes of the order's hex hash.
let sign: (t, SolanaKit.cryptoKeyPair) => promise<Uint8Array.t>

// Encodings of a 64-byte signature: hex for order/cancel bodies, base58 for
// login / wallet-adapter flows.
let signatureHex: Uint8Array.t => string
let signatureBs58: Uint8Array.t => string
