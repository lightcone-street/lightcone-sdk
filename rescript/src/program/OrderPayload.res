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

// ── Named constructors (Rust `new_bid` / `new_ask`) ───────────────────────────
// The ReScript payload carries no embedded signature (signatures travel
// separately), so these are pure record builders fixing the side.
let newBid = (
  ~nonce: bigint,
  ~salt: bigint,
  ~maker: SolanaKit.address,
  ~market: SolanaKit.address,
  ~baseMint: SolanaKit.address,
  ~quoteMint: SolanaKit.address,
  ~amountIn: bigint,
  ~amountOut: bigint,
  ~expiration: bigint=0n,
): t => {nonce, salt, maker, market, baseMint, quoteMint, side: 0, amountIn, amountOut, expiration}

let newAsk = (
  ~nonce: bigint,
  ~salt: bigint,
  ~maker: SolanaKit.address,
  ~market: SolanaKit.address,
  ~baseMint: SolanaKit.address,
  ~quoteMint: SolanaKit.address,
  ~amountIn: bigint,
  ~amountOut: bigint,
  ~expiration: bigint=0n,
): t => {nonce, salt, maker, market, baseMint, quoteMint, side: 1, amountIn, amountOut, expiration}

// ── Signed-order (de)serialization (233 bytes) ────────────────────────────────
// Layout: the 169-byte signing message followed by the 64-byte signature.
let serialize = (order: t, ~signature: Uint8Array.t): Uint8Array.t =>
  concatBytes([signingMessage(order), signature])

let bytesAt: (Uint8Array.t, int, int) => Uint8Array.t = %raw(`function (bytes, offset, length) {
  return bytes.subarray(offset, offset + length);
}`)
let byteLength: Uint8Array.t => int = %raw(`(bytes) => bytes.length`)

let u64At = (bytes, offset): bigint =>
  SolanaKitCodec.decode(SolanaKitCodec.getU64Decoder(), bytesAt(bytes, offset, 8))
let i64At = (bytes, offset): bigint =>
  SolanaKitCodec.decode(SolanaKitCodec.getI64Decoder(), bytesAt(bytes, offset, 8))
let u8At = (bytes, offset): int =>
  SolanaKitCodec.decode(SolanaKitCodec.getU8Decoder(), bytesAt(bytes, offset, 1))
let addressAt = (bytes, offset): SolanaKit.address =>
  SolanaKitCodec.decode(SolanaKitCodec.getAddressDecoder(), bytesAt(bytes, offset, 32))

// Deserialize a 233-byte signed order into the payload + its signature.
let deserialize = (bytes: Uint8Array.t): result<(t, Uint8Array.t), string> =>
  if byteLength(bytes) < Constants.signedOrderSize {
    Error(
      `invalid signed-order length (expected ${Int.toString(Constants.signedOrderSize)}, got ${Int.toString(
          byteLength(bytes),
        )})`,
    )
  } else {
    switch u8At(bytes, 144) {
    | (0 | 1) as side =>
      Ok((
        {
          nonce: u64At(bytes, 0),
          salt: u64At(bytes, 8),
          maker: addressAt(bytes, 16),
          market: addressAt(bytes, 48),
          baseMint: addressAt(bytes, 80),
          quoteMint: addressAt(bytes, 112),
          side,
          amountIn: u64At(bytes, 145),
          amountOut: u64At(bytes, 153),
          expiration: i64At(bytes, 161),
        },
        bytesAt(bytes, 169, 64),
      ))
    | other => Error(`invalid order side: ${Int.toString(other)}`)
    }
  }

// ── Signature helpers ─────────────────────────────────────────────────────────
// Import a 32-byte ed25519 public key for WebCrypto verification.
let publicKeyFromBytes: Uint8Array.t => promise<SolanaKit.cryptoKey> = %raw(`(bytes) =>
  crypto.subtle.importKey("raw", bytes, "Ed25519", true, ["verify"])`)

// Verify a signature against the order's maker (ed25519 over the UTF-8 bytes of
// the hex hash — the same message `sign` produces).
let verifySignature = async (order: t, ~signature: Uint8Array.t): bool => {
  let makerKey = await publicKeyFromBytes(
    SolanaKitCodec.encode(SolanaKitCodec.getAddressEncoder(), order.maker),
  )
  let hexBytes = SolanaKitCodec.encode(SolanaKitCodec.getUtf8Encoder(), hashHex(order))
  await SolanaKitKeys.verifySignature(makerKey, signature, hexBytes)
}

// Parse a base58 signature (wallet-adapter output) into the 64 raw bytes the
// request encoders expect (Rust `apply_signature`).
let signatureFromBs58 = (sigBs58: string): result<Uint8Array.t, string> =>
  switch SolanaKitCodec.encode(SolanaKitCodec.getBase58Encoder(), sigBs58) {
  | bytes => byteLength(bytes) == 64 ? Ok(bytes) : Error("invalid signature: expected 64 bytes")
  | exception JsExn(_) => Error("invalid signature: not base58")
  }

// The canonical orderbook id for this order's token pair.
let deriveOrderbookId = (order: t): Shared.orderBookId =>
  Shared.deriveOrderbookId(
    ~baseToken=SolanaKit.addressToString(order.baseMint),
    ~quoteToken=SolanaKit.addressToString(order.quoteMint),
  )

// ── Compact on-chain order (37 bytes, no maker/mints — Rust `Order`) ──────────
module Compact = {
  type t = {
    // u32 on-chain (the payload's u64 nonce truncates).
    nonce: int,
    salt: bigint,
    side: int,
    amountIn: bigint,
    amountOut: bigint,
    expiration: bigint,
  }

  let u32 = (value: int) => SolanaKitCodec.encode(SolanaKitCodec.getU32Encoder(), value)
  let u32At = (bytes, offset): int =>
    SolanaKitCodec.decode(SolanaKitCodec.getU32Decoder(), bytesAt(bytes, offset, 4))

  // Layout: nonce u32 @0, salt u64 @4, side u8 @12, amount_in u64 @13,
  // amount_out u64 @21, expiration i64 @29.
  let serialize = (order: t): Uint8Array.t =>
    concatBytes([
      u32(order.nonce),
      u64(order.salt),
      u8(order.side),
      u64(order.amountIn),
      u64(order.amountOut),
      i64(order.expiration),
    ])

  let deserialize = (bytes: Uint8Array.t): result<t, string> =>
    if byteLength(bytes) < Constants.orderSize {
      Error(
        `invalid order length (expected ${Int.toString(Constants.orderSize)}, got ${Int.toString(
            byteLength(bytes),
          )})`,
      )
    } else {
      switch u8At(bytes, 12) {
      | (0 | 1) as side =>
        Ok({
          nonce: u32At(bytes, 0),
          salt: u64At(bytes, 4),
          side,
          amountIn: u64At(bytes, 13),
          amountOut: u64At(bytes, 21),
          expiration: i64At(bytes, 29),
        })
      | other => Error(`invalid order side: ${Int.toString(other)}`)
      }
    }
}

// Payload → compact on-chain order (nonce truncates to u32).
let toOrder = (order: t): Compact.t => {
  nonce: BigInt.toFloat(BigInt.mod(order.nonce, 4294967296n))->Float.toInt,
  salt: order.salt,
  side: order.side,
  amountIn: order.amountIn,
  amountOut: order.amountOut,
  expiration: order.expiration,
}

// Compact order + account pubkeys → full payload (Rust `Order::to_signed`,
// minus the embedded signature).
let ofOrder = (
  compact: Compact.t,
  ~maker: SolanaKit.address,
  ~market: SolanaKit.address,
  ~baseMint: SolanaKit.address,
  ~quoteMint: SolanaKit.address,
): t => {
  nonce: BigInt.fromInt(compact.nonce),
  salt: compact.salt,
  maker,
  market,
  baseMint,
  quoteMint,
  side: compact.side,
  amountIn: compact.amountIn,
  amountOut: compact.amountOut,
  expiration: compact.expiration,
}

// ── Order math / predicates (Rust program/orders.rs helpers) ──────────────────
// Expired when a non-zero expiration is at or before `currentTime` (unix seconds).
let isOrderExpired = (order: t, ~currentTime: bigint): bool =>
  order.expiration != 0n && currentTime >= order.expiration

// Whether a bid and an ask can cross: buyer price ≥ seller price, computed
// multiplicatively (no division), zero amounts never cross.
let ordersCanCross = (~buyOrder: t, ~sellOrder: t): bool =>
  if buyOrder.side != 0 || sellOrder.side != 1 {
    false
  } else if (
    buyOrder.amountIn == 0n ||
    buyOrder.amountOut == 0n ||
    sellOrder.amountIn == 0n ||
    sellOrder.amountOut == 0n
  ) {
    false
  } else {
    BigInt.mul(buyOrder.amountIn, sellOrder.amountIn) >=
      BigInt.mul(buyOrder.amountOut, sellOrder.amountOut)
  }

let u64Max = 18446744073709551615n

// The taker amount for a maker fill: fill * amount_out / amount_in (floor).
// Errors on a zero maker amount_in or u64 overflow.
let calculateTakerFill = (makerOrder: t, ~makerFillAmount: bigint): result<bigint, string> =>
  if makerOrder.amountIn == 0n {
    Error("maker order has zero amount_in")
  } else {
    let value = BigInt.div(BigInt.mul(makerFillAmount, makerOrder.amountOut), makerOrder.amountIn)
    value > u64Max ? Error("taker fill overflows u64") : Ok(value)
  }

// keccak256(oracle ‖ question_id ‖ num_outcomes) — the market's condition id.
let deriveConditionId = (
  ~oracle: SolanaKit.address,
  ~questionId: Uint8Array.t,
  ~numOutcomes: int,
): Uint8Array.t =>
  NobleHashes.keccak256(
    concatBytes([
      SolanaKitCodec.encode(SolanaKitCodec.getAddressEncoder(), oracle),
      questionId,
      u8(numOutcomes),
    ]),
  )
