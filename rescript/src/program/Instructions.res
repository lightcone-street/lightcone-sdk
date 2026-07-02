// Instruction builders for the Lightcone Pinocchio program. Mirrors
// rust/src/program/instructions.rs EXACTLY — same ordered account list and the
// same manual byte-packed `data` per instruction.
//
// This is a Pinocchio program, NOT Anchor/borsh: `data` is a SINGLE-byte opcode
// at data[0] followed by little-endian scalar fields (and raw 32-byte pubkeys
// where applicable). Account `role` mirrors Rust's helper set:
//   signer_mut → WRITABLE_SIGNER (3)   writable → WRITABLE (1)
//   signer     → READONLY_SIGNER (2)   readonly → READONLY (0)
//
// Every builder is async because PDA + ATA derivation hashes via WebCrypto.

// ── data packing (reuse OrderPayload's concat + kit's LE codecs) ──────────────
let concatBytes = OrderPayload.concatBytes
let u8 = (value: int) => SolanaKitCodec.encode(SolanaKitCodec.getU8Encoder(), value)
let u64 = (value: bigint) => SolanaKitCodec.encode(SolanaKitCodec.getU64Encoder(), value)

// ── account-meta helpers (mirror the Rust signer_mut/writable/readonly) ───────
let meta = (address, role): SolanaKit.accountMeta => {address, role}
let signerMut = address => meta(address, SolanaKit.Role.writableSigner)
let writable = address => meta(address, SolanaKit.Role.writable)
let readonly = address => meta(address, SolanaKit.Role.readonly)

// Associated-token-account derivation. Both deposit-token and conditional-token
// ATAs use the SPL Token program (matches Rust's get_deposit_token_ata /
// get_conditional_token_ata, which both pass spl_token::id()).
let ata = (~owner, ~mint): promise<(SolanaKit.address, int)> =>
  SolanaProgramToken.findAssociatedTokenPda({
    owner,
    tokenProgram: Constants.tokenProgram,
    mint,
  })

// ── DepositToGlobal (opcode 17) ───────────────────────────────────────────────
// Rust: build_deposit_to_global_ix. Accounts (8):
// 0 user(signer,mut) 1 global_deposit_token 2 mint 3 user_global_deposit(mut)
// 4 user_token_account(mut) 5 token_program 6 system_program 7 exchange
// Data: [17, amount u64 LE]
let depositToGlobal = async (
  ~programId: SolanaKit.address,
  ~user: SolanaKit.address,
  ~mint: SolanaKit.address,
  ~amount: bigint,
): SolanaKit.instruction => {
  let (globalDepositToken, _) = await Pda.globalDepositToken(programId, ~mint)
  let (userGlobalDeposit, _) = await Pda.userGlobalDeposit(programId, ~user, ~mint)
  let (exchange, _) = await Pda.exchange(programId)
  let (userTokenAccount, _) = await ata(~owner=user, ~mint)
  {
    programAddress: programId,
    accounts: [
      signerMut(user),
      readonly(globalDepositToken),
      readonly(mint),
      writable(userGlobalDeposit),
      writable(userTokenAccount),
      readonly(Constants.tokenProgram),
      readonly(Constants.systemProgram),
      readonly(exchange),
    ],
    data: concatBytes([u8(Constants.Instruction.depositToGlobal), u64(amount)]),
  }
}

// ── WithdrawFromGlobal (opcode 22) ────────────────────────────────────────────
// Rust: build_withdraw_from_global_ix. Accounts (7 — NOTE: no system_program):
// 0 user(signer,mut) 1 global_deposit_token 2 mint 3 user_global_deposit(mut)
// 4 user_token_account(mut) 5 token_program 6 exchange
// Data: [22, amount u64 LE]
let withdrawFromGlobal = async (
  ~programId: SolanaKit.address,
  ~user: SolanaKit.address,
  ~mint: SolanaKit.address,
  ~amount: bigint,
): SolanaKit.instruction => {
  let (globalDepositToken, _) = await Pda.globalDepositToken(programId, ~mint)
  let (userGlobalDeposit, _) = await Pda.userGlobalDeposit(programId, ~user, ~mint)
  let (exchange, _) = await Pda.exchange(programId)
  let (userTokenAccount, _) = await ata(~owner=user, ~mint)
  {
    programAddress: programId,
    accounts: [
      signerMut(user),
      readonly(globalDepositToken),
      readonly(mint),
      writable(userGlobalDeposit),
      writable(userTokenAccount),
      readonly(Constants.tokenProgram),
      readonly(exchange),
    ],
    data: concatBytes([u8(Constants.Instruction.withdrawFromGlobal), u64(amount)]),
  }
}

// ── GlobalToMarketDeposit (opcode 18) ─────────────────────────────────────────
// Rust: build_global_to_market_deposit_ix. Accounts (12 + numOutcomes*2):
// 0 user(signer,mut) 1 exchange 2 market 3 deposit_mint 4 vault(mut)
// 5 global_deposit_token 6 user_global_deposit(mut) 7 position(mut)
// 8 mint_authority 9 token_program 10 ata_program 11 system_program
// + per outcome i: conditional_mint[i](mut), position_conditional_ata[i](mut)
// Data: [18, amount u64 LE]
let globalToMarketDeposit = async (
  ~programId: SolanaKit.address,
  ~user: SolanaKit.address,
  ~market: SolanaKit.address,
  ~mint: SolanaKit.address,
  ~amount: bigint,
  ~numOutcomes: int,
): SolanaKit.instruction => {
  let (exchange, _) = await Pda.exchange(programId)
  let (vault, _) = await Pda.vault(programId, ~depositMint=mint, ~market)
  let (globalDepositToken, _) = await Pda.globalDepositToken(programId, ~mint)
  let (userGlobalDeposit, _) = await Pda.userGlobalDeposit(programId, ~user, ~mint)
  let (position, _) = await Pda.position(programId, ~owner=user, ~market)
  let (mintAuthority, _) = await Pda.mintAuthority(programId, ~market)

  let accounts = [
    signerMut(user),
    readonly(exchange),
    readonly(market),
    readonly(mint),
    writable(vault),
    readonly(globalDepositToken),
    writable(userGlobalDeposit),
    writable(position),
    readonly(mintAuthority),
    readonly(Constants.tokenProgram),
    readonly(Constants.associatedTokenProgram),
    readonly(Constants.systemProgram),
  ]
  for outcomeIndex in 0 to numOutcomes - 1 {
    let (conditionalMint, _) =
      await Pda.conditionalMint(programId, ~market, ~depositMint=mint, ~outcomeIndex)
    let (positionAta, _) = await ata(~owner=position, ~mint=conditionalMint)
    accounts->Array.push(writable(conditionalMint))
    accounts->Array.push(writable(positionAta))
  }
  {
    programAddress: programId,
    accounts,
    data: concatBytes([u8(Constants.Instruction.globalToMarketDeposit), u64(amount)]),
  }
}

// ── MergeCompleteSet (opcode 4) ───────────────────────────────────────────────
// Rust: build_merge_ix. Accounts (9 + numOutcomes*2):
// 0 user(signer,mut) 1 exchange 2 market 3 deposit_mint 4 vault(mut)
// 5 position(mut) 6 user_deposit_ata(mut) 7 mint_authority 8 token_program
// + per outcome i: conditional_mint[i](mut), position_conditional_ata[i](mut)
// Data: [4, amount u64 LE]
let mergeCompleteSet = async (
  ~programId: SolanaKit.address,
  ~user: SolanaKit.address,
  ~market: SolanaKit.address,
  ~mint: SolanaKit.address,
  ~amount: bigint,
  ~numOutcomes: int,
): SolanaKit.instruction => {
  let (exchange, _) = await Pda.exchange(programId)
  let (vault, _) = await Pda.vault(programId, ~depositMint=mint, ~market)
  let (mintAuthority, _) = await Pda.mintAuthority(programId, ~market)
  let (position, _) = await Pda.position(programId, ~owner=user, ~market)
  let (userDepositAta, _) = await ata(~owner=user, ~mint)

  let accounts = [
    signerMut(user),
    readonly(exchange),
    readonly(market),
    readonly(mint),
    writable(vault),
    writable(position),
    writable(userDepositAta),
    readonly(mintAuthority),
    readonly(Constants.tokenProgram),
  ]
  for outcomeIndex in 0 to numOutcomes - 1 {
    let (conditionalMint, _) =
      await Pda.conditionalMint(programId, ~market, ~depositMint=mint, ~outcomeIndex)
    let (positionAta, _) = await ata(~owner=position, ~mint=conditionalMint)
    accounts->Array.push(writable(conditionalMint))
    accounts->Array.push(writable(positionAta))
  }
  {
    programAddress: programId,
    accounts,
    data: concatBytes([u8(Constants.Instruction.mergeCompleteSet), u64(amount)]),
  }
}

// ── RedeemWinnings (opcode 8) ─────────────────────────────────────────────────
// Rust: build_redeem_winnings_ix. Accounts (11):
// 0 user(signer,mut) 1 market 2 deposit_mint 3 vault(mut) 4 conditional_mint(mut)
// 5 position 6 position_conditional_ata(mut) 7 user_deposit_ata(mut)
// 8 mint_authority 9 token_program 10 exchange
// Data: [8, amount u64 LE, outcome_index u8]
let redeemWinnings = async (
  ~programId: SolanaKit.address,
  ~user: SolanaKit.address,
  ~market: SolanaKit.address,
  ~mint: SolanaKit.address,
  ~amount: bigint,
  ~outcomeIndex: int,
): SolanaKit.instruction => {
  let (exchange, _) = await Pda.exchange(programId)
  let (vault, _) = await Pda.vault(programId, ~depositMint=mint, ~market)
  let (mintAuthority, _) = await Pda.mintAuthority(programId, ~market)
  let (position, _) = await Pda.position(programId, ~owner=user, ~market)
  let (conditionalMint, _) =
    await Pda.conditionalMint(programId, ~market, ~depositMint=mint, ~outcomeIndex)
  let (positionConditionalAta, _) = await ata(~owner=position, ~mint=conditionalMint)
  let (userDepositAta, _) = await ata(~owner=user, ~mint)
  {
    programAddress: programId,
    accounts: [
      signerMut(user),
      readonly(market),
      readonly(mint),
      writable(vault),
      writable(conditionalMint),
      readonly(position),
      writable(positionConditionalAta),
      writable(userDepositAta),
      readonly(mintAuthority),
      readonly(Constants.tokenProgram),
      readonly(exchange),
    ],
    data: concatBytes([
      u8(Constants.Instruction.redeemWinnings),
      u64(amount),
      u8(outcomeIndex),
    ]),
  }
}

// ── InitPositionTokens (opcode 19) ────────────────────────────────────────────
// Rust: build_init_position_tokens_ix. Permissionless (payer != user allowed).
// Accounts (11 + per deposit_mint: 3 + numOutcomes*2):
// 0 payer(signer,mut) 1 user 2 exchange 3 market 4 position(mut)
// 5 lookup_table(mut) 6 mint_authority 7 token_program 8 ata_program
// 9 alt_program 10 system_program
// + per deposit_mint: deposit_mint, vault, global_deposit_token,
//   then per outcome i: conditional_mint[i], position_conditional_ata[i](mut)
// Data: [19, recent_slot u64 LE, deposit_mints.len u8]
let initPositionTokens = async (
  ~programId: SolanaKit.address,
  ~payer: SolanaKit.address,
  ~user: SolanaKit.address,
  ~market: SolanaKit.address,
  ~depositMints: array<SolanaKit.address>,
  ~recentSlot: bigint,
  ~numOutcomes: int,
): SolanaKit.instruction => {
  let (exchange, _) = await Pda.exchange(programId)
  let (position, _) = await Pda.position(programId, ~owner=user, ~market)
  let (lookupTable, _) = await Pda.positionAlt(~position, ~recentSlot)
  let (mintAuthority, _) = await Pda.mintAuthority(programId, ~market)

  let accounts = [
    signerMut(payer),
    readonly(user),
    readonly(exchange),
    readonly(market),
    writable(position),
    writable(lookupTable),
    readonly(mintAuthority),
    readonly(Constants.tokenProgram),
    readonly(Constants.associatedTokenProgram),
    readonly(Constants.altProgram),
    readonly(Constants.systemProgram),
  ]
  for mintIdx in 0 to Array.length(depositMints) - 1 {
    let depositMint = depositMints->Array.getUnsafe(mintIdx)
    let (vault, _) = await Pda.vault(programId, ~depositMint, ~market)
    let (globalDepositToken, _) = await Pda.globalDepositToken(programId, ~mint=depositMint)
    accounts->Array.push(readonly(depositMint))
    accounts->Array.push(readonly(vault))
    accounts->Array.push(readonly(globalDepositToken))
    for outcomeIndex in 0 to numOutcomes - 1 {
      let (conditionalMint, _) =
        await Pda.conditionalMint(programId, ~market, ~depositMint, ~outcomeIndex)
      let (positionAta, _) = await ata(~owner=position, ~mint=conditionalMint)
      accounts->Array.push(readonly(conditionalMint))
      accounts->Array.push(writable(positionAta))
    }
  }
  {
    programAddress: programId,
    accounts,
    data: concatBytes([
      u8(Constants.Instruction.initPositionTokens),
      u64(recentSlot),
      u8(Array.length(depositMints)),
    ]),
  }
}

// ── Deposit / MintCompleteSet (opcode 3) ──────────────────────────────────────
// Rust: build_deposit_ix. Market-level direct deposit: collateral moves from the
// user's wallet ATA into the market vault, minting a complete set of conditional
// tokens into the position. Accounts (11 + numOutcomes*2):
// 0 user(signer,mut) 1 exchange 2 market 3 deposit_mint 4 vault(mut)
// 5 user_deposit_ata(mut) 6 position(mut) 7 mint_authority 8 token_program
// 9 ata_program 10 system_program
// + per outcome i: conditional_mint[i](mut), position_conditional_ata[i](mut)
// Data: [3, amount u64 LE]
let deposit = async (
  ~programId: SolanaKit.address,
  ~user: SolanaKit.address,
  ~market: SolanaKit.address,
  ~mint: SolanaKit.address,
  ~amount: bigint,
  ~numOutcomes: int,
): SolanaKit.instruction => {
  let (exchange, _) = await Pda.exchange(programId)
  let (vault, _) = await Pda.vault(programId, ~depositMint=mint, ~market)
  let (mintAuthority, _) = await Pda.mintAuthority(programId, ~market)
  let (position, _) = await Pda.position(programId, ~owner=user, ~market)
  let (userDepositAta, _) = await ata(~owner=user, ~mint)

  let accounts = [
    signerMut(user),
    readonly(exchange),
    readonly(market),
    readonly(mint),
    writable(vault),
    writable(userDepositAta),
    writable(position),
    readonly(mintAuthority),
    readonly(Constants.tokenProgram),
    readonly(Constants.associatedTokenProgram),
    readonly(Constants.systemProgram),
  ]
  for outcomeIndex in 0 to numOutcomes - 1 {
    let (conditionalMint, _) =
      await Pda.conditionalMint(programId, ~market, ~depositMint=mint, ~outcomeIndex)
    let (positionAta, _) = await ata(~owner=position, ~mint=conditionalMint)
    accounts->Array.push(writable(conditionalMint))
    accounts->Array.push(writable(positionAta))
  }
  {
    programAddress: programId,
    accounts,
    data: concatBytes([u8(Constants.Instruction.mintCompleteSet), u64(amount)]),
  }
}

// ── WithdrawFromPosition (opcode 11) ──────────────────────────────────────────
// Rust: build_withdraw_from_position_ix. Moves conditional tokens from the
// position's ATA to the user's own ATA (`~mint` is the conditional mint).
// Accounts (8):
// 0 user(signer,mut) 1 market 2 position(mut) 3 mint 4 position_ata(mut)
// 5 user_ata(mut) 6 token_program 7 exchange
// Data: [11, amount u64 LE, outcome_index u8]
let withdrawFromPosition = async (
  ~programId: SolanaKit.address,
  ~user: SolanaKit.address,
  ~market: SolanaKit.address,
  ~mint: SolanaKit.address,
  ~amount: bigint,
  ~outcomeIndex: int,
): SolanaKit.instruction => {
  let (exchange, _) = await Pda.exchange(programId)
  let (position, _) = await Pda.position(programId, ~owner=user, ~market)
  let (positionAta, _) = await ata(~owner=position, ~mint)
  let (userAta, _) = await ata(~owner=user, ~mint)
  {
    programAddress: programId,
    accounts: [
      signerMut(user),
      readonly(market),
      writable(position),
      readonly(mint),
      writable(positionAta),
      writable(userAta),
      readonly(Constants.tokenProgram),
      readonly(exchange),
    ],
    data: concatBytes([
      u8(Constants.Instruction.withdrawFromPosition),
      u64(amount),
      u8(outcomeIndex),
    ]),
  }
}

// ── IncrementNonce (opcode 6) ─────────────────────────────────────────────────
// Rust: build_increment_nonce_ix. Accounts (4):
// 0 user(signer,mut) 1 user_nonce(mut) 2 system_program 3 exchange
// Data: [6]  (opcode only)
let incrementNonce = async (
  ~programId: SolanaKit.address,
  ~user: SolanaKit.address,
): SolanaKit.instruction => {
  let (userNonce, _) = await Pda.userNonce(programId, ~user)
  let (exchange, _) = await Pda.exchange(programId)
  {
    programAddress: programId,
    accounts: [
      signerMut(user),
      writable(userNonce),
      readonly(Constants.systemProgram),
      readonly(exchange),
    ],
    data: concatBytes([u8(Constants.Instruction.incrementNonce)]),
  }
}

// ── ExtendPositionTokens (opcode 21) ──────────────────────────────────────────
// Rust: build_extend_position_tokens_ix. Operator-signed: appends the ATAs for
// newly-added deposit mints to an existing position ALT (from initPositionTokens).
// `depositMints` must be non-empty (the Rust builder rejects an empty extension).
// Accounts (10 + per deposit_mint: 3 + numOutcomes*2):
// 0 operator(signer,mut) 1 user 2 exchange 3 market 4 position 5 lookup_table(mut)
// 6 token_program 7 ata_program 8 alt_program 9 system_program
// + per deposit_mint: deposit_mint, vault, global_deposit_token,
//   then per outcome i: conditional_mint[i], position_conditional_ata[i](mut)
// Data: [21, deposit_mints.len u8]
let extendPositionTokens = async (
  ~programId: SolanaKit.address,
  ~operator: SolanaKit.address,
  ~user: SolanaKit.address,
  ~market: SolanaKit.address,
  ~lookupTable: SolanaKit.address,
  ~depositMints: array<SolanaKit.address>,
  ~numOutcomes: int,
): SolanaKit.instruction => {
  let (exchange, _) = await Pda.exchange(programId)
  let (position, _) = await Pda.position(programId, ~owner=user, ~market)

  let accounts = [
    signerMut(operator),
    readonly(user),
    readonly(exchange),
    readonly(market),
    readonly(position),
    writable(lookupTable),
    readonly(Constants.tokenProgram),
    readonly(Constants.associatedTokenProgram),
    readonly(Constants.altProgram),
    readonly(Constants.systemProgram),
  ]
  for mintIdx in 0 to Array.length(depositMints) - 1 {
    let depositMint = depositMints->Array.getUnsafe(mintIdx)
    let (vault, _) = await Pda.vault(programId, ~depositMint, ~market)
    let (globalDepositToken, _) = await Pda.globalDepositToken(programId, ~mint=depositMint)
    accounts->Array.push(readonly(depositMint))
    accounts->Array.push(readonly(vault))
    accounts->Array.push(readonly(globalDepositToken))
    for outcomeIndex in 0 to numOutcomes - 1 {
      let (conditionalMint, _) =
        await Pda.conditionalMint(programId, ~market, ~depositMint, ~outcomeIndex)
      let (positionAta, _) = await ata(~owner=position, ~mint=conditionalMint)
      accounts->Array.push(readonly(conditionalMint))
      accounts->Array.push(writable(positionAta))
    }
  }
  {
    programAddress: programId,
    accounts,
    data: concatBytes([
      u8(Constants.Instruction.extendPositionTokens),
      u8(Array.length(depositMints)),
    ]),
  }
}

// ── ClosePositionAlt (opcode 23) ──────────────────────────────────────────────
// Rust: build_close_position_alt_ix. Operator-signed: deactivates an active
// position ALT, or closes an already-deactivated one. `~position` is the position
// PDA address itself (passed through, not derived). Accounts (6):
// 0 operator(signer,mut) 1 exchange 2 position 3 market 4 lookup_table(mut)
// 5 alt_program
// Data: [23]  (opcode only)
let closePositionAlt = async (
  ~programId: SolanaKit.address,
  ~operator: SolanaKit.address,
  ~position: SolanaKit.address,
  ~market: SolanaKit.address,
  ~lookupTable: SolanaKit.address,
): SolanaKit.instruction => {
  let (exchange, _) = await Pda.exchange(programId)
  {
    programAddress: programId,
    accounts: [
      signerMut(operator),
      readonly(exchange),
      readonly(position),
      readonly(market),
      writable(lookupTable),
      readonly(Constants.altProgram),
    ],
    data: concatBytes([u8(Constants.Instruction.closePositionAlt)]),
  }
}

// ── ClosePositionTokenAccounts (opcode 25) ────────────────────────────────────
// Rust: build_close_position_token_accounts_ix. Operator-signed: closes empty SPL
// conditional ATAs owned by a position PDA after market resolution (non-empty
// accounts are skipped by the program). `~position` is the position PDA address
// itself; `depositMints` must be non-empty and `numOutcomes` within 2..=6 (the
// Rust builder validates both). Accounts (5 + per deposit_mint: 1 + numOutcomes*2):
// 0 operator(signer,mut) 1 exchange 2 market 3 position 4 token_program
// + per deposit_mint: deposit_mint,
//   then per outcome i: conditional_mint[i], position_conditional_ata[i](mut)
// Data: [25]  (opcode only)
let closePositionTokenAccounts = async (
  ~programId: SolanaKit.address,
  ~operator: SolanaKit.address,
  ~market: SolanaKit.address,
  ~position: SolanaKit.address,
  ~depositMints: array<SolanaKit.address>,
  ~numOutcomes: int,
): SolanaKit.instruction => {
  let (exchange, _) = await Pda.exchange(programId)
  let accounts = [
    signerMut(operator),
    readonly(exchange),
    readonly(market),
    readonly(position),
    readonly(Constants.tokenProgram),
  ]
  for mintIdx in 0 to Array.length(depositMints) - 1 {
    let depositMint = depositMints->Array.getUnsafe(mintIdx)
    accounts->Array.push(readonly(depositMint))
    for outcomeIndex in 0 to numOutcomes - 1 {
      let (conditionalMint, _) =
        await Pda.conditionalMint(programId, ~market, ~depositMint, ~outcomeIndex)
      let (positionAta, _) = await ata(~owner=position, ~mint=conditionalMint)
      accounts->Array.push(readonly(conditionalMint))
      accounts->Array.push(writable(positionAta))
    }
  }
  {
    programAddress: programId,
    accounts,
    data: concatBytes([u8(Constants.Instruction.closePositionTokenAccounts)]),
  }
}

// ═══ Operator / admin / lifecycle instructions ══════════════════════════════
// The full remaining rust/src/program/instructions.rs surface. Builders whose
// Rust twin validates inputs return `result` (Validation errors); the rest stay
// total. Roles: `signer` (readonly signer) joins the helpers above for the
// oracle/role-acceptance instructions.

let signer = address => meta(address, SolanaKit.Role.readonlySigner)
let u32 = (value: int) => SolanaKitCodec.encode(SolanaKitCodec.getU32Encoder(), value)
// Little-endian i16 via two's complement into the u16 encoder.
let i16 = (value: int) =>
  SolanaKitCodec.encode(SolanaKitCodec.getU16Encoder(), value < 0 ? value + 65536 : value)
let addressBytes = address => SolanaKitCodec.encode(SolanaKitCodec.getAddressEncoder(), address)
let utf8Bytes = text => SolanaKitCodec.encode(SolanaKitCodec.getUtf8Encoder(), text)
let byteLength: Uint8Array.t => int = %raw(`(bytes) => bytes.length`)

// The all-zeros pubkey (base58 "111…111", 32 ones) used for placeholder checks.
let zeroAddress = "11111111111111111111111111111111"
let isZeroAddress = address => SolanaKit.addressToString(address) == zeroAddress

// ── Validators (program/utils.rs) ─────────────────────────────────────────────
let validateOutcomeCount = (numOutcomes: int): result<unit, SdkError.t> =>
  numOutcomes >= Constants.minOutcomes && numOutcomes <= Constants.maxOutcomes
    ? Ok()
    : Error(SdkError.Validation(`invalid outcome count: ${Int.toString(numOutcomes)}`))

let validateOutcomeIndex = (outcomeIndex: int, ~numOutcomes: int): result<unit, SdkError.t> =>
  outcomeIndex >= 0 && outcomeIndex < numOutcomes
    ? Ok()
    : Error(SdkError.Validation(`invalid outcome index: ${Int.toString(outcomeIndex)}`))

// Fees are capped at ±500 bps each and must not sum negative.
let validateFeePair = (~makerFeeBps: int, ~takerFeeBps: int): result<unit, SdkError.t> =>
  if makerFeeBps < -500 || makerFeeBps > 500 || takerFeeBps < -500 || takerFeeBps > 500 {
    Error(SdkError.Validation("fee out of range (±500 bps)"))
  } else if makerFeeBps + takerFeeBps < 0 {
    Error(SdkError.Validation("maker + taker fees must not sum negative"))
  } else {
    Ok()
  }

// u32-length-prefixed UTF-8 string (metadata serialization).
let serializeStringU32 = (text: string): Uint8Array.t => {
  let bytes = utf8Bytes(text)
  concatBytes([u32(byteLength(bytes)), bytes])
}

let validateMetadataString = (field, value, maxBytes): result<unit, SdkError.t> =>
  byteLength(utf8Bytes(value)) <= maxBytes
    ? Ok()
    : Error(SdkError.Validation(`${field} exceeds ${Int.toString(maxBytes)} bytes`))

// name(≤32) ‖ symbol(≤10) ‖ uri(≤200), each u32-length-prefixed.
let serializeConditionalMetadata = (~name, ~symbol, ~uri): result<Uint8Array.t, SdkError.t> =>
  switch (
    validateMetadataString("name", name, 32),
    validateMetadataString("symbol", symbol, 10),
    validateMetadataString("uri", uri, 200),
  ) {
  | (Ok(), Ok(), Ok()) =>
    Ok(concatBytes([serializeStringU32(name), serializeStringU32(symbol), serializeStringU32(uri)]))
  | (Error(error), _, _) | (_, Error(error), _) | (_, _, Error(error)) => Error(error)
  }

// ── Signed-order params (matching instructions) ───────────────────────────────
// The ReScript payload carries no embedded signature, so matching params pair
// each order with its 64-byte signature explicitly.
type signedOrder = {order: OrderPayload.t, signature: Uint8Array.t}

// One maker fill in a MatchOrdersMulti instruction.
type matchMaker = {
  order: signedOrder,
  makerFillAmount: bigint,
  takerFillAmount: bigint,
  // Full fills skip the maker's order-status account.
  isFullFill: bool,
}

// One maker fill in a DepositAndSwap instruction.
type swapMaker = {
  order: signedOrder,
  makerFillAmount: bigint,
  takerFillAmount: bigint,
  isFullFill: bool,
  // Deposit from global (vs swapping existing conditional tokens).
  isDeposit: bool,
  // Only read when `isDeposit` is set.
  depositMint: SolanaKit.address,
}

let signedOrderBytes = (signed: signedOrder): Uint8Array.t =>
  concatBytes([OrderPayload.Compact.serialize(OrderPayload.toOrder(signed.order)), signed.signature])

// 2^index — bitmask bits are distinct per maker, so OR collapses to addition.
let bitValue = (index: int): int => Float.toInt(2.0 ** Int.toFloat(index))

// ── Initialize (opcode 0) ─────────────────────────────────────────────────────
// Creates the exchange singleton. Accounts (3): authority(signer,mut),
// exchange(mut), system_program. Data: [0]
let initialize = async (
  ~programId: SolanaKit.address,
  ~authority: SolanaKit.address,
): SolanaKit.instruction => {
  let (exchange, _) = await Pda.exchange(programId)
  {
    programAddress: programId,
    accounts: [signerMut(authority), writable(exchange), readonly(Constants.systemProgram)],
    data: concatBytes([u8(Constants.Instruction.initialize)]),
  }
}

// ── CreateMarket (opcode 1) ───────────────────────────────────────────────────
// Manager-only. Accounts (5): manager(signer,mut) exchange(mut) market(mut)
// system_program condition_tombstone(mut).
// Data: [1, num_outcomes u8, oracle 32, question_id 32, maker_fee i16, taker_fee i16]
let createMarket = async (
  ~programId: SolanaKit.address,
  ~manager: SolanaKit.address,
  ~marketId: bigint,
  ~numOutcomes: int,
  ~oracle: SolanaKit.address,
  ~questionId: Uint8Array.t,
  ~makerFeeBps: int,
  ~takerFeeBps: int,
): result<SolanaKit.instruction, SdkError.t> =>
  switch (validateOutcomeCount(numOutcomes), validateFeePair(~makerFeeBps, ~takerFeeBps)) {
  | (Error(error), _) | (_, Error(error)) => Error(error)
  | (Ok(), Ok()) =>
    let (exchange, _) = await Pda.exchange(programId)
    let (market, _) = await Pda.market(programId, ~marketId)
    let conditionId = OrderPayload.deriveConditionId(~oracle, ~questionId, ~numOutcomes)
    let (conditionTombstone, _) = await Pda.condition(programId, ~conditionId)
    Ok({
      programAddress: programId,
      accounts: [
        signerMut(manager),
        writable(exchange),
        writable(market),
        readonly(Constants.systemProgram),
        writable(conditionTombstone),
      ],
      data: concatBytes([
        u8(Constants.Instruction.createMarket),
        u8(numOutcomes),
        addressBytes(oracle),
        questionId,
        i16(makerFeeBps),
        i16(takerFeeBps),
      ]),
    })
  }

// ── AddDepositMint (opcode 2) ─────────────────────────────────────────────────
// Manager-only: vault + conditional mints for a deposit token. Accounts
// (9 + numOutcomes): manager(signer,mut) exchange market deposit_mint vault(mut)
// mint_authority token_program system_program global_deposit_token
// + conditional_mint[i](mut). Data: [2]
let addDepositMint = async (
  ~programId: SolanaKit.address,
  ~manager: SolanaKit.address,
  ~market: SolanaKit.address,
  ~depositMint: SolanaKit.address,
  ~numOutcomes: int,
): result<SolanaKit.instruction, SdkError.t> =>
  switch validateOutcomeCount(numOutcomes) {
  | Error(error) => Error(error)
  | Ok() =>
    let (exchange, _) = await Pda.exchange(programId)
    let (vault, _) = await Pda.vault(programId, ~depositMint, ~market)
    let (mintAuthority, _) = await Pda.mintAuthority(programId, ~market)
    let (globalDepositToken, _) = await Pda.globalDepositToken(programId, ~mint=depositMint)
    let accounts = [
      signerMut(manager),
      readonly(exchange),
      readonly(market),
      readonly(depositMint),
      writable(vault),
      readonly(mintAuthority),
      readonly(Constants.tokenProgram),
      readonly(Constants.systemProgram),
      readonly(globalDepositToken),
    ]
    for outcomeIndex in 0 to numOutcomes - 1 {
      let (conditionalMint, _) = await Pda.conditionalMint(programId, ~market, ~depositMint, ~outcomeIndex)
      accounts->Array.push(writable(conditionalMint))
    }
    Ok({
      programAddress: programId,
      accounts,
      data: concatBytes([u8(Constants.Instruction.addDepositMint)]),
    })
  }

// ── CancelOrder (opcode 5) ────────────────────────────────────────────────────
// Marks an on-chain order status cancelled and closes it. Accounts (4):
// operator(signer,mut) exchange market order_status(mut).
// Data: [5, order_hash 32, signed order 233] = 266 bytes
let cancelOrder = async (
  ~programId: SolanaKit.address,
  ~operator: SolanaKit.address,
  ~market: SolanaKit.address,
  ~order: OrderPayload.t,
  ~signature: Uint8Array.t,
): SolanaKit.instruction => {
  let orderHash = OrderPayload.hash(order)
  let (exchange, _) = await Pda.exchange(programId)
  let (orderStatus, _) = await Pda.orderStatus(programId, ~orderHash)
  {
    programAddress: programId,
    accounts: [signerMut(operator), readonly(exchange), readonly(market), writable(orderStatus)],
    data: concatBytes([
      u8(Constants.Instruction.cancelOrder),
      orderHash,
      OrderPayload.serialize(order, ~signature),
    ]),
  }
}

// ── SettleMarket (opcode 7) ───────────────────────────────────────────────────
// Oracle-only resolution; the denominator is the checked sum of the numerators.
// Accounts (3): oracle(signer) exchange market(mut). Data: [7, numerators u32 LE…]
let settleMarket = async (
  ~programId: SolanaKit.address,
  ~oracle: SolanaKit.address,
  ~marketId: bigint,
  ~payoutNumerators: array<int>,
): result<SolanaKit.instruction, SdkError.t> => {
  let count = Array.length(payoutNumerators)
  if count < Constants.minOutcomes || count > Constants.maxOutcomes {
    Error(SdkError.Validation(`invalid outcome count: ${Int.toString(count)}`))
  } else if payoutNumerators->Array.some(numerator => numerator < 0) {
    Error(SdkError.Validation("payout numerators must be non-negative"))
  } else if payoutNumerators->Array.reduce(0.0, (sum, n) => sum +. Int.toFloat(n)) == 0.0 {
    Error(SdkError.Validation("payout numerators must not all be zero"))
  } else if payoutNumerators->Array.reduce(0.0, (sum, n) => sum +. Int.toFloat(n)) > 4294967295.0 {
    Error(SdkError.Validation("payout numerator sum overflows u32"))
  } else {
    let (exchange, _) = await Pda.exchange(programId)
    let (market, _) = await Pda.market(programId, ~marketId)
    Ok({
      programAddress: programId,
      accounts: [signer(oracle), readonly(exchange), writable(market)],
      data: concatBytes(
        [u8(Constants.Instruction.settleMarket)]->Array.concat(
          payoutNumerators->Array.map(numerator => u32(numerator)),
        ),
      ),
    })
  }
}

// Winner-takes-all payout numerators: 1 for the winning outcome, 0 elsewhere
// (Rust `SettleMarketParams::winner_takes_all`).
let winnerTakesAllNumerators = (~winningOutcome: int, ~numOutcomes: int): result<array<int>, SdkError.t> =>
  switch (validateOutcomeCount(numOutcomes), validateOutcomeIndex(winningOutcome, ~numOutcomes)) {
  | (Error(error), _) | (_, Error(error)) => Error(error)
  | (Ok(), Ok()) =>
    Ok(Array.fromInitializer(~length=numOutcomes, index => index == winningOutcome ? 1 : 0))
  }

// ── SetPaused (opcode 9) ──────────────────────────────────────────────────────
let setPaused = async (
  ~programId: SolanaKit.address,
  ~authority: SolanaKit.address,
  ~paused: bool,
): SolanaKit.instruction => {
  let (exchange, _) = await Pda.exchange(programId)
  {
    programAddress: programId,
    accounts: [signerMut(authority), writable(exchange)],
    data: concatBytes([u8(Constants.Instruction.setPaused), u8(paused ? 1 : 0)]),
  }
}

// ── Role management (opcodes 10, 14, 28, 35–37) ───────────────────────────────
// Propose a new role holder; the change lands when the incoming holder accepts.
let proposeRole = async (~programId, ~current, ~proposed, ~opcode): SolanaKit.instruction => {
  let (exchange, _) = await Pda.exchange(programId)
  {
    programAddress: programId,
    accounts: [signerMut(current), writable(exchange)],
    data: concatBytes([u8(opcode), addressBytes(proposed)]),
  }
}

let setOperator = (~programId: SolanaKit.address, ~authority: SolanaKit.address, ~newOperator: SolanaKit.address) =>
  proposeRole(~programId, ~current=authority, ~proposed=newOperator, ~opcode=Constants.Instruction.setOperator)

let setAuthority = (
  ~programId: SolanaKit.address,
  ~currentAuthority: SolanaKit.address,
  ~newAuthority: SolanaKit.address,
) =>
  proposeRole(
    ~programId,
    ~current=currentAuthority,
    ~proposed=newAuthority,
    ~opcode=Constants.Instruction.setAuthority,
  )

let setManager = (~programId: SolanaKit.address, ~authority: SolanaKit.address, ~newManager: SolanaKit.address) =>
  proposeRole(~programId, ~current=authority, ~proposed=newManager, ~opcode=Constants.Instruction.setManager)

// The proposed holder accepts the role. Accounts (2): incoming(signer) exchange(mut).
let acceptRole = async (~programId, ~incomingRole, ~opcode): SolanaKit.instruction => {
  let (exchange, _) = await Pda.exchange(programId)
  {
    programAddress: programId,
    accounts: [signer(incomingRole), writable(exchange)],
    data: concatBytes([u8(opcode)]),
  }
}

let acceptAuthority = (~programId: SolanaKit.address, ~incomingRole: SolanaKit.address) =>
  acceptRole(~programId, ~incomingRole, ~opcode=Constants.Instruction.acceptAuthority)
let acceptManager = (~programId: SolanaKit.address, ~incomingRole: SolanaKit.address) =>
  acceptRole(~programId, ~incomingRole, ~opcode=Constants.Instruction.acceptManager)
let acceptOperator = (~programId: SolanaKit.address, ~incomingRole: SolanaKit.address) =>
  acceptRole(~programId, ~incomingRole, ~opcode=Constants.Instruction.acceptOperator)

// ── ActivateMarket (opcode 12) ────────────────────────────────────────────────
// Manager: Pending → Active. Accounts (3): manager(signer,mut) exchange market(mut).
let activateMarket = async (
  ~programId: SolanaKit.address,
  ~manager: SolanaKit.address,
  ~marketId: bigint,
): SolanaKit.instruction => {
  let (exchange, _) = await Pda.exchange(programId)
  let (market, _) = await Pda.market(programId, ~marketId)
  {
    programAddress: programId,
    accounts: [signerMut(manager), readonly(exchange), writable(market)],
    data: concatBytes([u8(Constants.Instruction.activateMarket)]),
  }
}

// ── SetOracle (opcode 33) ─────────────────────────────────────────────────────
// Authority-only oracle reassignment (not allowed to be the zero pubkey).
let setOracle = async (
  ~programId: SolanaKit.address,
  ~authority: SolanaKit.address,
  ~market: SolanaKit.address,
  ~newOracle: SolanaKit.address,
): result<SolanaKit.instruction, SdkError.t> =>
  if isZeroAddress(newOracle) {
    Error(SdkError.Validation("oracle must not be the zero pubkey"))
  } else {
    let (exchange, _) = await Pda.exchange(programId)
    Ok({
      programAddress: programId,
      accounts: [signer(authority), readonly(exchange), writable(market)],
      data: concatBytes([u8(Constants.Instruction.setOracle), addressBytes(newOracle)]),
    })
  }

// ── SetMarketFees (opcode 29) ─────────────────────────────────────────────────
// Manager-only; updates one or more markets in one instruction.
type marketFeeUpdate = {
  market: SolanaKit.address,
  makerFeeBps: int,
  takerFeeBps: int,
}

let setMarketFees = async (
  ~programId: SolanaKit.address,
  ~manager: SolanaKit.address,
  ~updates: array<marketFeeUpdate>,
): result<SolanaKit.instruction, SdkError.t> =>
  if Array.length(updates) == 0 {
    Error(SdkError.Validation("updates is required"))
  } else {
    switch updates->Array.reduce(Ok(), (acc, update) =>
      switch acc {
      | Error(_) => acc
      | Ok() => validateFeePair(~makerFeeBps=update.makerFeeBps, ~takerFeeBps=update.takerFeeBps)
      }
    ) {
    | Error(error) => Error(error)
    | Ok() =>
      let (exchange, _) = await Pda.exchange(programId)
      let accounts = [signerMut(manager), readonly(exchange)]
      let dataParts = [u8(Constants.Instruction.setMarketFees)]
      updates->Array.forEach(update => {
        accounts->Array.push(writable(update.market))
        dataParts->Array.push(i16(update.makerFeeBps))
        dataParts->Array.push(i16(update.takerFeeBps))
      })
      Ok({programAddress: programId, accounts, data: concatBytes(dataParts)})
    }
  }

// ── SetFeeReceiver (opcode 30) ────────────────────────────────────────────────
// Authority-only. The with-ATAs variant appends the optional account block the
// program uses to create the receiver's quote ATAs (same opcode + data).
let setFeeReceiver = async (
  ~programId: SolanaKit.address,
  ~authority: SolanaKit.address,
  ~newFeeReceiver: SolanaKit.address,
): result<SolanaKit.instruction, SdkError.t> =>
  if isZeroAddress(newFeeReceiver) {
    Error(SdkError.Validation("fee receiver must not be the zero pubkey"))
  } else {
    let (exchange, _) = await Pda.exchange(programId)
    Ok({
      programAddress: programId,
      accounts: [signerMut(authority), writable(exchange)],
      data: concatBytes([u8(Constants.Instruction.setFeeReceiver), addressBytes(newFeeReceiver)]),
    })
  }

let setFeeReceiverWithAtas = async (
  ~programId: SolanaKit.address,
  ~authority: SolanaKit.address,
  ~newFeeReceiver: SolanaKit.address,
  ~quoteMints: array<SolanaKit.address>,
): result<SolanaKit.instruction, SdkError.t> =>
  if isZeroAddress(newFeeReceiver) {
    Error(SdkError.Validation("fee receiver must not be the zero pubkey"))
  } else if Array.length(quoteMints) == 0 {
    Error(SdkError.Validation("quote_mints is required"))
  } else {
    let (exchange, _) = await Pda.exchange(programId)
    let accounts = [
      signerMut(authority),
      writable(exchange),
      readonly(newFeeReceiver),
      readonly(Constants.tokenProgram),
      readonly(Constants.associatedTokenProgram),
      readonly(Constants.systemProgram),
    ]
    for index in 0 to Array.length(quoteMints) - 1 {
      let quoteMint = quoteMints->Array.getUnsafe(index)
      let (receiverQuoteAta, _) = await ata(~owner=newFeeReceiver, ~mint=quoteMint)
      accounts->Array.push(readonly(quoteMint))
      accounts->Array.push(writable(receiverQuoteAta))
    }
    Ok({
      programAddress: programId,
      accounts,
      data: concatBytes([u8(Constants.Instruction.setFeeReceiver), addressBytes(newFeeReceiver)]),
    })
  }

// ── WhitelistDepositToken (opcode 16) / SetDepositTokenStatus (opcode 38) ─────
let whitelistDepositToken = async (
  ~programId: SolanaKit.address,
  ~authority: SolanaKit.address,
  ~mint: SolanaKit.address,
): SolanaKit.instruction => {
  let (exchange, _) = await Pda.exchange(programId)
  let (globalDepositToken, _) = await Pda.globalDepositToken(programId, ~mint)
  {
    programAddress: programId,
    accounts: [
      signerMut(authority),
      writable(exchange),
      readonly(mint),
      writable(globalDepositToken),
      readonly(Constants.systemProgram),
    ],
    data: concatBytes([u8(Constants.Instruction.whitelistDepositToken)]),
  }
}

let setDepositTokenStatus = async (
  ~programId: SolanaKit.address,
  ~manager: SolanaKit.address,
  ~mint: SolanaKit.address,
  ~active: bool,
): SolanaKit.instruction => {
  let (exchange, _) = await Pda.exchange(programId)
  let (globalDepositToken, _) = await Pda.globalDepositToken(programId, ~mint)
  {
    programAddress: programId,
    accounts: [signer(manager), readonly(exchange), writable(globalDepositToken)],
    data: concatBytes([u8(Constants.Instruction.setDepositTokenStatus), u8(active ? 1 : 0)]),
  }
}

// ── Conditional metadata (opcodes 31 / 32) ────────────────────────────────────
let conditionalMetadata = async (
  ~programId,
  ~manager,
  ~market,
  ~depositMint,
  ~outcomeIndex,
  ~name,
  ~symbol,
  ~uri,
  ~isCreate,
): result<SolanaKit.instruction, SdkError.t> =>
  if outcomeIndex < 0 || outcomeIndex >= Constants.maxOutcomes {
    Error(SdkError.Validation(`invalid outcome index: ${Int.toString(outcomeIndex)}`))
  } else {
    switch serializeConditionalMetadata(~name, ~symbol, ~uri) {
    | Error(error) => Error(error)
    | Ok(metadataBytes) =>
      let (exchange, _) = await Pda.exchange(programId)
      let (conditionalMint, _) = await Pda.conditionalMint(programId, ~market, ~depositMint, ~outcomeIndex)
      let (mintAuthority, _) = await Pda.mintAuthority(programId, ~market)
      let (metadata, _) = await Pda.mplMetadata(~conditionalMint)
      let accounts = [
        isCreate ? signerMut(manager) : signer(manager),
        readonly(exchange),
        readonly(market),
        readonly(depositMint),
        readonly(conditionalMint),
        writable(metadata),
        readonly(mintAuthority),
        readonly(Constants.mplTokenMetadataProgram),
      ]
      if isCreate {
        accounts->Array.push(readonly(Constants.systemProgram))
        accounts->Array.push(readonly(Constants.rentSysvar))
      }
      Ok({
        programAddress: programId,
        accounts,
        data: concatBytes([
          u8(
            isCreate
              ? Constants.Instruction.createConditionalMetadata
              : Constants.Instruction.updateConditionalMetadata,
          ),
          u8(outcomeIndex),
          metadataBytes,
        ]),
      })
    }
  }

let createConditionalMetadata = (
  ~programId: SolanaKit.address,
  ~manager: SolanaKit.address,
  ~market: SolanaKit.address,
  ~depositMint: SolanaKit.address,
  ~outcomeIndex: int,
  ~name: string,
  ~symbol: string,
  ~uri: string,
) =>
  conditionalMetadata(~programId, ~manager, ~market, ~depositMint, ~outcomeIndex, ~name, ~symbol, ~uri, ~isCreate=true)

let updateConditionalMetadata = (
  ~programId: SolanaKit.address,
  ~manager: SolanaKit.address,
  ~market: SolanaKit.address,
  ~depositMint: SolanaKit.address,
  ~outcomeIndex: int,
  ~name: string,
  ~symbol: string,
  ~uri: string,
) =>
  conditionalMetadata(~programId, ~manager, ~market, ~depositMint, ~outcomeIndex, ~name, ~symbol, ~uri, ~isCreate=false)

// ── MatchOrdersMulti (opcode 13) ──────────────────────────────────────────────
// Operator-only taker↔makers match. Full fills (taker bit 7, maker bit i) skip
// that order's status account. Data: [13, taker order 37, taker sig 64,
// num_makers u8, full_fill_bitmask u8, per maker: order 37 ‖ sig 64 ‖
// maker_fill u64 ‖ taker_fill u64].
let matchOrdersMulti = async (
  ~programId: SolanaKit.address,
  ~operator: SolanaKit.address,
  ~market: SolanaKit.address,
  ~baseMint: SolanaKit.address,
  ~quoteMint: SolanaKit.address,
  ~feeReceiver: SolanaKit.address,
  ~takerOrder: signedOrder,
  ~takerIsFullFill: bool,
  ~makers: array<matchMaker>,
): result<SolanaKit.instruction, SdkError.t> =>
  if Array.length(makers) == 0 {
    Error(SdkError.Validation("maker_orders is required"))
  } else if Array.length(makers) > Constants.maxMakers {
    Error(SdkError.Validation(`too many makers: ${Int.toString(Array.length(makers))}`))
  } else {
    let (exchange, _) = await Pda.exchange(programId)
    let (orderbook, _) = await Pda.orderbook(programId, ~mintA=baseMint, ~mintB=quoteMint)
    let (takerNonce, _) = await Pda.userNonce(programId, ~user=takerOrder.order.maker)
    let (takerPosition, _) = await Pda.position(programId, ~owner=takerOrder.order.maker, ~market)
    let (takerBaseAta, _) = await ata(~owner=takerPosition, ~mint=baseMint)
    let (takerQuoteAta, _) = await ata(~owner=takerPosition, ~mint=quoteMint)
    let (feeReceiverQuoteAta, _) = await ata(~owner=feeReceiver, ~mint=quoteMint)

    let accounts = [signerMut(operator), readonly(exchange), readonly(market), readonly(orderbook)]
    if !takerIsFullFill {
      let (takerOrderStatus, _) =
        await Pda.orderStatus(programId, ~orderHash=OrderPayload.hash(takerOrder.order))
      accounts->Array.push(writable(takerOrderStatus))
    }
    accounts->Array.push(readonly(takerNonce))
    accounts->Array.push(writable(takerPosition))
    accounts->Array.push(readonly(baseMint))
    accounts->Array.push(readonly(quoteMint))
    accounts->Array.push(writable(takerBaseAta))
    accounts->Array.push(writable(takerQuoteAta))
    accounts->Array.push(readonly(Constants.tokenProgram))
    accounts->Array.push(readonly(Constants.systemProgram))
    accounts->Array.push(writable(feeReceiverQuoteAta))
    accounts->Array.push(readonly(feeReceiver))
    accounts->Array.push(readonly(Constants.associatedTokenProgram))

    let fullFillBitmask = ref(takerIsFullFill ? 128 : 0)
    for index in 0 to Array.length(makers) - 1 {
      let maker = makers->Array.getUnsafe(index)
      if maker.isFullFill {
        fullFillBitmask := fullFillBitmask.contents + bitValue(index)
      }
      if !maker.isFullFill {
        let (makerOrderStatus, _) =
          await Pda.orderStatus(programId, ~orderHash=OrderPayload.hash(maker.order.order))
        accounts->Array.push(writable(makerOrderStatus))
      }
      let (makerNonce, _) = await Pda.userNonce(programId, ~user=maker.order.order.maker)
      let (makerPosition, _) = await Pda.position(programId, ~owner=maker.order.order.maker, ~market)
      let (makerBaseAta, _) = await ata(~owner=makerPosition, ~mint=baseMint)
      let (makerQuoteAta, _) = await ata(~owner=makerPosition, ~mint=quoteMint)
      accounts->Array.push(readonly(makerNonce))
      accounts->Array.push(writable(makerPosition))
      accounts->Array.push(writable(makerBaseAta))
      accounts->Array.push(writable(makerQuoteAta))
    }

    let dataParts = [
      u8(Constants.Instruction.matchOrdersMulti),
      signedOrderBytes(takerOrder),
      u8(Array.length(makers)),
      u8(fullFillBitmask.contents),
    ]
    makers->Array.forEach(maker => {
      dataParts->Array.push(signedOrderBytes(maker.order))
      dataParts->Array.push(u64(maker.makerFillAmount))
      dataParts->Array.push(u64(maker.takerFillAmount))
    })
    Ok({programAddress: programId, accounts, data: concatBytes(dataParts)})
  }

// ── CreateOrderbook (opcode 15) ───────────────────────────────────────────────
// Manager-only; the mint pair is canonicalized by raw byte order (matching the
// orderbook PDA derivation), and `base` names which INPUT is the base asset.
type orderbookMint = {
  mint: SolanaKit.address,
  depositMint: SolanaKit.address,
  outcomeIndex: int,
}

let addressLte: (Uint8Array.t, Uint8Array.t) => bool = %raw(`function (a, b) {
  for (let i = 0; i < a.length; i++) { if (a[i] !== b[i]) return a[i] < b[i]; }
  return true;
}`)

let createOrderbook = async (
  ~programId: SolanaKit.address,
  ~manager: SolanaKit.address,
  ~market: SolanaKit.address,
  ~mintA: orderbookMint,
  ~mintB: orderbookMint,
  // 0 ⇒ mintA is the base asset, 1 ⇒ mintB.
  ~baseIndex: int,
  ~feeReceiver: SolanaKit.address,
  ~recentSlot: bigint,
): result<SolanaKit.instruction, SdkError.t> =>
  if baseIndex < 0 || baseIndex > 1 {
    Error(SdkError.Validation(`invalid base index: ${Int.toString(baseIndex)}`))
  } else if SolanaKit.addressToString(mintA.mint) == SolanaKit.addressToString(mintB.mint) {
    Error(SdkError.Validation("mint_a and mint_b must differ"))
  } else {
    // Canonicalize by raw byte order; base flags follow their mints.
    let aIsFirst = addressLte(addressBytes(mintA.mint), addressBytes(mintB.mint))
    let (first, second) = aIsFirst ? (mintA, mintB) : (mintB, mintA)
    let firstIsBase = aIsFirst ? baseIndex == 0 : baseIndex == 1
    let canonicalBaseIndex = firstIsBase ? 0 : 1

    let (exchange, _) = await Pda.exchange(programId)
    let (orderbook, _) = await Pda.orderbook(programId, ~mintA=first.mint, ~mintB=second.mint)
    let (lookupTable, _) = await Pda.alt(~orderbook, ~recentSlot)
    let quoteMint = canonicalBaseIndex == 0 ? second.mint : first.mint
    let (feeReceiverQuoteAta, _) = await ata(~owner=feeReceiver, ~mint=quoteMint)
    Ok({
      programAddress: programId,
      accounts: [
        signerMut(manager),
        readonly(market),
        readonly(first.mint),
        readonly(second.mint),
        writable(orderbook),
        writable(lookupTable),
        readonly(exchange),
        readonly(Constants.altProgram),
        readonly(Constants.systemProgram),
        readonly(first.depositMint),
        readonly(second.depositMint),
        readonly(Constants.tokenProgram),
        readonly(Constants.associatedTokenProgram),
        readonly(feeReceiver),
        writable(feeReceiverQuoteAta),
      ],
      data: concatBytes([
        u8(Constants.Instruction.createOrderbook),
        u64(recentSlot),
        u8(canonicalBaseIndex),
        u8(first.outcomeIndex),
        u8(second.outcomeIndex),
      ]),
    })
  }

// ── RefreshOrderbookAlt (opcode 34) ───────────────────────────────────────────
// Manager-only: ensures the current fee receiver's quote ATA exists and is in
// the orderbook ALT.
let refreshOrderbookAlt = async (
  ~programId: SolanaKit.address,
  ~manager: SolanaKit.address,
  ~market: SolanaKit.address,
  ~orderbook: SolanaKit.address,
  ~lookupTable: SolanaKit.address,
  ~quoteMint: SolanaKit.address,
  ~feeReceiver: SolanaKit.address,
): SolanaKit.instruction => {
  let (exchange, _) = await Pda.exchange(programId)
  let (feeReceiverQuoteAta, _) = await ata(~owner=feeReceiver, ~mint=quoteMint)
  {
    programAddress: programId,
    accounts: [
      signerMut(manager),
      readonly(exchange),
      readonly(market),
      readonly(orderbook),
      writable(lookupTable),
      readonly(quoteMint),
      readonly(feeReceiver),
      writable(feeReceiverQuoteAta),
      readonly(Constants.tokenProgram),
      readonly(Constants.associatedTokenProgram),
      readonly(Constants.altProgram),
      readonly(Constants.systemProgram),
    ],
    data: concatBytes([u8(Constants.Instruction.refreshOrderbookAlt)]),
  }
}

// ── DepositToGlobal with ALT context (opcode 17) ──────────────────────────────
// The deposit-to-global instruction with the optional user-deposit-ALT block:
// `Create` derives the ALT from (user_nonce, recent_slot) and appends the slot
// to the data; `Extend` reuses an existing table.
type depositToGlobalAltContext =
  | Create({recentSlot: bigint})
  | Extend({lookupTable: SolanaKit.address})

let depositToGlobalWithAlt = async (
  ~programId: SolanaKit.address,
  ~user: SolanaKit.address,
  ~mint: SolanaKit.address,
  ~amount: bigint,
  ~altContext: depositToGlobalAltContext,
): SolanaKit.instruction => {
  let (globalDepositToken, _) = await Pda.globalDepositToken(programId, ~mint)
  let (userGlobalDeposit, _) = await Pda.userGlobalDeposit(programId, ~user, ~mint)
  let (exchange, _) = await Pda.exchange(programId)
  let (userTokenAccount, _) = await ata(~owner=user, ~mint)
  let (userNonce, _) = await Pda.userNonce(programId, ~user)

  let dataParts = [u8(Constants.Instruction.depositToGlobal), u64(amount)]
  let lookupTable = switch altContext {
  | Create({recentSlot}) =>
    dataParts->Array.push(u64(recentSlot))
    let (derived, _) = await Pda.alt(~orderbook=userNonce, ~recentSlot)
    derived
  | Extend({lookupTable}) => lookupTable
  }
  {
    programAddress: programId,
    accounts: [
      signerMut(user),
      readonly(globalDepositToken),
      readonly(mint),
      writable(userGlobalDeposit),
      writable(userTokenAccount),
      readonly(Constants.tokenProgram),
      readonly(Constants.systemProgram),
      readonly(exchange),
      readonly(userNonce),
      writable(lookupTable),
      readonly(Constants.altProgram),
    ],
    data: concatBytes(dataParts),
  }
}

// ── DepositAndSwap (opcode 20) ────────────────────────────────────────────────
// Unified order execution: each participant may deposit from global and/or swap
// conditional tokens. Bitmasks: full-fill (taker bit 7, maker bit i) and deposit
// (same layout). Data: [20, taker order 37, taker sig 64, num_makers u8,
// full_fill_bitmask u8, deposit_bitmask u8, per maker: order ‖ sig ‖ fills].
let depositAndSwap = async (
  ~programId: SolanaKit.address,
  ~operator: SolanaKit.address,
  ~market: SolanaKit.address,
  ~baseMint: SolanaKit.address,
  ~quoteMint: SolanaKit.address,
  ~feeReceiver: SolanaKit.address,
  ~takerOrder: signedOrder,
  ~takerIsFullFill: bool,
  ~takerIsDeposit: bool,
  ~takerDepositMint: SolanaKit.address,
  ~numOutcomes: int,
  ~makers: array<swapMaker>,
): result<SolanaKit.instruction, SdkError.t> =>
  if Array.length(makers) == 0 {
    Error(SdkError.Validation("makers is required"))
  } else if Array.length(makers) > Constants.maxMakers {
    Error(SdkError.Validation(`too many makers: ${Int.toString(Array.length(makers))}`))
  } else {
    let (exchange, _) = await Pda.exchange(programId)
    let (orderbook, _) = await Pda.orderbook(programId, ~mintA=baseMint, ~mintB=quoteMint)
    let (mintAuthority, _) = await Pda.mintAuthority(programId, ~market)
    let (takerPosition, _) = await Pda.position(programId, ~owner=takerOrder.order.maker, ~market)
    let (takerNonce, _) = await Pda.userNonce(programId, ~user=takerOrder.order.maker)
    let (feeReceiverQuoteAta, _) = await ata(~owner=feeReceiver, ~mint=quoteMint)
    // Bid: receive base, give quote; Ask: the reverse.
    let (receiveMint, giveMint) =
      takerOrder.order.side == 0 ? (baseMint, quoteMint) : (quoteMint, baseMint)

    let accounts = [
      signerMut(operator),
      readonly(exchange),
      readonly(market),
      readonly(orderbook),
      readonly(mintAuthority),
      readonly(Constants.tokenProgram),
      writable(feeReceiverQuoteAta),
      readonly(feeReceiver),
      readonly(Constants.associatedTokenProgram),
    ]
    if !takerIsFullFill {
      let (takerOrderStatus, _) =
        await Pda.orderStatus(programId, ~orderHash=OrderPayload.hash(takerOrder.order))
      accounts->Array.push(writable(takerOrderStatus))
    }
    let (takerReceiveAta, _) = await ata(~owner=takerPosition, ~mint=receiveMint)
    let (takerGiveAta, _) = await ata(~owner=takerPosition, ~mint=giveMint)
    accounts->Array.push(readonly(takerNonce))
    accounts->Array.push(writable(takerPosition))
    accounts->Array.push(readonly(baseMint))
    accounts->Array.push(readonly(quoteMint))
    accounts->Array.push(writable(takerReceiveAta))
    accounts->Array.push(writable(takerGiveAta))
    accounts->Array.push(readonly(Constants.systemProgram))

    if takerIsDeposit {
      let (vault, _) = await Pda.vault(programId, ~depositMint=takerDepositMint, ~market)
      let (gdt, _) = await Pda.globalDepositToken(programId, ~mint=takerDepositMint)
      let (takerGlobalDeposit, _) =
        await Pda.userGlobalDeposit(programId, ~user=takerOrder.order.maker, ~mint=takerDepositMint)
      accounts->Array.push(readonly(takerDepositMint))
      accounts->Array.push(writable(vault))
      accounts->Array.push(readonly(gdt))
      accounts->Array.push(writable(takerGlobalDeposit))
      for outcomeIndex in 0 to numOutcomes - 1 {
        let (conditionalMint, _) =
          await Pda.conditionalMint(programId, ~market, ~depositMint=takerDepositMint, ~outcomeIndex)
        let (takerAta, _) = await ata(~owner=takerPosition, ~mint=conditionalMint)
        accounts->Array.push(writable(conditionalMint))
        accounts->Array.push(writable(takerAta))
      }
    }

    let fullFillBitmask = ref(takerIsFullFill ? 128 : 0)
    let depositBitmask = ref(takerIsDeposit ? 128 : 0)
    for index in 0 to Array.length(makers) - 1 {
      let maker = makers->Array.getUnsafe(index)
      if maker.isFullFill {
        fullFillBitmask := fullFillBitmask.contents + bitValue(index)
      }
      if maker.isDeposit {
        depositBitmask := depositBitmask.contents + bitValue(index)
      }
      let (makerNonce, _) = await Pda.userNonce(programId, ~user=maker.order.order.maker)
      let (makerPosition, _) = await Pda.position(programId, ~owner=maker.order.order.maker, ~market)
      if !maker.isFullFill {
        let (makerOrderStatus, _) =
          await Pda.orderStatus(programId, ~orderHash=OrderPayload.hash(maker.order.order))
        accounts->Array.push(writable(makerOrderStatus))
      }
      accounts->Array.push(readonly(makerNonce))
      accounts->Array.push(writable(makerPosition))
      if maker.isDeposit {
        let (vault, _) = await Pda.vault(programId, ~depositMint=maker.depositMint, ~market)
        let (gdt, _) = await Pda.globalDepositToken(programId, ~mint=maker.depositMint)
        let (makerGlobalDeposit, _) =
          await Pda.userGlobalDeposit(programId, ~user=maker.order.order.maker, ~mint=maker.depositMint)
        accounts->Array.push(readonly(maker.depositMint))
        accounts->Array.push(writable(vault))
        accounts->Array.push(readonly(gdt))
        accounts->Array.push(writable(makerGlobalDeposit))
        for outcomeIndex in 0 to numOutcomes - 1 {
          let (conditionalMint, _) =
            await Pda.conditionalMint(programId, ~market, ~depositMint=maker.depositMint, ~outcomeIndex)
          let (makerAta, _) = await ata(~owner=makerPosition, ~mint=conditionalMint)
          accounts->Array.push(writable(conditionalMint))
          accounts->Array.push(writable(makerAta))
        }
      }
      let (makerReceiveAta, _) = await ata(~owner=makerPosition, ~mint=receiveMint)
      let (makerGiveAta, _) = await ata(~owner=makerPosition, ~mint=giveMint)
      accounts->Array.push(writable(makerReceiveAta))
      accounts->Array.push(writable(makerGiveAta))
    }

    let dataParts = [
      u8(Constants.Instruction.depositAndSwap),
      signedOrderBytes(takerOrder),
      u8(Array.length(makers)),
      u8(fullFillBitmask.contents),
      u8(depositBitmask.contents),
    ]
    makers->Array.forEach(maker => {
      dataParts->Array.push(signedOrderBytes(maker.order))
      dataParts->Array.push(u64(maker.makerFillAmount))
      dataParts->Array.push(u64(maker.takerFillAmount))
    })
    Ok({programAddress: programId, accounts, data: concatBytes(dataParts)})
  }

// ── CloseOrderStatus (opcode 24) ──────────────────────────────────────────────
// Operator-only: closes a fully-filled, non-cancelled order status PDA.
// Accounts (3): operator(signer,mut) exchange order_status(mut).
// Data: [24, order_hash 32]
let closeOrderStatus = async (
  ~programId: SolanaKit.address,
  ~operator: SolanaKit.address,
  ~orderHash: Uint8Array.t,
): SolanaKit.instruction => {
  let (exchange, _) = await Pda.exchange(programId)
  let (orderStatus, _) = await Pda.orderStatus(programId, ~orderHash)
  {
    programAddress: programId,
    accounts: [signerMut(operator), readonly(exchange), writable(orderStatus)],
    data: concatBytes([u8(Constants.Instruction.closeOrderStatus), orderHash]),
  }
}

// ── CloseOrderbookAlt (opcode 26) / CloseOrderbook (opcode 27) ────────────────
// Operator-only ALT deactivate/close, then the orderbook PDA itself.
let closeOrderbookAlt = async (
  ~programId: SolanaKit.address,
  ~operator: SolanaKit.address,
  ~orderbook: SolanaKit.address,
  ~market: SolanaKit.address,
  ~lookupTable: SolanaKit.address,
): SolanaKit.instruction => {
  let (exchange, _) = await Pda.exchange(programId)
  {
    programAddress: programId,
    accounts: [
      signerMut(operator),
      readonly(exchange),
      readonly(orderbook),
      readonly(market),
      writable(lookupTable),
      readonly(Constants.altProgram),
    ],
    data: concatBytes([u8(Constants.Instruction.closeOrderbookAlt)]),
  }
}

let closeOrderbook = async (
  ~programId: SolanaKit.address,
  ~operator: SolanaKit.address,
  ~orderbook: SolanaKit.address,
  ~market: SolanaKit.address,
  ~lookupTable: SolanaKit.address,
): SolanaKit.instruction => {
  let (exchange, _) = await Pda.exchange(programId)
  {
    programAddress: programId,
    accounts: [
      signerMut(operator),
      readonly(exchange),
      writable(orderbook),
      readonly(market),
      readonly(lookupTable),
    ],
    data: concatBytes([u8(Constants.Instruction.closeOrderbook)]),
  }
}
