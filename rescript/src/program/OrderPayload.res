// Order hashing & signing — the load-bearing crux. Reproduces
// rust/src/program/orders.rs byte-for-byte:
//   1. pack the 169-byte signing message, little-endian, at fixed offsets
//   2. digest = keccak256(message)            (@noble/hashes)
//   3. hashHex = lowercase hex of the digest  (kit base16 decoder)
//   4. signature = ed25519 over the UTF-8 bytes of hashHex  (kit signBytes)
// The signed payload is the 64-char ASCII hex STRING, not the raw digest —
// "UTF-8 safe for wallet compatibility" (matches the Rust comment).

// Concatenate byte slices into one Uint8Array (the manual `extend_from_slice`).
let concatBytes: array<Uint8Array.t> => Uint8Array.t = %raw(`function (parts) {
  let total = 0;
  for (const part of parts) total += part.length;
  const out = new Uint8Array(total);
  let offset = 0;
  for (const part of parts) { out.set(part, offset); offset += part.length; }
  return out;
}`)

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

let u64 = value => SolanaKitCodec.encode(SolanaKitCodec.getU64Encoder(), value)
let i64 = value => SolanaKitCodec.encode(SolanaKitCodec.getI64Encoder(), value)
let u8 = value => SolanaKitCodec.encode(SolanaKitCodec.getU8Encoder(), value)
let address = value => SolanaKitCodec.encode(SolanaKitCodec.getAddressEncoder(), value)

// The 169-byte message: nonce@0 salt@8 maker@16 market@48 baseMint@80 quoteMint@112
// side@144 amountIn@145 amountOut@153 expiration@161 — all little-endian.
let signingMessage = (order: t): Uint8Array.t =>
  concatBytes([
    u64(order.nonce),
    u64(order.salt),
    address(order.maker),
    address(order.market),
    address(order.baseMint),
    address(order.quoteMint),
    u8(order.side),
    u64(order.amountIn),
    u64(order.amountOut),
    i64(order.expiration),
  ])

let hash = (order: t): Uint8Array.t => NobleHashes.keccak256(signingMessage(order))

// 64-char lowercase hex of the keccak digest.
let hashHex = (order: t): string => SolanaKitCodec.decode(SolanaKitCodec.getBase16Decoder(), hash(order))

// ed25519 signature (64 bytes) over the UTF-8 bytes of the hex hash.
let sign = async (order: t, keypair: SolanaKit.cryptoKeyPair): Uint8Array.t => {
  let hexBytes = SolanaKitCodec.encode(SolanaKitCodec.getUtf8Encoder(), hashHex(order))
  await SolanaKitKeys.signBytes(keypair.privateKey, hexBytes)
}

// Encodings of the 64-byte signature: hex for SubmitOrderRequest/cancel bodies,
// base58 for login/wallet-adapter flows.
let signatureHex = (signature: Uint8Array.t): string =>
  SolanaKitCodec.decode(SolanaKitCodec.getBase16Decoder(), signature)
let signatureBs58 = (signature: Uint8Array.t): string =>
  SolanaKitCodec.decode(SolanaKitCodec.getBase58Decoder(), signature)
