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
