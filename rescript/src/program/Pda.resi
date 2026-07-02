// Program-derived addresses — the named PDA derivations from rust/src/program/pda.rs.
// Each is async (kit hashes with SHA-256 via WebCrypto) and returns `(address, bump)`.
// The seed/byte-encoding helpers and the generic `derive` are internal.

let exchange: SolanaKit.address => promise<(SolanaKit.address, int)>

let market: (SolanaKit.address, ~marketId: bigint) => promise<(SolanaKit.address, int)>

let userNonce: (SolanaKit.address, ~user: SolanaKit.address) => promise<(SolanaKit.address, int)>

let position: (
  SolanaKit.address,
  ~owner: SolanaKit.address,
  ~market: SolanaKit.address,
) => promise<(SolanaKit.address, int)>

let vault: (
  SolanaKit.address,
  ~depositMint: SolanaKit.address,
  ~market: SolanaKit.address,
) => promise<(SolanaKit.address, int)>

let mintAuthority: (
  SolanaKit.address,
  ~market: SolanaKit.address,
) => promise<(SolanaKit.address, int)>

let conditionalMint: (
  SolanaKit.address,
  ~market: SolanaKit.address,
  ~depositMint: SolanaKit.address,
  ~outcomeIndex: int,
) => promise<(SolanaKit.address, int)>

let condition: (
  SolanaKit.address,
  ~conditionId: Uint8Array.t,
) => promise<(SolanaKit.address, int)>

let orderStatus: (
  SolanaKit.address,
  ~orderHash: Uint8Array.t,
) => promise<(SolanaKit.address, int)>

let globalDepositToken: (
  SolanaKit.address,
  ~mint: SolanaKit.address,
) => promise<(SolanaKit.address, int)>

let userGlobalDeposit: (
  SolanaKit.address,
  ~user: SolanaKit.address,
  ~mint: SolanaKit.address,
) => promise<(SolanaKit.address, int)>

let orderbook: (
  SolanaKit.address,
  ~mintA: SolanaKit.address,
  ~mintB: SolanaKit.address,
) => promise<(SolanaKit.address, int)>

// @ MPL metadata program.
let mplMetadata: (~conditionalMint: SolanaKit.address) => promise<(SolanaKit.address, int)>

// @ ALT program.
let alt: (~orderbook: SolanaKit.address, ~recentSlot: bigint) => promise<(SolanaKit.address, int)>

// @ ALT program.
let positionAlt: (
  ~position: SolanaKit.address,
  ~recentSlot: bigint,
) => promise<(SolanaKit.address, int)>

// Every outcome's conditional mint for (market, deposit_mint), in outcome order.
let allConditionalMints: (
  SolanaKit.address,
  ~market: SolanaKit.address,
  ~depositMint: SolanaKit.address,
  ~numOutcomes: int,
) => promise<array<SolanaKit.address>>
