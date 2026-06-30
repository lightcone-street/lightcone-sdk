// Program-derived addresses — mirrors rust/src/program/pda.rs exactly (same seed
// byte strings, same order, canonical mint sorting). All derivations are async
// (kit's getProgramDerivedAddress hashes with SHA-256 via WebCrypto) and return
// `(address, bump)`.

let seedBytes = text => SolanaKitCodec.encode(SolanaKitCodec.getUtf8Encoder(), text)
let addressBytes = address => SolanaKitCodec.encode(SolanaKitCodec.getAddressEncoder(), address)
let u64le = (value: bigint) => SolanaKitCodec.encode(SolanaKitCodec.getU64Encoder(), value)
let u8byte = (value: int) => SolanaKitCodec.encode(SolanaKitCodec.getU8Encoder(), value)

let derive = (programId: SolanaKit.address, seeds: array<Uint8Array.t>): promise<(SolanaKit.address, int)> =>
  SolanaKitPda.getProgramDerivedAddress({programAddress: programId, seeds})

// Lexicographic byte-array comparison (a <= b), matching Rust's `mint.as_ref()` ordering.
let bytesLte: (Uint8Array.t, Uint8Array.t) => bool = %raw(`function (a, b) {
  for (let i = 0; i < a.length; i++) { if (a[i] !== b[i]) return a[i] < b[i]; }
  return true;
}`)

// Seeds: ["central_state"]
let exchange = (programId): promise<(SolanaKit.address, int)> => derive(programId, [seedBytes("central_state")])

// Seeds: ["market", market_id u64 LE]
let market = (programId, ~marketId: bigint) => derive(programId, [seedBytes("market"), u64le(marketId)])

// Seeds: ["user_nonce", user]
let userNonce = (programId, ~user) => derive(programId, [seedBytes("user_nonce"), addressBytes(user)])

// Seeds: ["position", owner, market]
let position = (programId, ~owner, ~market) =>
  derive(programId, [seedBytes("position"), addressBytes(owner), addressBytes(market)])

// Seeds: ["market_deposit_token_account", deposit_mint, market]
let vault = (programId, ~depositMint, ~market) =>
  derive(programId, [seedBytes("market_deposit_token_account"), addressBytes(depositMint), addressBytes(market)])

// Seeds: ["market_mint_authority", market]
let mintAuthority = (programId, ~market) =>
  derive(programId, [seedBytes("market_mint_authority"), addressBytes(market)])

// Seeds: ["conditional_mint", market, deposit_mint, outcome_index u8]
let conditionalMint = (programId, ~market, ~depositMint, ~outcomeIndex) =>
  derive(
    programId,
    [seedBytes("conditional_mint"), addressBytes(market), addressBytes(depositMint), u8byte(outcomeIndex)],
  )

// Seeds: ["condition", condition_id (32)]
let condition = (programId, ~conditionId: Uint8Array.t) =>
  derive(programId, [seedBytes("condition"), conditionId])

// Seeds: ["order_status", order_hash (32)]
let orderStatus = (programId, ~orderHash: Uint8Array.t) =>
  derive(programId, [seedBytes("order_status"), orderHash])

// Seeds: ["global_deposit", mint]  (whitelist entry)
let globalDepositToken = (programId, ~mint) =>
  derive(programId, [seedBytes("global_deposit"), addressBytes(mint)])

// Seeds: ["global_deposit", user, mint]  (user's global token account)
let userGlobalDeposit = (programId, ~user, ~mint) =>
  derive(programId, [seedBytes("global_deposit"), addressBytes(user), addressBytes(mint)])

// Seeds: ["orderbook", canonical_mint_a, canonical_mint_b]
let orderbook = (programId, ~mintA, ~mintB) => {
  let bytesA = addressBytes(mintA)
  let bytesB = addressBytes(mintB)
  let (first, second) = bytesLte(bytesA, bytesB) ? (bytesA, bytesB) : (bytesB, bytesA)
  derive(programId, [seedBytes("orderbook"), first, second])
}

// Seeds: ["metadata", MPL_PROGRAM, conditional_mint] @ MPL metadata program
let mplMetadata = (~conditionalMint) =>
  derive(
    Constants.mplTokenMetadataProgram,
    [seedBytes("metadata"), addressBytes(Constants.mplTokenMetadataProgram), addressBytes(conditionalMint)],
  )

// Seeds: [orderbook, slot u64 LE] @ ALT program
let alt = (~orderbook, ~recentSlot: bigint) =>
  derive(Constants.altProgram, [addressBytes(orderbook), u64le(recentSlot)])

// Seeds: [position, slot u64 LE] @ ALT program
let positionAlt = (~position, ~recentSlot: bigint) =>
  derive(Constants.altProgram, [addressBytes(position), u64le(recentSlot)])
