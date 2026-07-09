// Binding to @noble/hashes — keccak256, the one hash primitive @solana/kit lacks.
//
// The Lightcone program hashes the 169-byte order message with Keccak256 (sha3
// crate in the Rust SDK), then hex-encodes and ed25519-signs the hex string. We
// only need the one-shot `keccak_256` function. In @noble/hashes v2 the sha3
// family ships from "@noble/hashes/sha3.js"; `keccak_256` is a callable CHash:
// `keccak_256(bytes) => 32-byte digest`.

@module("@noble/hashes/sha3.js")
external keccak256: Uint8Array.t => Uint8Array.t = "keccak_256"
