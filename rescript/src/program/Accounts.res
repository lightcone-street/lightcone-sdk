// On-chain account decoders.
// Each account's DATA bytes (a Uint8Array, base64-decoded from getAccountInfo)
// open with an 8-byte discriminator we verify against `Constants.Discriminator.*`,
// then a fixed, little-endian field layout we read at exact offsets. The offsets
// here MUST match the on-chain program's account layouts — the program is the
// source of truth.
//
// Numeric convention (see Rpc.res / the project memo): domain numerics → `float`;
// pubkeys → `Shared.pubkeyStr`; u64 counters/ids that need exactness → `bigint`.
// 32-byte non-address ids (question_id / condition_id) surface as lowercase hex.

// ── Account sizes (bytes), as allocated by the on-chain program ────────────────
module Size = {
  let exchange = 216
  let market = 216
  let position = 80
  let userNonce = 16
  let orderbook = 144
  let orderStatus = 32
  let globalDepositToken = 47
}

// ── Low-level field readers (SolanaKitCodec decoders over byte slices) ─────────
// Decoders are created once; kit's fixed-size decoders read exactly N bytes from
// offset 0, so we slice to the precise field width before decoding.
let u8Decoder = SolanaKitCodec.getU8Decoder()
let u16Decoder = SolanaKitCodec.getU16Decoder()
let u32Decoder = SolanaKitCodec.getU32Decoder()
let u64Decoder = SolanaKitCodec.getU64Decoder()
let addressDecoder = SolanaKitCodec.getAddressDecoder()
let base16Decoder = SolanaKitCodec.getBase16Decoder()

// A zero-copy view of `length` bytes at `offset` (kit decoders only read it).
let bytesAt: (Uint8Array.t, int, int) => Uint8Array.t = %raw(`function (bytes, offset, length) {
  return bytes.subarray(offset, offset + length);
}`)

let byteLength: Uint8Array.t => int = %raw(`(bytes) => bytes.length`)

let u8At = (bytes, offset): int => SolanaKitCodec.decode(u8Decoder, bytesAt(bytes, offset, 1))
let u16At = (bytes, offset): int => SolanaKitCodec.decode(u16Decoder, bytesAt(bytes, offset, 2))
let u32At = (bytes, offset): int => SolanaKitCodec.decode(u32Decoder, bytesAt(bytes, offset, 4))
let u64At = (bytes, offset): bigint => SolanaKitCodec.decode(u64Decoder, bytesAt(bytes, offset, 8))
let boolAt = (bytes, offset): bool => u8At(bytes, offset) != 0

// Signed 16-bit LE: kit has no i16 decoder, so read u16 and fold the sign bit.
let i16At = (bytes, offset): float => {
  let raw = u16At(bytes, offset)
  Int.toFloat(raw >= 32768 ? raw - 65536 : raw)
}

// 32 bytes → base58 address string (a `Shared.pubkeyStr`).
let addressAt = (bytes, offset): Shared.pubkeyStr =>
  SolanaKitCodec.decode(addressDecoder, bytesAt(bytes, offset, 32))->SolanaKit.addressToString

// `length` raw bytes → lowercase hex string (for non-address 32-byte ids).
let hexAt = (bytes, offset, length): string =>
  SolanaKitCodec.decode(base16Decoder, bytesAt(bytes, offset, length))

// First-8-bytes discriminator equality against a `Constants.Discriminator.*` array.
let discriminatorMatches: (Uint8Array.t, array<int>) => bool = %raw(`function (bytes, expected) {
  if (bytes.length < expected.length) return false;
  for (let i = 0; i < expected.length; i++) { if (bytes[i] !== expected[i]) return false; }
  return true;
}`)

// ── Enums (decoded from a single byte, per the program's type definitions) ─────
// Market lifecycle status, stored as a u8. This is the on-chain (program-layer)
// status — distinct from the REST `Market.Status` string enum.
module MarketStatus = {
  type t =
    | @as("Pending") Pending
    | @as("Active") Active
    | @as("Resolved") Resolved
    | @as("Cancelled") Cancelled

  let fromU8 = (value: int): result<t, SdkError.t> =>
    switch value {
    | 0 => Ok(Pending)
    | 1 => Ok(Active)
    | 2 => Ok(Resolved)
    | 3 => Ok(Cancelled)
    | other => Error(SdkError.Program(`Invalid market status: ${Int.toString(other)}`))
    }

  let toString = (status: t): string =>
    switch status {
    | Pending => "Pending"
    | Active => "Active"
    | Resolved => "Resolved"
    | Cancelled => "Cancelled"
    }
}

// Pending privileged-role transfer kind stored on the Exchange account.
module PendingRoleKind = {
  type t =
    | @as("None") None
    | @as("Authority") Authority
    | @as("Manager") Manager
    | @as("Operator") Operator

  let fromU8 = (value: int): result<t, SdkError.t> =>
    switch value {
    | 0 => Ok(None)
    | 1 => Ok(Authority)
    | 2 => Ok(Manager)
    | 3 => Ok(Operator)
    | other => Error(SdkError.Program(`Invalid pending role kind: ${Int.toString(other)}`))
    }
}

// ── Decode guards ──────────────────────────────────────────────────────────────
// Length + discriminator guard run before every field read.
let checkHeader = (
  bytes: Uint8Array.t,
  size: int,
  discriminator: array<int>,
  name: string,
): result<unit, SdkError.t> =>
  if byteLength(bytes) < size {
    Error(
      SdkError.Program(
        `${name}: invalid data length (expected ${Int.toString(size)}, got ${Int.toString(
            byteLength(bytes),
          )})`,
      ),
    )
  } else if !discriminatorMatches(bytes, discriminator) {
    Error(SdkError.Program(`${name}: invalid account discriminator`))
  } else {
    Ok()
  }

// Run a decode body, converting any kit decoder exception into an SdkError.
let guard = (name: string, body: unit => result<'a, SdkError.t>): result<'a, SdkError.t> =>
  switch body() {
  | decoded => decoded
  | exception JsExn(error) =>
    Error(SdkError.Program(`${name}: decode failed (${error->JsExn.message->Option.getOr("decode error")})`))
  }

// ── Domain records (the clean, gentype-exported account shapes) ────────────────
// The 8-byte discriminator is verified, not surfaced. Reserved/padding is dropped.
// Each account row is a module: its record `t` plus the byte `decode` entry point.

// Exchange — singleton exchange state (216 bytes).
module Exchange = {
  type t = {
    authority: Shared.pubkeyStr,
    operator: Shared.pubkeyStr,
    manager: Shared.pubkeyStr,
    marketCount: bigint,
    paused: bool,
    bump: float,
    depositTokenCount: float,
    feeReceiver: Shared.pubkeyStr,
    pendingRole: Shared.pubkeyStr,
    pendingRoleKind: PendingRoleKind.t,
  }

  let decode = (bytes: Uint8Array.t): result<t, SdkError.t> =>
    guard("Exchange", () =>
      switch checkHeader(bytes, Size.exchange, Constants.Discriminator.exchange, "Exchange") {
      | Error(error) => Error(error)
      | Ok() =>
        switch PendingRoleKind.fromU8(u8At(bytes, 180)) {
        | Error(error) => Error(error)
        | Ok(pendingRoleKind) =>
          Ok({
            authority: addressAt(bytes, 8),
            operator: addressAt(bytes, 40),
            manager: addressAt(bytes, 72),
            marketCount: u64At(bytes, 104),
            paused: boolAt(bytes, 112),
            bump: Int.toFloat(u8At(bytes, 113)),
            depositTokenCount: Int.toFloat(u16At(bytes, 114)),
            feeReceiver: addressAt(bytes, 116),
            pendingRole: addressAt(bytes, 148),
            pendingRoleKind,
          })
        }
      }
    )
}

// Market — a prediction market (216 bytes). `questionId`/`conditionId` are
// lowercase hex (32 bytes → 64 chars); only the first `numOutcomes` payout
// numerators are meaningful.
module Market = {
  type t = {
    marketId: bigint,
    numOutcomes: float,
    status: MarketStatus.t,
    bump: float,
    makerFeeBps: float,
    takerFeeBps: float,
    oracle: Shared.pubkeyStr,
    questionId: string,
    conditionId: string,
    payoutNumerators: array<float>,
    payoutDenominator: float,
  }

  let decode = (bytes: Uint8Array.t): result<t, SdkError.t> =>
    guard("Market", () =>
      switch checkHeader(bytes, Size.market, Constants.Discriminator.market, "Market") {
      | Error(error) => Error(error)
      | Ok() =>
        switch MarketStatus.fromU8(u8At(bytes, 17)) {
        | Error(error) => Error(error)
        | Ok(status) =>
          Ok({
            marketId: u64At(bytes, 8),
            numOutcomes: Int.toFloat(u8At(bytes, 16)),
            status,
            bump: Int.toFloat(u8At(bytes, 18)),
            makerFeeBps: i16At(bytes, 20),
            takerFeeBps: i16At(bytes, 22),
            oracle: addressAt(bytes, 24),
            questionId: hexAt(bytes, 56, 32),
            conditionId: hexAt(bytes, 88, 32),
            // [u32; 6] at offset 120, stride 4 (only first numOutcomes meaningful).
            payoutNumerators: [
              u32At(bytes, 120),
              u32At(bytes, 124),
              u32At(bytes, 128),
              u32At(bytes, 132),
              u32At(bytes, 136),
              u32At(bytes, 140),
            ]->Array.map(Int.toFloat),
            payoutDenominator: Int.toFloat(u32At(bytes, 144)),
          })
        }
      }
    )
}

// Orderbook — on-chain book with its ALT (144 bytes). `baseIndex` 0 ⇒ mintA is
// the base asset, 1 ⇒ mintB.
module Orderbook = {
  type t = {
    market: Shared.pubkeyStr,
    mintA: Shared.pubkeyStr,
    mintB: Shared.pubkeyStr,
    lookupTable: Shared.pubkeyStr,
    baseIndex: float,
    bump: float,
  }

  let decode = (bytes: Uint8Array.t): result<t, SdkError.t> =>
    guard("Orderbook", () =>
      switch checkHeader(bytes, Size.orderbook, Constants.Discriminator.orderbook, "Orderbook") {
      | Error(error) => Error(error)
      | Ok() =>
        Ok({
          market: addressAt(bytes, 8),
          mintA: addressAt(bytes, 40),
          mintB: addressAt(bytes, 72),
          lookupTable: addressAt(bytes, 104),
          baseIndex: Int.toFloat(u8At(bytes, 136)),
          bump: Int.toFloat(u8At(bytes, 137)),
        })
      }
    )
}

// Position — a user's per-market custody account (80 bytes).
module Position = {
  type t = {
    owner: Shared.pubkeyStr,
    market: Shared.pubkeyStr,
    bump: float,
  }

  let decode = (bytes: Uint8Array.t): result<t, SdkError.t> =>
    guard("Position", () =>
      switch checkHeader(bytes, Size.position, Constants.Discriminator.position, "Position") {
      | Error(error) => Error(error)
      | Ok() =>
        Ok({
          owner: addressAt(bytes, 8),
          market: addressAt(bytes, 40),
          bump: Int.toFloat(u8At(bytes, 72)),
        })
      }
    )
}

// UserNonce — a user's mass-cancel nonce (16 bytes). `nonce` is a u64 counter,
// kept exact as a bigint.
module UserNonce = {
  type t = {
    nonce: bigint,
  }

  let decode = (bytes: Uint8Array.t): result<t, SdkError.t> =>
    guard("UserNonce", () =>
      switch checkHeader(bytes, Size.userNonce, Constants.Discriminator.userNonce, "UserNonce") {
      | Error(error) => Error(error)
      | Ok() => Ok({nonce: u64At(bytes, 8)})
      }
    )
}

// OrderStatus — on-chain order status (remaining fill amounts + cancellation flag).
module OrderStatus = {
  type t = {
    remaining: bigint,
    baseRemaining: bigint,
    isCancelled: bool,
  }

  let decode = (bytes: Uint8Array.t): result<t, SdkError.t> =>
    guard("OrderStatus", () =>
      switch checkHeader(bytes, Size.orderStatus, Constants.Discriminator.orderStatus, "OrderStatus") {
      | Error(error) => Error(error)
      | Ok() =>
        Ok({
          remaining: u64At(bytes, 8),
          baseRemaining: u64At(bytes, 16),
          isCancelled: boolAt(bytes, 24),
        })
      }
    )
}

// GlobalDepositToken — a whitelisted global-deposit token (mint + whitelist
// index + active flag).
module GlobalDepositToken = {
  type t = {
    mint: Shared.pubkeyStr,
    bump: float,
    index: float,
    active: bool,
  }

  let decode = (bytes: Uint8Array.t): result<t, SdkError.t> =>
    guard("GlobalDepositToken", () =>
      switch checkHeader(
        bytes,
        Size.globalDepositToken,
        Constants.Discriminator.globalDepositToken,
        "GlobalDepositToken",
      ) {
      | Error(error) => Error(error)
      | Ok() =>
        Ok({
          mint: addressAt(bytes, 8),
          bump: Int.toFloat(u8At(bytes, 40)),
          index: Int.toFloat(u16At(bytes, 41)),
          active: boolAt(bytes, 43),
        })
      }
    )
}
