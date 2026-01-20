import {
  PublicKey,
  TransactionInstruction,
  AccountMeta,
} from "@solana/web3.js";
import {
  PROGRAM_ID,
  INSTRUCTION,
  SYSTEM_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  TOKEN_2022_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  RENT_SYSVAR_ID,
  INSTRUCTIONS_SYSVAR_ID,
} from "../shared/constants";
import {
  InitializeParams,
  CreateMarketParams,
  AddDepositMintParams,
  MintCompleteSetParams,
  MergeCompleteSetParams,
  SettleMarketParams,
  RedeemWinningsParams,
  WithdrawFromPositionParams,
  ActivateMarketParams,
  MatchOrdersMultiParams,
  FullOrder,
  OutcomeMetadata,
} from "../shared/types";
import {
  getExchangePda,
  getMarketPda,
  getVaultPda,
  getMintAuthorityPda,
  getConditionalMintPda,
  getAllConditionalMintPdas,
  getOrderStatusPda,
  getUserNoncePda,
  getPositionPda,
} from "./pda";
import {
  toU8,
  toU64Le,
  serializeString,
  getConditionalTokenAta,
  getDepositTokenAta,
  validateOutcomes,
} from "../shared/utils";
import { hashOrder, serializeFullOrder, serializeCompactOrder } from "./orders";

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

/**
 * Create an account meta for a signer
 */
function signerMut(pubkey: PublicKey): AccountMeta {
  return { pubkey, isSigner: true, isWritable: true };
}

/**
 * Create an account meta for a writable account
 */
function writable(pubkey: PublicKey): AccountMeta {
  return { pubkey, isSigner: false, isWritable: true };
}

/**
 * Create an account meta for a read-only account
 */
function readonly(pubkey: PublicKey): AccountMeta {
  return { pubkey, isSigner: false, isWritable: false };
}

// ============================================================================
// INSTRUCTION BUILDERS
// ============================================================================

/**
 * Build Initialize instruction
 * Creates the exchange account (singleton)
 *
 * Accounts:
 * 0. authority (signer, mut) - Initial admin
 * 1. exchange (mut) - Exchange PDA
 * 2. system_program (readonly)
 *
 * Data: [discriminator (1 byte)]
 */
export function buildInitializeIx(
  params: InitializeParams,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  const [exchange] = getExchangePda(programId);

  const keys: AccountMeta[] = [
    signerMut(params.authority),
    writable(exchange),
    readonly(SYSTEM_PROGRAM_ID),
  ];

  const data = Buffer.from([INSTRUCTION.INITIALIZE]);

  return new TransactionInstruction({
    keys,
    programId,
    data,
  });
}

/**
 * Build CreateMarket instruction
 * Creates a new market in Pending status
 *
 * Accounts:
 * 0. authority (signer, mut) - Must be exchange authority
 * 1. exchange (mut) - Exchange PDA
 * 2. market (mut) - Market PDA
 * 3. system_program (readonly)
 *
 * Data: [discriminator, num_outcomes (u8), oracle (32), question_id (32)]
 */
export function buildCreateMarketIx(
  params: CreateMarketParams,
  marketId: bigint,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  validateOutcomes(params.numOutcomes);

  const [exchange] = getExchangePda(programId);
  const [market] = getMarketPda(marketId, programId);

  const keys: AccountMeta[] = [
    signerMut(params.authority),
    writable(exchange),
    writable(market),
    readonly(SYSTEM_PROGRAM_ID),
  ];

  // Data: [1, numOutcomes (u8), oracle (32), questionId (32)]
  const data = Buffer.concat([
    Buffer.from([INSTRUCTION.CREATE_MARKET]),
    toU8(params.numOutcomes),
    params.oracle.toBuffer(),
    params.questionId,
  ]);

  return new TransactionInstruction({
    keys,
    programId,
    data,
  });
}

/**
 * Serialize outcome metadata for addDepositMint instruction
 */
function serializeOutcomeMetadata(metadata: OutcomeMetadata[]): Buffer {
  const buffers: Buffer[] = [];
  for (const m of metadata) {
    buffers.push(serializeString(m.name));
    buffers.push(serializeString(m.symbol));
    buffers.push(serializeString(m.uri));
  }
  return Buffer.concat(buffers);
}

/**
 * Build AddDepositMint instruction
 * Sets up vault and conditional mints for a deposit token
 *
 * Accounts (from Rust):
 * 0. payer (signer)
 * 1. market
 * 2. deposit_mint
 * 3. vault
 * 4. mint_authority
 * 5. token_program (SPL Token)
 * 6. token_2022_program
 * 7. system_program
 * 8+ conditional_mints[0..num_outcomes]
 *
 * Data: [discriminator, ...serialized_metadata]
 */
export function buildAddDepositMintIx(
  params: AddDepositMintParams,
  market: PublicKey,
  numOutcomes: number,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  if (params.outcomeMetadata.length !== numOutcomes) {
    throw new Error(
      `Outcome metadata count (${params.outcomeMetadata.length}) must match numOutcomes (${numOutcomes})`
    );
  }

  const [vault] = getVaultPda(params.depositMint, market, programId);
  const [mintAuthority] = getMintAuthorityPda(market, programId);
  const conditionalMints = getAllConditionalMintPdas(
    market,
    params.depositMint,
    numOutcomes,
    programId
  );

  const keys: AccountMeta[] = [
    signerMut(params.authority),           // 0. payer (signer)
    writable(market),                       // 1. market
    readonly(params.depositMint),           // 2. deposit_mint
    writable(vault),                        // 3. vault
    readonly(mintAuthority),                // 4. mint_authority
    readonly(TOKEN_PROGRAM_ID),             // 5. token_program (SPL Token)
    readonly(TOKEN_2022_PROGRAM_ID),        // 6. token_2022_program
    readonly(SYSTEM_PROGRAM_ID),            // 7. system_program
  ];

  // 8+ conditional_mints[0..num_outcomes]
  for (const [mint] of conditionalMints) {
    keys.push(writable(mint));
  }

  const data = Buffer.concat([
    Buffer.from([INSTRUCTION.ADD_DEPOSIT_MINT]),
    serializeOutcomeMetadata(params.outcomeMetadata),
  ]);

  return new TransactionInstruction({
    keys,
    programId,
    data,
  });
}

/**
 * Build MintCompleteSet instruction
 * Deposits collateral and mints all outcome tokens into Position PDA
 *
 * Accounts (from Rust):
 * 0. user (signer) - User depositing collateral
 * 1. exchange - Exchange account (for pause check)
 * 2. market - Market account
 * 3. deposit_mint - Collateral token mint
 * 4. vault - Market vault for collateral
 * 5. user_deposit_ata - User's collateral token account
 * 6. position - Position PDA (created if not exists)
 * 7. position_collateral_ata - Position's collateral ATA (created if not exists)
 * 8. mint_authority - Market mint authority PDA
 * 9. token_program - SPL Token program (for collateral)
 * 10. token_2022_program - Token-2022 program (for conditional tokens)
 * 11. associated_token_program - ATA program
 * 12. system_program - System program
 *
 * Remaining accounts (for each outcome i):
 * - conditional_mint[i]
 * - position_conditional_ata[i]
 *
 * Data: [discriminator, amount (u64)]
 */
export function buildMintCompleteSetIx(
  params: MintCompleteSetParams,
  numOutcomes: number,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  const [exchange] = getExchangePda(programId);
  const [vault] = getVaultPda(params.depositMint, params.market, programId);
  const [mintAuthority] = getMintAuthorityPda(params.market, programId);
  const [position] = getPositionPda(params.user, params.market, programId);
  const userDepositAta = getDepositTokenAta(params.depositMint, params.user);
  const positionCollateralAta = getDepositTokenAta(params.depositMint, position);
  const conditionalMints = getAllConditionalMintPdas(
    params.market,
    params.depositMint,
    numOutcomes,
    programId
  );

  const keys: AccountMeta[] = [
    signerMut(params.user),                   // 0. user (signer)
    readonly(exchange),                        // 1. exchange
    readonly(params.market),                   // 2. market
    readonly(params.depositMint),              // 3. deposit_mint
    writable(vault),                           // 4. vault
    writable(userDepositAta),                  // 5. user_deposit_ata
    writable(position),                        // 6. position
    writable(positionCollateralAta),           // 7. position_collateral_ata
    readonly(mintAuthority),                   // 8. mint_authority
    readonly(TOKEN_PROGRAM_ID),                // 9. token_program
    readonly(TOKEN_2022_PROGRAM_ID),           // 10. token_2022_program
    readonly(ASSOCIATED_TOKEN_PROGRAM_ID),     // 11. associated_token_program
    readonly(SYSTEM_PROGRAM_ID),               // 12. system_program
  ];

  // Remaining: conditional_mint[i], position_conditional_ata[i] for each outcome
  for (const [mint] of conditionalMints) {
    keys.push(writable(mint));
    const positionAta = getConditionalTokenAta(mint, position);
    keys.push(writable(positionAta));
  }

  const data = Buffer.concat([
    Buffer.from([INSTRUCTION.MINT_COMPLETE_SET]),
    toU64Le(params.amount),
  ]);

  return new TransactionInstruction({
    keys,
    programId,
    data,
  });
}

/**
 * Build MergeCompleteSet instruction
 * Burns all outcome tokens from Position and releases collateral
 *
 * Accounts (from Rust):
 * 0. user (signer) - Position owner
 * 1. exchange - Exchange account (for pause check)
 * 2. market - Market account
 * 3. deposit_mint - Collateral token mint
 * 4. vault - Market vault
 * 5. position - User's Position PDA
 * 6. user_deposit_ata - User's collateral ATA to receive funds
 * 7. mint_authority - Market mint authority PDA
 * 8. token_program - SPL Token program (for collateral)
 * 9. token_2022_program - Token-2022 program (for conditional tokens)
 *
 * Remaining accounts (for each outcome i):
 * - conditional_mint[i]
 * - position_conditional_ata[i]
 *
 * Data: [discriminator, amount (u64)]
 */
export function buildMergeCompleteSetIx(
  params: MergeCompleteSetParams,
  numOutcomes: number,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  const [exchange] = getExchangePda(programId);
  const [vault] = getVaultPda(params.depositMint, params.market, programId);
  const [mintAuthority] = getMintAuthorityPda(params.market, programId);
  const [position] = getPositionPda(params.user, params.market, programId);
  const userDepositAta = getDepositTokenAta(params.depositMint, params.user);
  const conditionalMints = getAllConditionalMintPdas(
    params.market,
    params.depositMint,
    numOutcomes,
    programId
  );

  const keys: AccountMeta[] = [
    signerMut(params.user),                   // 0. user (signer)
    readonly(exchange),                        // 1. exchange
    readonly(params.market),                   // 2. market
    readonly(params.depositMint),              // 3. deposit_mint
    writable(vault),                           // 4. vault
    writable(position),                        // 5. position
    writable(userDepositAta),                  // 6. user_deposit_ata
    readonly(mintAuthority),                   // 7. mint_authority
    readonly(TOKEN_PROGRAM_ID),                // 8. token_program
    readonly(TOKEN_2022_PROGRAM_ID),           // 9. token_2022_program
  ];

  // Remaining: conditional_mint[i], position_conditional_ata[i] for each outcome
  for (const [mint] of conditionalMints) {
    keys.push(writable(mint));
    const positionAta = getConditionalTokenAta(mint, position);
    keys.push(writable(positionAta));
  }

  const data = Buffer.concat([
    Buffer.from([INSTRUCTION.MERGE_COMPLETE_SET]),
    toU64Le(params.amount),
  ]);

  return new TransactionInstruction({
    keys,
    programId,
    data,
  });
}

/**
 * Build CancelOrder instruction
 * Marks an order as cancelled
 *
 * Accounts:
 * 0. maker (signer, mut)
 * 1. order_status (mut)
 * 2. system_program (readonly)
 *
 * Data: [discriminator, order_hash (32), full_order (225)]
 */
export function buildCancelOrderIx(
  maker: PublicKey,
  order: FullOrder,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  const orderHash = hashOrder(order);
  const [orderStatus] = getOrderStatusPda(orderHash, programId);

  const keys: AccountMeta[] = [
    signerMut(maker),
    writable(orderStatus),
    readonly(SYSTEM_PROGRAM_ID),
  ];

  const data = Buffer.concat([
    Buffer.from([INSTRUCTION.CANCEL_ORDER]),
    orderHash,
    serializeFullOrder(order),
  ]);

  return new TransactionInstruction({
    keys,
    programId,
    data,
  });
}

/**
 * Build IncrementNonce instruction
 * Increments user's nonce for replay protection
 *
 * Accounts:
 * 0. user (signer, mut)
 * 1. user_nonce (mut)
 * 2. system_program (readonly)
 *
 * Data: [discriminator]
 */
export function buildIncrementNonceIx(
  user: PublicKey,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  const [userNonce] = getUserNoncePda(user, programId);

  const keys: AccountMeta[] = [
    signerMut(user),
    writable(userNonce),
    readonly(SYSTEM_PROGRAM_ID),
  ];

  const data = Buffer.from([INSTRUCTION.INCREMENT_NONCE]);

  return new TransactionInstruction({
    keys,
    programId,
    data,
  });
}

/**
 * Build SettleMarket instruction
 * Oracle resolves the market with winning outcome
 *
 * Accounts:
 * 0. oracle (signer)
 * 1. exchange (readonly)
 * 2. market (mut)
 *
 * Data: [discriminator, winning_outcome (u8)]
 */
export function buildSettleMarketIx(
  params: SettleMarketParams,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  const [exchange] = getExchangePda(programId);
  const [market] = getMarketPda(params.marketId, programId);

  const keys: AccountMeta[] = [
    signerMut(params.oracle),
    readonly(exchange),
    writable(market),
  ];

  const data = Buffer.concat([
    Buffer.from([INSTRUCTION.SETTLE_MARKET]),
    toU8(params.winningOutcome),
  ]);

  return new TransactionInstruction({
    keys,
    programId,
    data,
  });
}

/**
 * Build RedeemWinnings instruction
 * Redeem winning outcome tokens from Position for collateral after market resolution
 *
 * Accounts (from Rust):
 * 0. user (signer) - Position owner
 * 1. market - Market account (must be resolved)
 * 2. deposit_mint - Collateral token mint
 * 3. vault - Market vault
 * 4. winning_conditional_mint - Mint for the winning outcome
 * 5. position - User's Position PDA
 * 6. position_conditional_ata - Position's ATA for winning conditional token
 * 7. user_deposit_ata - User's collateral ATA to receive funds
 * 8. mint_authority - Market mint authority PDA
 * 9. token_program - SPL Token program (for collateral)
 * 10. token_2022_program - Token-2022 program (for conditional tokens)
 *
 * Data: [discriminator, amount (u64)]
 */
export function buildRedeemWinningsIx(
  params: RedeemWinningsParams,
  winningOutcome: number,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  const [vault] = getVaultPda(params.depositMint, params.market, programId);
  const [mintAuthority] = getMintAuthorityPda(params.market, programId);
  const [position] = getPositionPda(params.user, params.market, programId);
  const [winningMint] = getConditionalMintPda(
    params.market,
    params.depositMint,
    winningOutcome,
    programId
  );
  const positionWinningAta = getConditionalTokenAta(winningMint, position);
  const userDepositAta = getDepositTokenAta(params.depositMint, params.user);

  const keys: AccountMeta[] = [
    signerMut(params.user),                   // 0. user (signer)
    readonly(params.market),                   // 1. market
    readonly(params.depositMint),              // 2. deposit_mint
    writable(vault),                           // 3. vault
    writable(winningMint),                     // 4. winning_conditional_mint
    writable(position),                        // 5. position
    writable(positionWinningAta),              // 6. position_conditional_ata
    writable(userDepositAta),                  // 7. user_deposit_ata
    readonly(mintAuthority),                   // 8. mint_authority
    readonly(TOKEN_PROGRAM_ID),                // 9. token_program
    readonly(TOKEN_2022_PROGRAM_ID),           // 10. token_2022_program
  ];

  const data = Buffer.concat([
    Buffer.from([INSTRUCTION.REDEEM_WINNINGS]),
    toU64Le(params.amount),
  ]);

  return new TransactionInstruction({
    keys,
    programId,
    data,
  });
}

/**
 * Build SetPaused instruction
 * Admin: pause/unpause exchange
 *
 * Accounts:
 * 0. authority (signer)
 * 1. exchange (mut)
 *
 * Data: [discriminator, paused (u8)]
 */
export function buildSetPausedIx(
  authority: PublicKey,
  paused: boolean,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  const [exchange] = getExchangePda(programId);

  const keys: AccountMeta[] = [signerMut(authority), writable(exchange)];

  const data = Buffer.concat([
    Buffer.from([INSTRUCTION.SET_PAUSED]),
    toU8(paused ? 1 : 0),
  ]);

  return new TransactionInstruction({
    keys,
    programId,
    data,
  });
}

/**
 * Build SetOperator instruction
 * Admin: change operator pubkey
 *
 * Accounts:
 * 0. authority (signer)
 * 1. exchange (mut)
 *
 * Data: [discriminator, new_operator (32)]
 */
export function buildSetOperatorIx(
  authority: PublicKey,
  newOperator: PublicKey,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  const [exchange] = getExchangePda(programId);

  const keys: AccountMeta[] = [signerMut(authority), writable(exchange)];

  const data = Buffer.concat([
    Buffer.from([INSTRUCTION.SET_OPERATOR]),
    newOperator.toBuffer(),
  ]);

  return new TransactionInstruction({
    keys,
    programId,
    data,
  });
}

/**
 * Build WithdrawFromPosition instruction
 * Withdraw tokens from Position ATA to user's ATA
 *
 * Accounts:
 * 0. user (signer, mut)
 * 1. position (mut)
 * 2. mint (readonly) - Token mint (conditional or collateral)
 * 3. position_ata (mut)
 * 4. user_ata (mut)
 * 5. token_program (readonly) - SPL Token or Token-2022
 *
 * Data: [discriminator, amount (u64)]
 */
export function buildWithdrawFromPositionIx(
  params: WithdrawFromPositionParams,
  isToken2022: boolean,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  const [position] = getPositionPda(params.user, params.market, programId);
  const positionAta = isToken2022
    ? getConditionalTokenAta(params.mint, position)
    : getDepositTokenAta(params.mint, position);
  const userAta = isToken2022
    ? getConditionalTokenAta(params.mint, params.user)
    : getDepositTokenAta(params.mint, params.user);
  const tokenProgram = isToken2022 ? TOKEN_2022_PROGRAM_ID : TOKEN_PROGRAM_ID;

  const keys: AccountMeta[] = [
    signerMut(params.user),
    writable(position),
    readonly(params.mint),
    writable(positionAta),
    writable(userAta),
    readonly(tokenProgram),
  ];

  const data = Buffer.concat([
    Buffer.from([INSTRUCTION.WITHDRAW_FROM_POSITION]),
    toU64Le(params.amount),
  ]);

  return new TransactionInstruction({
    keys,
    programId,
    data,
  });
}

/**
 * Build ActivateMarket instruction
 * Authority: Pending → Active
 *
 * Accounts:
 * 0. authority (signer)
 * 1. exchange (readonly)
 * 2. market (mut)
 *
 * Data: [discriminator]
 */
export function buildActivateMarketIx(
  params: ActivateMarketParams,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  const [exchange] = getExchangePda(programId);
  const [market] = getMarketPda(params.marketId, programId);

  const keys: AccountMeta[] = [
    signerMut(params.authority),
    readonly(exchange),
    writable(market),
  ];

  const data = Buffer.from([INSTRUCTION.ACTIVATE_MARKET]);

  return new TransactionInstruction({
    keys,
    programId,
    data,
  });
}

/**
 * Build MatchOrdersMulti instruction
 * Match taker against up to 5 makers
 *
 * Accounts (13 fixed + 5 per maker):
 * 0. operator (signer)
 * 1. exchange (readonly)
 * 2. market (readonly)
 * 3. taker_order_status (mut)
 * 4. taker_nonce (readonly)
 * 5. taker_position (mut)
 * 6. base_mint (readonly)
 * 7. quote_mint (readonly)
 * 8. taker_position_base_ata (mut)
 * 9. taker_position_quote_ata (mut)
 * 10. token_2022_program (readonly)
 * 11. system_program (readonly)
 * 12. instructions_sysvar (readonly)
 *
 * Per Maker (5 each):
 * - maker_order_status (mut)
 * - maker_nonce (readonly)
 * - maker_position (mut)
 * - maker_position_base_ata (mut)
 * - maker_position_quote_ata (mut)
 *
 * Data:
 * [discriminator, taker_order_hash (32), taker_compact_order (65), taker_sig (64),
 *  num_makers (u8), ...per_maker_data]
 * Per maker: [maker_order_hash (32), maker_compact_order (65), maker_sig (64), maker_fill_amount (u64)]
 */
export function buildMatchOrdersMultiIx(
  params: MatchOrdersMultiParams,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  if (params.makerOrders.length === 0) {
    throw new Error("At least one maker order is required");
  }
  if (params.makerOrders.length > 5) {
    throw new Error("Maximum 5 maker orders allowed");
  }
  if (params.makerOrders.length !== params.fillAmounts.length) {
    throw new Error("Fill amounts must match maker orders count");
  }

  const [exchange] = getExchangePda(programId);
  const takerOrderHash = hashOrder(params.takerOrder);
  const [takerOrderStatus] = getOrderStatusPda(takerOrderHash, programId);
  const [takerNonce] = getUserNoncePda(params.takerOrder.maker, programId);
  const [takerPosition] = getPositionPda(
    params.takerOrder.maker,
    params.market,
    programId
  );
  const takerBaseAta = getConditionalTokenAta(params.baseMint, takerPosition);
  const takerQuoteAta = getConditionalTokenAta(params.quoteMint, takerPosition);

  const keys: AccountMeta[] = [
    signerMut(params.operator),
    readonly(exchange),
    readonly(params.market),
    writable(takerOrderStatus),
    readonly(takerNonce),
    writable(takerPosition),
    readonly(params.baseMint),
    readonly(params.quoteMint),
    writable(takerBaseAta),
    writable(takerQuoteAta),
    readonly(TOKEN_2022_PROGRAM_ID),
    readonly(SYSTEM_PROGRAM_ID),
    readonly(INSTRUCTIONS_SYSVAR_ID),
  ];

  // Add maker accounts
  for (const makerOrder of params.makerOrders) {
    const makerOrderHash = hashOrder(makerOrder);
    const [makerOrderStatus] = getOrderStatusPda(makerOrderHash, programId);
    const [makerNonce] = getUserNoncePda(makerOrder.maker, programId);
    const [makerPosition] = getPositionPda(
      makerOrder.maker,
      params.market,
      programId
    );
    const makerBaseAta = getConditionalTokenAta(params.baseMint, makerPosition);
    const makerQuoteAta = getConditionalTokenAta(
      params.quoteMint,
      makerPosition
    );

    keys.push(writable(makerOrderStatus));
    keys.push(readonly(makerNonce));
    keys.push(writable(makerPosition));
    keys.push(writable(makerBaseAta));
    keys.push(writable(makerQuoteAta));
  }

  // Build data
  const dataBuffers: Buffer[] = [
    Buffer.from([INSTRUCTION.MATCH_ORDERS_MULTI]),
    takerOrderHash,
    serializeCompactOrder({
      nonce: params.takerOrder.nonce,
      maker: params.takerOrder.maker,
      side: params.takerOrder.side,
      makerAmount: params.takerOrder.makerAmount,
      takerAmount: params.takerOrder.takerAmount,
      expiration: params.takerOrder.expiration,
    }),
    params.takerOrder.signature,
    toU8(params.makerOrders.length),
  ];

  // Add maker data
  for (let i = 0; i < params.makerOrders.length; i++) {
    const makerOrder = params.makerOrders[i];
    const makerOrderHash = hashOrder(makerOrder);

    dataBuffers.push(makerOrderHash);
    dataBuffers.push(
      serializeCompactOrder({
        nonce: makerOrder.nonce,
        maker: makerOrder.maker,
        side: makerOrder.side,
        makerAmount: makerOrder.makerAmount,
        takerAmount: makerOrder.takerAmount,
        expiration: makerOrder.expiration,
      })
    );
    dataBuffers.push(makerOrder.signature);
    dataBuffers.push(toU64Le(params.fillAmounts[i]));
  }

  const data = Buffer.concat(dataBuffers);

  return new TransactionInstruction({
    keys,
    programId,
    data,
  });
}
