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
} from "./constants";
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
  SetAuthorityParams,
  CreateOrderbookParams,
  SignedOrder,
  OutcomeMetadata,
} from "./types";
import {
  getExchangePda,
  getMarketPda,
  getVaultPda,
  getMintAuthorityPda,
  getAllConditionalMintPdas,
  getOrderStatusPda,
  getUserNoncePda,
  getPositionPda,
  getOrderbookPda,
  getAltPda,
} from "./pda";
import {
  toU8,
  toU64Le,
  serializeString,
  getConditionalTokenAta,
  getDepositTokenAta,
  validateOutcomes,
} from "./utils";
import { hashOrder, serializeSignedOrder, serializeOrder, signedOrderToOrder } from "./orders";

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

function signerMut(pubkey: PublicKey): AccountMeta {
  return { pubkey, isSigner: true, isWritable: true };
}

function writable(pubkey: PublicKey): AccountMeta {
  return { pubkey, isSigner: false, isWritable: true };
}

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
 *
 * Accounts:
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
    signerMut(params.authority),
    writable(market),
    readonly(params.depositMint),
    writable(vault),
    readonly(mintAuthority),
    readonly(TOKEN_PROGRAM_ID),
    readonly(TOKEN_2022_PROGRAM_ID),
    readonly(SYSTEM_PROGRAM_ID),
  ];

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
 *
 * Accounts:
 * 0. user (signer)
 * 1. exchange
 * 2. market
 * 3. deposit_mint
 * 4. vault
 * 5. user_deposit_ata
 * 6. position
 * 7. position_collateral_ata
 * 8. mint_authority
 * 9. token_program
 * 10. token_2022_program
 * 11. associated_token_program
 * 12. system_program
 * Remaining: [conditional_mint[i], position_conditional_ata[i]]
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
    signerMut(params.user),
    readonly(exchange),
    readonly(params.market),
    readonly(params.depositMint),
    writable(vault),
    writable(userDepositAta),
    writable(position),
    writable(positionCollateralAta),
    readonly(mintAuthority),
    readonly(TOKEN_PROGRAM_ID),
    readonly(TOKEN_2022_PROGRAM_ID),
    readonly(ASSOCIATED_TOKEN_PROGRAM_ID),
    readonly(SYSTEM_PROGRAM_ID),
  ];

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
 *
 * Accounts:
 * 0. user (signer)
 * 1. exchange
 * 2. market
 * 3. deposit_mint
 * 4. vault
 * 5. position
 * 6. user_deposit_ata
 * 7. mint_authority
 * 8. token_program
 * 9. token_2022_program
 * Remaining: [conditional_mint[i], position_conditional_ata[i]]
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
    signerMut(params.user),
    readonly(exchange),
    readonly(params.market),
    readonly(params.depositMint),
    writable(vault),
    writable(position),
    writable(userDepositAta),
    readonly(mintAuthority),
    readonly(TOKEN_PROGRAM_ID),
    readonly(TOKEN_2022_PROGRAM_ID),
  ];

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
 *
 * Accounts:
 * 0. maker (signer, mut)
 * 1. market (readonly)
 * 2. order_status (mut)
 * 3. system_program (readonly)
 *
 * Data: [discriminator, order_hash (32), signed_order (225)]
 */
export function buildCancelOrderIx(
  maker: PublicKey,
  order: SignedOrder,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  const orderHash = hashOrder(order);
  const [orderStatus] = getOrderStatusPda(orderHash, programId);

  const keys: AccountMeta[] = [
    signerMut(maker),
    readonly(order.market),
    writable(orderStatus),
    readonly(SYSTEM_PROGRAM_ID),
  ];

  const data = Buffer.concat([
    Buffer.from([INSTRUCTION.CANCEL_ORDER]),
    orderHash,
    serializeSignedOrder(order),
  ]);

  return new TransactionInstruction({
    keys,
    programId,
    data,
  });
}

/**
 * Build IncrementNonce instruction
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
 *
 * Accounts:
 * 0. user (signer)
 * 1. market
 * 2. deposit_mint
 * 3. vault
 * 4. winning_conditional_mint
 * 5. position
 * 6. position_conditional_ata
 * 7. user_deposit_ata
 * 8. mint_authority
 * 9. token_program
 * 10. token_2022_program
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
  const [winningMint] = getAllConditionalMintPdas(
    params.market,
    params.depositMint,
    winningOutcome + 1,
    programId
  )[winningOutcome];
  const positionWinningAta = getConditionalTokenAta(winningMint, position);
  const userDepositAta = getDepositTokenAta(params.depositMint, params.user);

  const keys: AccountMeta[] = [
    signerMut(params.user),
    readonly(params.market),
    readonly(params.depositMint),
    writable(vault),
    writable(winningMint),
    writable(position),
    writable(positionWinningAta),
    writable(userDepositAta),
    readonly(mintAuthority),
    readonly(TOKEN_PROGRAM_ID),
    readonly(TOKEN_2022_PROGRAM_ID),
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
 *
 * Accounts:
 * 0. user (signer, mut)
 * 1. market (readonly)
 * 2. position (mut)
 * 3. mint (readonly)
 * 4. position_ata (mut)
 * 5. user_ata (mut)
 * 6. token_program (readonly)
 *
 * Data: [discriminator, amount (u64), outcome_index (u8)]
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
    readonly(params.market),
    writable(position),
    readonly(params.mint),
    writable(positionAta),
    writable(userAta),
    readonly(tokenProgram),
  ];

  const data = Buffer.concat([
    Buffer.from([INSTRUCTION.WITHDRAW_FROM_POSITION]),
    toU64Le(params.amount),
    toU8(params.outcomeIndex),
  ]);

  return new TransactionInstruction({
    keys,
    programId,
    data,
  });
}

/**
 * Build ActivateMarket instruction
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
 *
 * Dynamic accounts based on full_fill_bitmask:
 * 0. operator (signer)
 * 1. exchange (readonly)
 * 2. market (readonly)
 * [taker_order_status if bit7=0] (mut)
 * taker_nonce (readonly)
 * taker_position (mut)
 * base_mint (readonly)
 * quote_mint (readonly)
 * taker_base_ata (mut)
 * taker_quote_ata (mut)
 * token_2022_program (readonly)
 * system_program (readonly)
 * Per maker:
 *   [maker_order_status if bit_i=0] (mut)
 *   maker_nonce (readonly)
 *   maker_position (mut)
 *   maker_base_ata (mut)
 *   maker_quote_ata (mut)
 *
 * Data:
 * [0] discriminator
 * [1..30] taker Order (29 bytes)
 * [30..94] taker_signature (64 bytes)
 * [94] num_makers
 * [95] full_fill_bitmask
 * Per maker (109 bytes):
 *   [+0..+29] maker Order (29)
 *   [+29..+93] maker_signature (64)
 *   [+93..+101] maker_fill_amount (8)
 *   [+101..+109] taker_fill_amount (8)
 */
export function buildMatchOrdersMultiIx(
  params: MatchOrdersMultiParams,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  if (params.makerOrders.length === 0) {
    throw new Error("At least one maker order is required");
  }
  if (params.makerOrders.length > 3) {
    throw new Error("Maximum 3 maker orders allowed");
  }
  if (params.makerOrders.length !== params.makerFillAmounts.length) {
    throw new Error("Maker fill amounts must match maker orders count");
  }
  if (params.makerOrders.length !== params.takerFillAmounts.length) {
    throw new Error("Taker fill amounts must match maker orders count");
  }

  const [exchange] = getExchangePda(programId);
  const takerOrderHash = hashOrder(params.takerOrder);
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
  ];

  // Taker order status if not fully filled (bit 7 = 0)
  const takerFullFill = (params.fullFillBitmask & 0x80) !== 0;
  if (!takerFullFill) {
    const [takerOrderStatus] = getOrderStatusPda(takerOrderHash, programId);
    keys.push(writable(takerOrderStatus));
  }

  keys.push(readonly(takerNonce));
  keys.push(writable(takerPosition));
  keys.push(readonly(params.baseMint));
  keys.push(readonly(params.quoteMint));
  keys.push(writable(takerBaseAta));
  keys.push(writable(takerQuoteAta));
  keys.push(readonly(TOKEN_2022_PROGRAM_ID));
  keys.push(readonly(SYSTEM_PROGRAM_ID));

  // Add maker accounts
  for (let i = 0; i < params.makerOrders.length; i++) {
    const makerOrder = params.makerOrders[i];
    const makerFullFill = (params.fullFillBitmask & (1 << i)) !== 0;

    if (!makerFullFill) {
      const makerOrderHash = hashOrder(makerOrder);
      const [makerOrderStatus] = getOrderStatusPda(makerOrderHash, programId);
      keys.push(writable(makerOrderStatus));
    }

    const [makerNonce] = getUserNoncePda(makerOrder.maker, programId);
    const [makerPosition] = getPositionPda(
      makerOrder.maker,
      params.market,
      programId
    );
    const makerBaseAta = getConditionalTokenAta(params.baseMint, makerPosition);
    const makerQuoteAta = getConditionalTokenAta(params.quoteMint, makerPosition);

    keys.push(readonly(makerNonce));
    keys.push(writable(makerPosition));
    keys.push(writable(makerBaseAta));
    keys.push(writable(makerQuoteAta));
  }

  // Build data
  const takerCompact = signedOrderToOrder(params.takerOrder);
  const dataBuffers: Buffer[] = [
    Buffer.from([INSTRUCTION.MATCH_ORDERS_MULTI]),
    serializeOrder(takerCompact),
    params.takerOrder.signature,
    toU8(params.makerOrders.length),
    toU8(params.fullFillBitmask),
  ];

  // Add maker data
  for (let i = 0; i < params.makerOrders.length; i++) {
    const makerOrder = params.makerOrders[i];
    const makerCompact = signedOrderToOrder(makerOrder);

    dataBuffers.push(serializeOrder(makerCompact));
    dataBuffers.push(makerOrder.signature);
    dataBuffers.push(toU64Le(params.makerFillAmounts[i]));
    dataBuffers.push(toU64Le(params.takerFillAmounts[i]));
  }

  const data = Buffer.concat(dataBuffers);

  return new TransactionInstruction({
    keys,
    programId,
    data,
  });
}

/**
 * Build SetAuthority instruction
 *
 * Accounts:
 * 0. authority (signer)
 * 1. exchange (mut)
 *
 * Data: [discriminator, new_authority (32)]
 */
export function buildSetAuthorityIx(
  params: SetAuthorityParams,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  const [exchange] = getExchangePda(programId);

  const keys: AccountMeta[] = [
    signerMut(params.currentAuthority),
    writable(exchange),
  ];

  const data = Buffer.concat([
    Buffer.from([INSTRUCTION.SET_AUTHORITY]),
    params.newAuthority.toBuffer(),
  ]);

  return new TransactionInstruction({
    keys,
    programId,
    data,
  });
}

/**
 * Build CreateOrderbook instruction
 *
 * Accounts:
 * 0. payer (signer, mut)
 * 1. authority (signer)
 * 2. exchange (readonly)
 * 3. market (readonly)
 * 4. mint_a (readonly)
 * 5. mint_b (readonly)
 * 6. orderbook (mut)
 * 7. address_lookup_table (mut)
 * 8. system_program (readonly)
 *
 * Data: [discriminator, recent_slot (u64)]
 */
export function buildCreateOrderbookIx(
  params: CreateOrderbookParams,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  const [exchange] = getExchangePda(programId);
  const [orderbook] = getOrderbookPda(params.mintA, params.mintB, programId);
  const [alt] = getAltPda(orderbook, params.recentSlot);

  const keys: AccountMeta[] = [
    signerMut(params.payer),
    signerMut(params.payer), // authority = payer for now
    readonly(exchange),
    readonly(params.market),
    readonly(params.mintA),
    readonly(params.mintB),
    writable(orderbook),
    writable(alt),
    readonly(SYSTEM_PROGRAM_ID),
  ];

  const data = Buffer.concat([
    Buffer.from([INSTRUCTION.CREATE_ORDERBOOK]),
    toU64Le(params.recentSlot),
  ]);

  return new TransactionInstruction({
    keys,
    programId,
    data,
  });
}
