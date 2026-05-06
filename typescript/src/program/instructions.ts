import {
  PublicKey,
  Transaction,
  TransactionInstruction,
  AccountMeta,
} from "@solana/web3.js";
import {
  INSTRUCTION,
  SYSTEM_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  TOKEN_2022_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  ALT_PROGRAM_ID,
  MAX_MAKERS,
  MAX_OUTCOMES,
  MIN_OUTCOMES,
} from "./constants";
import { PROGRAM_ID } from "../env";
import {
  InitializeParams,
  CreateMarketParams,
  AddDepositMintParams,
  BuildDepositParams,
  BuildMergeParams,
  SettleMarketParams,
  RedeemWinningsParams,
  WithdrawFromPositionParams,
  ActivateMarketParams,
  MatchOrdersMultiParams,
  SetAuthorityParams,
  SetManagerParams,
  CreateOrderbookParams,
  WhitelistDepositTokenParams,
  DepositToGlobalParams,
  DepositToGlobalAltContext,
  GlobalToMarketDepositParams,
  InitPositionTokensParams,
  ExtendPositionTokensParams,
  DepositAndSwapParams,
  WithdrawFromGlobalParams,
  ClosePositionAltParams,
  CloseOrderStatusParams,
  ClosePositionTokenAccountsParams,
  CloseOrderbookAltParams,
  CloseOrderbookParams,
  SignedOrder,
  OutcomeMetadata,
  OrderSide,
} from "./types";
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
  getConditionTombstonePda,
  getOrderbookPda,
  getAltPda,
  getGlobalDepositTokenPda,
  getUserGlobalDepositPda,
  getPositionAltPda,
} from "./pda";
import {
  toU8,
  toU32Le,
  toU64Le,
  serializeString,
  getConditionalTokenAta,
  getDepositTokenAta,
  validateOutcomes,
  deriveConditionId,
} from "./utils";
import { hashOrder, serializeSignedOrder, serializeOrder, signedOrderToOrder } from "./orders";
import { ProgramSdkError } from "./error";

// ============================================================================
// HELPER FUNCTIONS
// ============================================================================

function signerMut(pubkey: PublicKey): AccountMeta {
  return { pubkey, isSigner: true, isWritable: true };
}

function signer(pubkey: PublicKey): AccountMeta {
  return { pubkey, isSigner: true, isWritable: false };
}

function writable(pubkey: PublicKey): AccountMeta {
  return { pubkey, isSigner: false, isWritable: true };
}

function readonly(pubkey: PublicKey): AccountMeta {
  return { pubkey, isSigner: false, isWritable: false };
}

interface OrderbookMintInput {
  mint: PublicKey;
  depositMint: PublicKey;
  outcomeIndex: number;
  isBase: boolean;
}

interface CanonicalOrderbookMints {
  mintA: OrderbookMintInput;
  mintB: OrderbookMintInput;
  baseIndex: number;
}

function canonicalOrderbookMints(params: CreateOrderbookParams): CanonicalOrderbookMints {
  if (params.baseIndex > 1) {
    throw ProgramSdkError.invalidOutcomeIndex(params.baseIndex, 1);
  }
  if (params.mintA.equals(params.mintB)) {
    throw ProgramSdkError.invalidMintOrder();
  }

  const left: OrderbookMintInput = {
    mint: params.mintA,
    depositMint: params.mintADepositMint,
    outcomeIndex: params.mintAOutcomeIndex,
    isBase: params.baseIndex === 0,
  };
  const right: OrderbookMintInput = {
    mint: params.mintB,
    depositMint: params.mintBDepositMint,
    outcomeIndex: params.mintBOutcomeIndex,
    isBase: params.baseIndex === 1,
  };

  const [mintA, mintB] =
    Buffer.compare(left.mint.toBuffer(), right.mint.toBuffer()) < 0
      ? [left, right]
      : [right, left];

  return {
    mintA,
    mintB,
    baseIndex: mintA.isBase ? 0 : 1,
  };
}

function validatePayoutNumerators(payoutNumerators: number[]): void {
  const count = payoutNumerators.length;
  if (count < MIN_OUTCOMES || count > MAX_OUTCOMES) {
    throw ProgramSdkError.invalidOutcomeCount(count);
  }

  let denominator = 0n;
  for (const numerator of payoutNumerators) {
    if (
      !Number.isInteger(numerator) ||
      numerator < 0 ||
      numerator > 0xffffffff
    ) {
      throw ProgramSdkError.payoutVectorExceedsU32();
    }
    denominator += BigInt(numerator);
    if (denominator > 0xffffffffn) {
      throw ProgramSdkError.overflow("Payout denominator overflow");
    }
  }

  if (denominator === 0n) {
    throw ProgramSdkError.invalidPayoutNumerators();
  }
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
 * 0. manager (signer, mut) - Must be exchange manager
 * 1. exchange (mut) - Exchange PDA
 * 2. market (mut) - Market PDA
 * 3. system_program (readonly)
 * 4. condition_tombstone (mut) - Condition uniqueness PDA
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
  const conditionId = deriveConditionId(
    params.oracle,
    params.questionId,
    params.numOutcomes
  );
  const [conditionTombstone] = getConditionTombstonePda(conditionId, programId);

  const keys: AccountMeta[] = [
    signerMut(params.manager),
    writable(exchange),
    writable(market),
    readonly(SYSTEM_PROGRAM_ID),
    writable(conditionTombstone),
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
 * 0. manager (signer)
 * 1. exchange
 * 2. market
 * 3. deposit_mint
 * 4. vault
 * 5. mint_authority
 * 6. token_program (SPL Token)
 * 7. token_2022_program
 * 8. system_program
 * 9. global_deposit_token
 * 10+ conditional_mints[0..num_outcomes]
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
    throw ProgramSdkError.invalidOutcomeCount(params.outcomeMetadata.length);
  }

  const [vault] = getVaultPda(params.depositMint, market, programId);
  const [mintAuthority] = getMintAuthorityPda(market, programId);
  const conditionalMints = getAllConditionalMintPdas(
    market,
    params.depositMint,
    numOutcomes,
    programId
  );

  const [exchange] = getExchangePda(programId);
  const [globalDepositToken] = getGlobalDepositTokenPda(params.depositMint, programId);

  const keys: AccountMeta[] = [
    signerMut(params.manager),
    readonly(exchange),
    readonly(market),
    readonly(params.depositMint),
    writable(vault),
    readonly(mintAuthority),
    readonly(TOKEN_PROGRAM_ID),
    readonly(TOKEN_2022_PROGRAM_ID),
    readonly(SYSTEM_PROGRAM_ID),
    readonly(globalDepositToken),
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
 * 7. mint_authority
 * 8. token_program
 * 9. token_2022_program
 * 10. associated_token_program
 * 11. system_program
 * Remaining: [conditional_mint[i], position_conditional_ata[i]]
 *
 * Data: [discriminator, amount (u64)]
 */
export function buildDepositIx(
  params: BuildDepositParams,
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
    writable(userDepositAta),
    writable(position),
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
export function buildMergeIx(
  params: BuildMergeParams,
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
 * 0. operator (signer, mut)
 * 1. exchange (readonly)
 * 2. market (readonly)
 * 3. order_status (mut)
 *
 * Data: [discriminator, order_hash (32), signed_order (233)]
 */
export function buildCancelOrderIx(
  operator: PublicKey,
  market: PublicKey,
  order: SignedOrder,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  const orderHash = hashOrder(order);
  const [exchange] = getExchangePda(programId);
  const [orderStatus] = getOrderStatusPda(orderHash, programId);

  const keys: AccountMeta[] = [
    signerMut(operator),
    readonly(exchange),
    readonly(market),
    writable(orderStatus),
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
 * 3. exchange (readonly)
 *
 * Data: [discriminator]
 */
export function buildIncrementNonceIx(
  user: PublicKey,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  const [userNonce] = getUserNoncePda(user, programId);
  const [exchange] = getExchangePda(programId);

  const keys: AccountMeta[] = [
    signerMut(user),
    writable(userNonce),
    readonly(SYSTEM_PROGRAM_ID),
    readonly(exchange),
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
 * Data: [discriminator, payout_numerator_0 (u32), ...]
 */
export function buildSettleMarketIx(
  params: SettleMarketParams,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  validatePayoutNumerators(params.payoutNumerators);

  const [exchange] = getExchangePda(programId);
  const [market] = getMarketPda(params.marketId, programId);

  const keys: AccountMeta[] = [
    signer(params.oracle),
    readonly(exchange),
    writable(market),
  ];

  const data = Buffer.concat([
    Buffer.from([INSTRUCTION.SETTLE_MARKET]),
    ...params.payoutNumerators.map(toU32Le),
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
 * 11. exchange
 *
 * Data: [discriminator, amount (u64), outcome_index (u8)]
 */
export function buildRedeemWinningsIx(
  params: RedeemWinningsParams,
  outcomeIndex: number,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  if (!Number.isInteger(outcomeIndex) || outcomeIndex < 0 || outcomeIndex > 0xff) {
    throw ProgramSdkError.invalidOutcomeIndex(outcomeIndex, 0xff);
  }

  const [exchange] = getExchangePda(programId);
  const [vault] = getVaultPda(params.depositMint, params.market, programId);
  const [mintAuthority] = getMintAuthorityPda(params.market, programId);
  const [position] = getPositionPda(params.user, params.market, programId);
  const [conditionalMint] = getConditionalMintPda(
    params.market,
    params.depositMint,
    outcomeIndex,
    programId
  );
  const positionConditionalAta = getConditionalTokenAta(conditionalMint, position);
  const userDepositAta = getDepositTokenAta(params.depositMint, params.user);

  const keys: AccountMeta[] = [
    signerMut(params.user),
    readonly(params.market),
    readonly(params.depositMint),
    writable(vault),
    writable(conditionalMint),
    readonly(position),
    writable(positionConditionalAta),
    writable(userDepositAta),
    readonly(mintAuthority),
    readonly(TOKEN_PROGRAM_ID),
    readonly(TOKEN_2022_PROGRAM_ID),
    readonly(exchange),
  ];

  const data = Buffer.concat([
    Buffer.from([INSTRUCTION.REDEEM_WINNINGS]),
    toU64Le(params.amount),
    toU8(outcomeIndex),
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
 * 7. exchange (readonly)
 *
 * Data: [discriminator, amount (u64), outcome_index (u8)]
 */
export function buildWithdrawFromPositionIx(
  params: WithdrawFromPositionParams,
  isToken2022: boolean,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  const [exchange] = getExchangePda(programId);
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
    readonly(exchange),
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
 * 0. manager (signer)
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
    signerMut(params.manager),
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
 * 3. orderbook (readonly)
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
 * [1..38] taker Order (37 bytes)
 * [38..102] taker_signature (64 bytes)
 * [102] num_makers
 * [103] full_fill_bitmask
 * Per maker (117 bytes):
 *   [+0..+37] maker Order (37)
 *   [+37..+101] maker_signature (64)
 *   [+101..+109] maker_fill_amount (8)
 *   [+109..+117] taker_fill_amount (8)
 */
export function buildMatchOrdersMultiIx(
  params: MatchOrdersMultiParams,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  if (params.makerOrders.length === 0) {
    throw ProgramSdkError.missingField("makers");
  }
  if (params.makerOrders.length > MAX_MAKERS) {
    throw ProgramSdkError.tooManyMakers(params.makerOrders.length);
  }
  if (params.makerOrders.length !== params.makerFillAmounts.length) {
    throw ProgramSdkError.invalidDataLength("makerFillAmounts", params.makerOrders.length, params.makerFillAmounts.length);
  }
  if (params.makerOrders.length !== params.takerFillAmounts.length) {
    throw ProgramSdkError.invalidDataLength("takerFillAmounts", params.makerOrders.length, params.takerFillAmounts.length);
  }

  const [exchange] = getExchangePda(programId);
  const [orderbook] = getOrderbookPda(params.baseMint, params.quoteMint, programId);
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
    readonly(orderbook),
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
 * 0. manager (signer, mut)
 * 1. market (readonly)
 * 2. mint_a (readonly, canonical order)
 * 3. mint_b (readonly, canonical order)
 * 4. orderbook (mut)
 * 5. lookup_table (mut)
 * 6. exchange (readonly)
 * 7. alt_program (readonly)
 * 8. system_program (readonly)
 * 9. mint_a_deposit_mint (readonly, canonical order)
 * 10. mint_b_deposit_mint (readonly, canonical order)
 *
 * Data: [discriminator, recent_slot (u64), base_index (u8), mint_a_outcome_index (u8), mint_b_outcome_index (u8)]
 */
export function buildCreateOrderbookIx(
  params: CreateOrderbookParams,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  const canonical = canonicalOrderbookMints(params);
  const [exchange] = getExchangePda(programId);
  const [orderbook] = getOrderbookPda(canonical.mintA.mint, canonical.mintB.mint, programId);
  const [alt] = getAltPda(orderbook, params.recentSlot);

  const keys: AccountMeta[] = [
    signerMut(params.manager),
    readonly(params.market),
    readonly(canonical.mintA.mint),
    readonly(canonical.mintB.mint),
    writable(orderbook),
    writable(alt),
    readonly(exchange),
    readonly(ALT_PROGRAM_ID),
    readonly(SYSTEM_PROGRAM_ID),
    readonly(canonical.mintA.depositMint),
    readonly(canonical.mintB.depositMint),
  ];

  const data = Buffer.concat([
    Buffer.from([INSTRUCTION.CREATE_ORDERBOOK]),
    toU64Le(params.recentSlot),
    toU8(canonical.baseIndex),
    toU8(canonical.mintA.outcomeIndex),
    toU8(canonical.mintB.outcomeIndex),
  ]);

  return new TransactionInstruction({
    keys,
    programId,
    data,
  });
}

/**
 * Build SetManager instruction
 *
 * Accounts:
 * 0. authority (signer)
 * 1. exchange (mut)
 *
 * Data: [discriminator, new_manager (32)]
 */
export function buildSetManagerIx(
  params: SetManagerParams,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  const [exchange] = getExchangePda(programId);

  const keys: AccountMeta[] = [
    signerMut(params.authority),
    writable(exchange),
  ];

  const data = Buffer.concat([
    Buffer.from([INSTRUCTION.SET_MANAGER]),
    params.newManager.toBuffer(),
  ]);

  return new TransactionInstruction({
    keys,
    programId,
    data,
  });
}

/**
 * Build WhitelistDepositToken instruction
 *
 * Accounts:
 * 0. authority (signer, mut)
 * 1. exchange (readonly)
 * 2. mint (readonly)
 * 3. global_deposit_token (mut)
 * 4. system_program (readonly)
 *
 * Data: [discriminator]
 */
export function buildWhitelistDepositTokenIx(
  params: WhitelistDepositTokenParams,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  const [exchange] = getExchangePda(programId);
  const [globalDepositToken] = getGlobalDepositTokenPda(params.mint, programId);

  const keys: AccountMeta[] = [
    signerMut(params.authority),
    writable(exchange),
    readonly(params.mint),
    writable(globalDepositToken),
    readonly(SYSTEM_PROGRAM_ID),
  ];

  return new TransactionInstruction({
    keys,
    programId,
    data: Buffer.from([INSTRUCTION.WHITELIST_DEPOSIT_TOKEN]),
  });
}

/**
 * Build DepositToGlobal instruction
 *
 * Accounts:
 * 0. user (signer, mut)
 * 1. global_deposit_token (readonly)
 * 2. mint (readonly)
 * 3. user_global_deposit (mut)
 * 4. user_token_account (mut)
 * 5. token_program (readonly)
 * 6. system_program (readonly)
 * 7. exchange (readonly)
 * Optional ALT accounts:
 * 8. user_nonce (readonly)
 * 9. lookup_table (mut)
 * 10. alt_program (readonly)
 *
 * Data: [discriminator, amount (u64), recent_slot (u64 if creating ALT)]
 */
export function buildDepositToGlobalIx(
  params: DepositToGlobalParams,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  return buildDepositToGlobalIxInner(params, undefined, programId);
}

/**
 * Build DepositToGlobal instruction with user deposit ALT create/extend accounts.
 */
export function buildDepositToGlobalIxWithAlt(
  params: DepositToGlobalParams,
  altContext: DepositToGlobalAltContext,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  return buildDepositToGlobalIxInner(params, altContext, programId);
}

function buildDepositToGlobalIxInner(
  params: DepositToGlobalParams,
  altContext: DepositToGlobalAltContext | undefined,
  programId: PublicKey
): TransactionInstruction {
  const [globalDepositToken] = getGlobalDepositTokenPda(params.mint, programId);
  const [userGlobalDeposit] = getUserGlobalDepositPda(params.user, params.mint, programId);
  const [exchange] = getExchangePda(programId);
  const userTokenAccount = getDepositTokenAta(params.mint, params.user);

  const keys: AccountMeta[] = [
    signerMut(params.user),
    readonly(globalDepositToken),
    readonly(params.mint),
    writable(userGlobalDeposit),
    writable(userTokenAccount),
    readonly(TOKEN_PROGRAM_ID),
    readonly(SYSTEM_PROGRAM_ID),
    readonly(exchange),
  ];

  const dataBuffers = [
    Buffer.from([INSTRUCTION.DEPOSIT_TO_GLOBAL]),
    toU64Le(params.amount),
  ];

  if (altContext !== undefined) {
    const [userNonce] = getUserNoncePda(params.user, programId);
    const lookupTable =
      altContext.kind === "create"
        ? getAltPda(userNonce, altContext.recentSlot)[0]
        : altContext.lookupTable;

    if (altContext.kind === "create") {
      dataBuffers.push(toU64Le(altContext.recentSlot));
    }

    keys.push(readonly(userNonce));
    keys.push(writable(lookupTable));
    keys.push(readonly(ALT_PROGRAM_ID));
  }

  return new TransactionInstruction({
    keys,
    programId,
    data: Buffer.concat(dataBuffers),
  });
}

/**
 * Build GlobalToMarketDeposit instruction
 *
 * Accounts:
 * 0. user (signer, mut)
 * 1. exchange (readonly)
 * 2. market (readonly)
 * 3. deposit_mint (readonly)
 * 4. vault (mut)
 * 5. global_deposit_token (readonly)
 * 6. user_global_deposit (mut)
 * 7. position (mut)
 * 8. mint_authority (readonly)
 * 9. token_program (readonly)
 * 10. token_2022_program (readonly)
 * 11. ata_program (readonly)
 * 12. system_program (readonly)
 * + per outcome: conditional_mint[i] (mut), position_conditional_ata[i] (mut)
 *
 * Data: [discriminator, amount (u64)]
 */
export function buildGlobalToMarketDepositIx(
  params: GlobalToMarketDepositParams,
  numOutcomes: number,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  const [exchange] = getExchangePda(programId);
  const [vault] = getVaultPda(params.depositMint, params.market, programId);
  const [globalDepositToken] = getGlobalDepositTokenPda(params.depositMint, programId);
  const [userGlobalDeposit] = getUserGlobalDepositPda(params.user, params.depositMint, programId);
  const [position] = getPositionPda(params.user, params.market, programId);
  const [mintAuthority] = getMintAuthorityPda(params.market, programId);

  const keys: AccountMeta[] = [
    signerMut(params.user),
    readonly(exchange),
    readonly(params.market),
    readonly(params.depositMint),
    writable(vault),
    readonly(globalDepositToken),
    writable(userGlobalDeposit),
    writable(position),
    readonly(mintAuthority),
    readonly(TOKEN_PROGRAM_ID),
    readonly(TOKEN_2022_PROGRAM_ID),
    readonly(ASSOCIATED_TOKEN_PROGRAM_ID),
    readonly(SYSTEM_PROGRAM_ID),
  ];

  for (let i = 0; i < numOutcomes; i += 1) {
    const [conditionalMint] = getAllConditionalMintPdas(
      params.market,
      params.depositMint,
      numOutcomes,
      programId
    )[i];
    keys.push(writable(conditionalMint));
    keys.push(writable(getConditionalTokenAta(conditionalMint, position)));
  }

  const data = Buffer.concat([
    Buffer.from([INSTRUCTION.GLOBAL_TO_MARKET_DEPOSIT]),
    toU64Le(params.amount),
  ]);

  return new TransactionInstruction({
    keys,
    programId,
    data,
  });
}

/**
 * Build InitPositionTokens instruction
 *
 * Accounts:
 * 0. payer (signer, mut)
 * 1. user (readonly)
 * 2. exchange (readonly)
 * 3. market (readonly)
 * 4. position (mut)
 * 5. lookup_table (mut)
 * 6. mint_authority (readonly)
 * 7. token_2022_program (readonly)
 * 8. ata_program (readonly)
 * 9. alt_program (readonly)
 * 10. system_program (readonly)
 * + per deposit_mint: deposit_mint, vault, gdt, conditional_mint/position_ata pairs
 *
 * Data: [discriminator, recent_slot (u64)]
 */
export function buildInitPositionTokensIx(
  params: InitPositionTokensParams,
  numOutcomes: number,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  const [exchange] = getExchangePda(programId);
  const [position] = getPositionPda(params.user, params.market, programId);
  const [lookupTable] = getPositionAltPda(position, params.recentSlot);
  const [mintAuthority] = getMintAuthorityPda(params.market, programId);

  const keys: AccountMeta[] = [
    signerMut(params.payer),
    readonly(params.user),
    readonly(exchange),
    readonly(params.market),
    writable(position),
    writable(lookupTable),
    readonly(mintAuthority),
    readonly(TOKEN_2022_PROGRAM_ID),
    readonly(ASSOCIATED_TOKEN_PROGRAM_ID),
    readonly(ALT_PROGRAM_ID),
    readonly(SYSTEM_PROGRAM_ID),
  ];

  for (const depositMint of params.depositMints) {
    const [vault] = getVaultPda(depositMint, params.market, programId);
    const [gdt] = getGlobalDepositTokenPda(depositMint, programId);
    keys.push(readonly(depositMint));
    keys.push(readonly(vault));
    keys.push(readonly(gdt));

    for (let i = 0; i < numOutcomes; i += 1) {
      const [conditionalMint] = getAllConditionalMintPdas(
        params.market,
        depositMint,
        numOutcomes,
        programId
      )[i];
      keys.push(readonly(conditionalMint));
      keys.push(writable(getConditionalTokenAta(conditionalMint, position)));
    }
  }

  const data = Buffer.concat([
    Buffer.from([INSTRUCTION.INIT_POSITION_TOKENS]),
    toU64Le(params.recentSlot),
    toU8(params.depositMints.length),
  ]);

  return new TransactionInstruction({
    keys,
    programId,
    data,
  });
}

/**
 * Build ExtendPositionTokens instruction
 *
 * Extends an existing position's lookup table with new deposit mints.
 *
 * Accounts:
 * 0. operator (signer, mut)
 * 1. user (readonly)
 * 2. exchange (readonly)
 * 3. market (readonly)
 * 4. position (readonly)
 * 5. lookup_table (mut)
 * 6. token_2022_program (readonly)
 * 7. ata_program (readonly)
 * 8. alt_program (readonly)
 * 9. system_program (readonly)
 * Per deposit_mint:
 *   deposit_mint (readonly), vault (readonly), gdt (readonly),
 *   per outcome: conditional_mint (readonly), position_ata (mut)
 *
 * Data: [discriminator, num_deposit_mints (u8)]
 */
export function buildExtendPositionTokensIx(
  params: ExtendPositionTokensParams,
  numOutcomes: number,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  if (params.depositMints.length === 0) {
    throw ProgramSdkError.missingField("deposit_mints");
  }

  const [exchange] = getExchangePda(programId);
  const [position] = getPositionPda(params.user, params.market, programId);

  const keys: AccountMeta[] = [
    signerMut(params.operator),
    readonly(params.user),
    readonly(exchange),
    readonly(params.market),
    readonly(position),
    writable(params.lookupTable),
    readonly(TOKEN_2022_PROGRAM_ID),
    readonly(ASSOCIATED_TOKEN_PROGRAM_ID),
    readonly(ALT_PROGRAM_ID),
    readonly(SYSTEM_PROGRAM_ID),
  ];

  for (const depositMint of params.depositMints) {
    const [vault] = getVaultPda(depositMint, params.market, programId);
    const [gdt] = getGlobalDepositTokenPda(depositMint, programId);
    keys.push(readonly(depositMint));
    keys.push(readonly(vault));
    keys.push(readonly(gdt));

    for (let i = 0; i < numOutcomes; i += 1) {
      const [conditionalMint] = getAllConditionalMintPdas(
        params.market,
        depositMint,
        numOutcomes,
        programId
      )[i];
      keys.push(readonly(conditionalMint));
      keys.push(writable(getConditionalTokenAta(conditionalMint, position)));
    }
  }

  const data = Buffer.from([
    INSTRUCTION.EXTEND_POSITION_TOKENS,
    params.depositMints.length,
  ]);

  return new TransactionInstruction({
    keys,
    programId,
    data,
  });
}

/**
 * Build DepositAndSwap instruction.
 * Supports a mix of global deposits and token swaps in a single instruction.
 */
export function buildDepositAndSwapIx(
  params: DepositAndSwapParams,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  validateOutcomes(params.numOutcomes);

  if (params.makers.length === 0) {
    throw ProgramSdkError.missingField("makers");
  }
  if (params.makers.length > MAX_MAKERS) {
    throw ProgramSdkError.tooManyMakers(params.makers.length);
  }

  const [exchange] = getExchangePda(programId);
  const [orderbook] = getOrderbookPda(params.baseMint, params.quoteMint, programId);
  const [mintAuthority] = getMintAuthorityPda(params.market, programId);
  const [takerNonce] = getUserNoncePda(params.takerOrder.maker, programId);
  const [takerPosition] = getPositionPda(params.takerOrder.maker, params.market, programId);
  const [receiveMint, giveMint] =
    params.takerOrder.side === OrderSide.BID
      ? [params.baseMint, params.quoteMint]
      : [params.quoteMint, params.baseMint];

  let fullFillBitmask = 0;
  let depositBitmask = 0;

  if (params.takerIsFullFill) {
    fullFillBitmask |= 0x80;
  }
  if (params.takerIsDeposit) {
    depositBitmask |= 0x80;
  }

  for (let i = 0; i < params.makers.length; i += 1) {
    const maker = params.makers[i];
    if (maker.isFullFill) {
      fullFillBitmask |= 1 << i;
    }
    if (maker.isDeposit) {
      depositBitmask |= 1 << i;
    }
  }

  const keys: AccountMeta[] = [
    signerMut(params.operator),
    readonly(exchange),
    readonly(params.market),
    readonly(orderbook),
    readonly(mintAuthority),
    readonly(TOKEN_PROGRAM_ID),
  ];

  if (!params.takerIsFullFill) {
    const takerOrderHash = hashOrder(params.takerOrder);
    const [takerOrderStatus] = getOrderStatusPda(takerOrderHash, programId);
    keys.push(writable(takerOrderStatus));
  }

  keys.push(readonly(takerNonce));
  keys.push(writable(takerPosition));
  keys.push(readonly(params.baseMint));
  keys.push(readonly(params.quoteMint));
  keys.push(writable(getConditionalTokenAta(receiveMint, takerPosition)));
  keys.push(writable(getConditionalTokenAta(giveMint, takerPosition)));
  keys.push(readonly(TOKEN_2022_PROGRAM_ID));
  keys.push(readonly(SYSTEM_PROGRAM_ID));

  if (params.takerIsDeposit) {
    const [vault] = getVaultPda(
      params.takerDepositMint,
      params.market,
      programId
    );
    const [globalDepositToken] = getGlobalDepositTokenPda(
      params.takerDepositMint,
      programId
    );
    const [takerGlobalDeposit] = getUserGlobalDepositPda(
      params.takerOrder.maker,
      params.takerDepositMint,
      programId
    );

    keys.push(readonly(params.takerDepositMint));
    keys.push(writable(vault));
    keys.push(readonly(globalDepositToken));
    keys.push(writable(takerGlobalDeposit));

    for (const [conditionalMint] of getAllConditionalMintPdas(
      params.market,
      params.takerDepositMint,
      params.numOutcomes,
      programId
    )) {
      keys.push(writable(conditionalMint));
      keys.push(writable(getConditionalTokenAta(conditionalMint, takerPosition)));
    }
  }

  for (const maker of params.makers) {
    const [makerNonce] = getUserNoncePda(maker.order.maker, programId);
    const [makerPosition] = getPositionPda(maker.order.maker, params.market, programId);

    if (!maker.isFullFill) {
      const makerOrderHash = hashOrder(maker.order);
      const [makerOrderStatus] = getOrderStatusPda(makerOrderHash, programId);
      keys.push(writable(makerOrderStatus));
    }

    keys.push(readonly(makerNonce));
    keys.push(writable(makerPosition));

    if (maker.isDeposit) {
      const [vault] = getVaultPda(maker.depositMint, params.market, programId);
      const [globalDepositToken] = getGlobalDepositTokenPda(
        maker.depositMint,
        programId
      );
      const [makerGlobalDeposit] = getUserGlobalDepositPda(
        maker.order.maker,
        maker.depositMint,
        programId
      );

      keys.push(readonly(maker.depositMint));
      keys.push(writable(vault));
      keys.push(readonly(globalDepositToken));
      keys.push(writable(makerGlobalDeposit));

      for (const [conditionalMint] of getAllConditionalMintPdas(
        params.market,
        maker.depositMint,
        params.numOutcomes,
        programId
      )) {
        keys.push(writable(conditionalMint));
        keys.push(writable(getConditionalTokenAta(conditionalMint, makerPosition)));
      }
    }

    keys.push(writable(getConditionalTokenAta(receiveMint, makerPosition)));
    keys.push(writable(getConditionalTokenAta(giveMint, makerPosition)));
  }

  const buffers: Buffer[] = [
    Buffer.from([INSTRUCTION.DEPOSIT_AND_SWAP]),
    serializeOrder(signedOrderToOrder(params.takerOrder)),
    params.takerOrder.signature,
    toU8(params.makers.length),
    toU8(fullFillBitmask),
    toU8(depositBitmask),
  ];

  for (const maker of params.makers) {
    buffers.push(serializeOrder(signedOrderToOrder(maker.order)));
    buffers.push(maker.order.signature);
    buffers.push(toU64Le(maker.makerFillAmount));
    buffers.push(toU64Le(maker.takerFillAmount));
  }

  return new TransactionInstruction({
    keys,
    programId,
    data: Buffer.concat(buffers),
  });
}

/**
 * Build WithdrawFromGlobal instruction
 *
 * Accounts:
 * 0. user (signer, mut)
 * 1. global_deposit_token (readonly) - PDA ["global_deposit", mint]
 * 2. mint (readonly)
 * 3. user_global_deposit (mut) - PDA ["global_deposit", user, mint]
 * 4. user_token_account (mut) - user's ATA for mint
 * 5. token_program (readonly)
 * 6. exchange (readonly)
 *
 * Data: [discriminator, amount (u64)]
 */
export function buildWithdrawFromGlobalIx(
  params: WithdrawFromGlobalParams,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  const [globalDepositToken] = getGlobalDepositTokenPda(params.mint, programId);
  const [userGlobalDeposit] = getUserGlobalDepositPda(params.user, params.mint, programId);
  const [exchange] = getExchangePda(programId);
  const userTokenAccount = getDepositTokenAta(params.mint, params.user);

  const keys: AccountMeta[] = [
    signerMut(params.user),
    readonly(globalDepositToken),
    readonly(params.mint),
    writable(userGlobalDeposit),
    writable(userTokenAccount),
    readonly(TOKEN_PROGRAM_ID),
    readonly(exchange),
  ];

  const data = Buffer.concat([
    Buffer.from([INSTRUCTION.WITHDRAW_FROM_GLOBAL]),
    toU64Le(params.amount),
  ]);

  return new TransactionInstruction({
    keys,
    programId,
    data,
  });
}

/**
 * Build ClosePositionAlt instruction
 *
 * Accounts:
 * 0. operator (signer, mut)
 * 1. exchange (readonly)
 * 2. position (readonly)
 * 3. market (readonly)
 * 4. lookup_table (mut)
 * 5. alt_program (readonly)
 *
 * Data: [discriminator]
 */
export function buildClosePositionAltIx(
  params: ClosePositionAltParams,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  const [exchange] = getExchangePda(programId);

  const keys: AccountMeta[] = [
    signerMut(params.operator),
    readonly(exchange),
    readonly(params.position),
    readonly(params.market),
    writable(params.lookupTable),
    readonly(ALT_PROGRAM_ID),
  ];

  return new TransactionInstruction({
    keys,
    programId,
    data: Buffer.from([INSTRUCTION.CLOSE_POSITION_ALT]),
  });
}

/**
 * Build CloseOrderStatus instruction
 *
 * Accounts:
 * 0. operator (signer, mut)
 * 1. exchange (readonly)
 * 2. order_status (mut)
 *
 * Data: [discriminator, order_hash (32)]
 */
export function buildCloseOrderStatusIx(
  params: CloseOrderStatusParams,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  const [exchange] = getExchangePda(programId);
  const [orderStatus] = getOrderStatusPda(params.orderHash, programId);

  const keys: AccountMeta[] = [
    signerMut(params.operator),
    readonly(exchange),
    writable(orderStatus),
  ];

  const data = Buffer.concat([
    Buffer.from([INSTRUCTION.CLOSE_ORDER_STATUS]),
    params.orderHash,
  ]);

  return new TransactionInstruction({
    keys,
    programId,
    data,
  });
}

/**
 * Build ClosePositionTokenAccounts instruction
 *
 * Accounts:
 * 0. operator (signer, mut)
 * 1. exchange (readonly)
 * 2. market (readonly)
 * 3. position (readonly)
 * 4. token_2022_program (readonly)
 * + per deposit mint: deposit_mint, conditional_mint/position_ata pairs
 *
 * Data: [discriminator]
 */
export function buildClosePositionTokenAccountsIx(
  params: ClosePositionTokenAccountsParams,
  numOutcomes: number,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  validateOutcomes(numOutcomes);
  if (params.depositMints.length === 0) {
    throw ProgramSdkError.missingField("deposit_mints");
  }

  const [exchange] = getExchangePda(programId);

  const keys: AccountMeta[] = [
    signerMut(params.operator),
    readonly(exchange),
    readonly(params.market),
    readonly(params.position),
    readonly(TOKEN_2022_PROGRAM_ID),
  ];

  for (const depositMint of params.depositMints) {
    keys.push(readonly(depositMint));

    for (let i = 0; i < numOutcomes; i += 1) {
      const [conditionalMint] = getConditionalMintPda(
        params.market,
        depositMint,
        i,
        programId
      );
      keys.push(readonly(conditionalMint));
      keys.push(writable(getConditionalTokenAta(conditionalMint, params.position)));
    }
  }

  return new TransactionInstruction({
    keys,
    programId,
    data: Buffer.from([INSTRUCTION.CLOSE_POSITION_TOKEN_ACCOUNTS]),
  });
}

/**
 * Build CloseOrderbookAlt instruction
 *
 * Accounts:
 * 0. operator (signer, mut)
 * 1. exchange (readonly)
 * 2. orderbook (readonly)
 * 3. market (readonly)
 * 4. lookup_table (mut)
 * 5. alt_program (readonly)
 *
 * Data: [discriminator]
 */
export function buildCloseOrderbookAltIx(
  params: CloseOrderbookAltParams,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  const [exchange] = getExchangePda(programId);

  const keys: AccountMeta[] = [
    signerMut(params.operator),
    readonly(exchange),
    readonly(params.orderbook),
    readonly(params.market),
    writable(params.lookupTable),
    readonly(ALT_PROGRAM_ID),
  ];

  return new TransactionInstruction({
    keys,
    programId,
    data: Buffer.from([INSTRUCTION.CLOSE_ORDERBOOK_ALT]),
  });
}

/**
 * Build CloseOrderbook instruction
 *
 * Accounts:
 * 0. operator (signer, mut)
 * 1. exchange (readonly)
 * 2. orderbook (mut)
 * 3. market (readonly)
 * 4. lookup_table (readonly)
 *
 * Data: [discriminator]
 */
export function buildCloseOrderbookIx(
  params: CloseOrderbookParams,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  const [exchange] = getExchangePda(programId);

  const keys: AccountMeta[] = [
    signerMut(params.operator),
    readonly(exchange),
    writable(params.orderbook),
    readonly(params.market),
    readonly(params.lookupTable),
  ];

  return new TransactionInstruction({
    keys,
    programId,
    data: Buffer.from([INSTRUCTION.CLOSE_ORDERBOOK]),
  });
}

// ============================================================================
// TRANSACTION BUILDERS (_tx convenience wrappers)
// Each wraps the corresponding _ix builder into a Transaction with feePayer set.
// ============================================================================

export function buildInitializeTx(
  params: InitializeParams,
  programId: PublicKey = PROGRAM_ID
): Transaction {
  const ix = buildInitializeIx(params, programId);
  return new Transaction({ feePayer: params.authority }).add(ix);
}

export function buildCreateMarketTx(
  params: CreateMarketParams,
  marketId: bigint,
  programId: PublicKey = PROGRAM_ID
): Transaction {
  const ix = buildCreateMarketIx(params, marketId, programId);
  return new Transaction({ feePayer: params.manager }).add(ix);
}

export function buildAddDepositMintTx(
  params: AddDepositMintParams,
  market: PublicKey,
  numOutcomes: number,
  programId: PublicKey = PROGRAM_ID
): Transaction {
  const ix = buildAddDepositMintIx(params, market, numOutcomes, programId);
  return new Transaction({ feePayer: params.manager }).add(ix);
}

export function buildDepositTx(
  params: BuildDepositParams,
  numOutcomes: number,
  programId: PublicKey = PROGRAM_ID
): Transaction {
  const ix = buildDepositIx(params, numOutcomes, programId);
  return new Transaction({ feePayer: params.user }).add(ix);
}

export function buildMergeTx(
  params: BuildMergeParams,
  numOutcomes: number,
  programId: PublicKey = PROGRAM_ID
): Transaction {
  const ix = buildMergeIx(params, numOutcomes, programId);
  return new Transaction({ feePayer: params.user }).add(ix);
}

export function buildCancelOrderTx(
  operator: PublicKey,
  market: PublicKey,
  order: SignedOrder,
  programId: PublicKey = PROGRAM_ID
): Transaction {
  const ix = buildCancelOrderIx(operator, market, order, programId);
  return new Transaction({ feePayer: operator }).add(ix);
}

export function buildIncrementNonceTx(
  user: PublicKey,
  programId: PublicKey = PROGRAM_ID
): Transaction {
  const ix = buildIncrementNonceIx(user, programId);
  return new Transaction({ feePayer: user }).add(ix);
}

export function buildSettleMarketTx(
  params: SettleMarketParams,
  programId: PublicKey = PROGRAM_ID
): Transaction {
  const ix = buildSettleMarketIx(params, programId);
  return new Transaction({ feePayer: params.oracle }).add(ix);
}

export function buildRedeemWinningsTx(
  params: RedeemWinningsParams,
  outcomeIndex: number,
  programId: PublicKey = PROGRAM_ID
): Transaction {
  const ix = buildRedeemWinningsIx(params, outcomeIndex, programId);
  return new Transaction({ feePayer: params.user }).add(ix);
}

export function buildSetPausedTx(
  authority: PublicKey,
  paused: boolean,
  programId: PublicKey = PROGRAM_ID
): Transaction {
  const ix = buildSetPausedIx(authority, paused, programId);
  return new Transaction({ feePayer: authority }).add(ix);
}

export function buildSetOperatorTx(
  authority: PublicKey,
  newOperator: PublicKey,
  programId: PublicKey = PROGRAM_ID
): Transaction {
  const ix = buildSetOperatorIx(authority, newOperator, programId);
  return new Transaction({ feePayer: authority }).add(ix);
}

export function buildWithdrawFromPositionTx(
  params: WithdrawFromPositionParams,
  isToken2022: boolean,
  programId: PublicKey = PROGRAM_ID
): Transaction {
  const ix = buildWithdrawFromPositionIx(params, isToken2022, programId);
  return new Transaction({ feePayer: params.user }).add(ix);
}

export function buildActivateMarketTx(
  params: ActivateMarketParams,
  programId: PublicKey = PROGRAM_ID
): Transaction {
  const ix = buildActivateMarketIx(params, programId);
  return new Transaction({ feePayer: params.manager }).add(ix);
}

export function buildMatchOrdersMultiTx(
  params: MatchOrdersMultiParams,
  programId: PublicKey = PROGRAM_ID
): Transaction {
  const ix = buildMatchOrdersMultiIx(params, programId);
  return new Transaction({ feePayer: params.operator }).add(ix);
}

export function buildSetAuthorityTx(
  params: SetAuthorityParams,
  programId: PublicKey = PROGRAM_ID
): Transaction {
  const ix = buildSetAuthorityIx(params, programId);
  return new Transaction({ feePayer: params.currentAuthority }).add(ix);
}

export function buildSetManagerTx(
  params: SetManagerParams,
  programId: PublicKey = PROGRAM_ID
): Transaction {
  const ix = buildSetManagerIx(params, programId);
  return new Transaction({ feePayer: params.authority }).add(ix);
}

export function buildCreateOrderbookTx(
  params: CreateOrderbookParams,
  programId: PublicKey = PROGRAM_ID
): Transaction {
  const ix = buildCreateOrderbookIx(params, programId);
  return new Transaction({ feePayer: params.manager }).add(ix);
}

export function buildWhitelistDepositTokenTx(
  params: WhitelistDepositTokenParams,
  programId: PublicKey = PROGRAM_ID
): Transaction {
  const ix = buildWhitelistDepositTokenIx(params, programId);
  return new Transaction({ feePayer: params.authority }).add(ix);
}

export function buildDepositToGlobalTx(
  params: DepositToGlobalParams,
  programId: PublicKey = PROGRAM_ID
): Transaction {
  const ix = buildDepositToGlobalIx(params, programId);
  return new Transaction({ feePayer: params.user }).add(ix);
}

export function buildDepositToGlobalTxWithAlt(
  params: DepositToGlobalParams,
  altContext: DepositToGlobalAltContext,
  programId: PublicKey = PROGRAM_ID
): Transaction {
  const ix = buildDepositToGlobalIxWithAlt(params, altContext, programId);
  return new Transaction({ feePayer: params.user }).add(ix);
}

export function buildGlobalToMarketDepositTx(
  params: GlobalToMarketDepositParams,
  numOutcomes: number,
  programId: PublicKey = PROGRAM_ID
): Transaction {
  const ix = buildGlobalToMarketDepositIx(params, numOutcomes, programId);
  return new Transaction({ feePayer: params.user }).add(ix);
}

export function buildInitPositionTokensTx(
  params: InitPositionTokensParams,
  numOutcomes: number,
  programId: PublicKey = PROGRAM_ID
): Transaction {
  const ix = buildInitPositionTokensIx(params, numOutcomes, programId);
  return new Transaction({ feePayer: params.payer }).add(ix);
}

export function buildExtendPositionTokensTx(
  params: ExtendPositionTokensParams,
  numOutcomes: number,
  programId: PublicKey = PROGRAM_ID
): Transaction {
  const ix = buildExtendPositionTokensIx(params, numOutcomes, programId);
  return new Transaction({ feePayer: params.operator }).add(ix);
}

export function buildDepositAndSwapTx(
  params: DepositAndSwapParams,
  programId: PublicKey = PROGRAM_ID
): Transaction {
  const ix = buildDepositAndSwapIx(params, programId);
  return new Transaction({ feePayer: params.operator }).add(ix);
}

export function buildWithdrawFromGlobalTx(
  params: WithdrawFromGlobalParams,
  programId: PublicKey = PROGRAM_ID
): Transaction {
  const ix = buildWithdrawFromGlobalIx(params, programId);
  return new Transaction({ feePayer: params.user }).add(ix);
}

export function buildClosePositionAltTx(
  params: ClosePositionAltParams,
  programId: PublicKey = PROGRAM_ID
): Transaction {
  const ix = buildClosePositionAltIx(params, programId);
  return new Transaction({ feePayer: params.operator }).add(ix);
}

export function buildCloseOrderStatusTx(
  params: CloseOrderStatusParams,
  programId: PublicKey = PROGRAM_ID
): Transaction {
  const ix = buildCloseOrderStatusIx(params, programId);
  return new Transaction({ feePayer: params.operator }).add(ix);
}

export function buildClosePositionTokenAccountsTx(
  params: ClosePositionTokenAccountsParams,
  numOutcomes: number,
  programId: PublicKey = PROGRAM_ID
): Transaction {
  const ix = buildClosePositionTokenAccountsIx(params, numOutcomes, programId);
  return new Transaction({ feePayer: params.operator }).add(ix);
}

export function buildCloseOrderbookAltTx(
  params: CloseOrderbookAltParams,
  programId: PublicKey = PROGRAM_ID
): Transaction {
  const ix = buildCloseOrderbookAltIx(params, programId);
  return new Transaction({ feePayer: params.operator }).add(ix);
}

export function buildCloseOrderbookTx(
  params: CloseOrderbookParams,
  programId: PublicKey = PROGRAM_ID
): Transaction {
  const ix = buildCloseOrderbookIx(params, programId);
  return new Transaction({ feePayer: params.operator }).add(ix);
}
