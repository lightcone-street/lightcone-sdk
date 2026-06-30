// @solana/kit (modular web3.js v2) — the Solana/crypto layer of the SDK. The
// surface is split across this package by concern: this main module holds the
// shared TYPES + base58 addresses, and the sibling modules add the rest —
// `SolanaKit.Codec` (byte codecs), `SolanaKit.Keys` (ed25519 keypairs + signing),
// `SolanaKit.Pda` (program-derived addresses), `SolanaKit.Tx` (transactions),
// `SolanaKit.Rpc`. The one primitive kit lacks — keccak256 — lives in `NobleHashes`.
//
// All keypair/sign/PDA operations are async (kit uses WebCrypto; native ed25519 on
// Node >=18.4 / Bun >=1.2.6 — no polyfill needed).

// ── Addresses ────────────────────────────────────────────────────────────────
// A base58 Solana address. `address()` validates and brands the string; it throws
// on an invalid address (callers at the SDK layer catch into `result`).
type address

@module("@solana/kit") external address: string => address = "address"
@module("@solana/kit") external isAddress: string => bool = "isAddress"
// An address IS a string at runtime — coerce back without a runtime call.
external addressToString: address => string = "%identity"

// ── Shared codec types ───────────────────────────────────────────────────────
// kit codecs are objects exposing `.encode(value) => bytes` and `.decode(bytes)
// => value` (see the `Codec` module). u64/i64 carry their value as a bigint.
type encoder<'a>
type decoder<'a>
type codec<'a>

// ── Shared key types ─────────────────────────────────────────────────────────
// CryptoKeyPair is a WebCrypto pair; a signer wraps a keypair + its `.address`
// (used as a transaction fee payer). See the `Keys` module for operations.
type cryptoKey
type cryptoKeyPair = {privateKey: cryptoKey, publicKey: cryptoKey}
type keyPairSigner

// ── Shared PDA + instruction types ───────────────────────────────────────────
type pdaSeedsInput = {programAddress: address, seeds: array<Uint8Array.t>}

// kit AccountRole is an int enum: 0 READONLY, 1 WRITABLE, 2 READONLY_SIGNER,
// 3 WRITABLE_SIGNER.
type accountMeta = {address: address, role: int}
type instruction = {programAddress: address, accounts: array<accountMeta>, data: Uint8Array.t}

module Role = {
  let readonly = 0
  let writable = 1
  let readonlySigner = 2
  let writableSigner = 3
}
