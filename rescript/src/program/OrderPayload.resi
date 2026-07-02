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

// ── Named constructors (Rust `new_bid` / `new_ask`) ───────────────────────────
// Pure record builders fixing the side (signatures travel separately).
let newBid: (
  ~nonce: bigint,
  ~salt: bigint,
  ~maker: SolanaKit.address,
  ~market: SolanaKit.address,
  ~baseMint: SolanaKit.address,
  ~quoteMint: SolanaKit.address,
  ~amountIn: bigint,
  ~amountOut: bigint,
  ~expiration: bigint=?,
) => t

let newAsk: (
  ~nonce: bigint,
  ~salt: bigint,
  ~maker: SolanaKit.address,
  ~market: SolanaKit.address,
  ~baseMint: SolanaKit.address,
  ~quoteMint: SolanaKit.address,
  ~amountIn: bigint,
  ~amountOut: bigint,
  ~expiration: bigint=?,
) => t

// ── Signed-order (de)serialization (233 bytes: message ‖ signature) ───────────
let serialize: (t, ~signature: Uint8Array.t) => Uint8Array.t
let deserialize: Uint8Array.t => result<(t, Uint8Array.t), string>

// ── Signature helpers ─────────────────────────────────────────────────────────
// Verify a signature against the order's maker (ed25519 over the UTF-8 bytes of
// the hex hash — the same message `sign` produces).
let verifySignature: (t, ~signature: Uint8Array.t) => promise<bool>

// Parse a base58 signature (wallet-adapter output) into the 64 raw bytes the
// request encoders expect.
let signatureFromBs58: string => result<Uint8Array.t, string>

// The canonical orderbook id for this order's token pair.
let deriveOrderbookId: t => Shared.orderBookId

// ── Compact on-chain order (37 bytes, no maker/mints — Rust `Order`) ──────────
module Compact: {
  type t = {
    // u32 on-chain (the payload's u64 nonce truncates).
    nonce: int,
    salt: bigint,
    side: int,
    amountIn: bigint,
    amountOut: bigint,
    expiration: bigint,
  }

  let serialize: t => Uint8Array.t
  let deserialize: Uint8Array.t => result<t, string>
}

// Payload → compact on-chain order (nonce truncates to u32).
let toOrder: t => Compact.t

// Compact order + account pubkeys → full payload (Rust `Order::to_signed`,
// minus the embedded signature).
let ofOrder: (
  Compact.t,
  ~maker: SolanaKit.address,
  ~market: SolanaKit.address,
  ~baseMint: SolanaKit.address,
  ~quoteMint: SolanaKit.address,
) => t

// ── Order math / predicates ───────────────────────────────────────────────────
// Expired when a non-zero expiration is at or before `currentTime` (unix seconds).
let isOrderExpired: (t, ~currentTime: bigint) => bool

// Whether a bid and an ask can cross (buyer price ≥ seller price, computed
// multiplicatively; zero amounts never cross).
let ordersCanCross: (~buyOrder: t, ~sellOrder: t) => bool

// The taker amount for a maker fill: fill * amount_out / amount_in (floor).
// Errors on a zero maker amount_in or u64 overflow.
let calculateTakerFill: (t, ~makerFillAmount: bigint) => result<bigint, string>

// keccak256(oracle ‖ question_id ‖ num_outcomes) — the market's condition id.
let deriveConditionId: (
  ~oracle: SolanaKit.address,
  ~questionId: Uint8Array.t,
  ~numOutcomes: int,
) => Uint8Array.t
