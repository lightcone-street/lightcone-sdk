import { V1Transaction, type V1TransactionContext } from "./transaction";
import {
  PublicKey,
  TransactionInstruction,
  AccountMeta,
} from "@solana/web3.js";
import {
  INSTRUCTION,
  SYSTEM_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  MPL_TOKEN_METADATA_PROGRAM_ID,
  RENT_SYSVAR_ID,
  MAX_DEPOSIT_MINTS_PER_IX,
  MAX_MAKERS,
  TAKER_MASK,
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
  WithdrawConditionalFromPositionParams,
  WithdrawFromPositionParams,
  ActivateMarketParams,
  MatchOrdersMultiParams,
  SetAuthorityParams,
  SetManagerParams,
  AcceptRoleParams,
  SetOracleParams,
  CreateOrderbookParams,
  WhitelistDepositTokenParams,
  SetDepositTokenStatusParams,
  DepositToGlobalParams,
  GlobalToMarketDepositParams,
  InitPositionTokensParams,
  DepositAndSwapParams,
  WithdrawFromGlobalParams,
  CloseOrderStatusParams,
  ClosePositionTokenAccountsParams,
  CloseOrderbookParams,
  SignedOrder,
  ConditionalMetadataParams,
  SetFeeReceiverParams,
  SetFeeReceiverWithAtasParams,
  SetMarketFeesParams,
  OrderSide,
} from "./types";
import {
  getEventAuthorityPda,
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
  getGlobalDepositTokenPda,
  getUserGlobalDepositPda,
  getMplMetadataPda,
} from "./pda";
import {
  toU8,
  toU16Le,
  toI16Le,
  toU32Le,
  toU64Le,
  serializeConditionalMetadata,
  getConditionalTokenAta,
  getDepositTokenAta,
  validateOutcomes,
  validateFeePair,
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

/**
 * Build a public Lightcone instruction, appending the event transport trailer.
 *
 * The program pops the last two accounts of every public instruction before
 * dispatch: the event-authority PDA (seed "__event_authority", readonly, never
 * a signer) and the executable program account (readonly). It signs its final
 * event-batch self-CPI with that PDA, so an instruction without the trailer
 * fails closed before any state change (on-chain errors 46 and 68). Public
 * instructions require transaction-level invocation except for the governance
 * allowlist documented in this module's README. Unsupported CPI fails with error 73.
 * Routing every builder through this constructor keeps the invariant in one place.
 */
function publicInstruction(
  programId: PublicKey,
  keys: AccountMeta[],
  data: Buffer
): TransactionInstruction {
  const [eventAuthority] = getEventAuthorityPda(programId);
  return new TransactionInstruction({
    keys: [...keys, readonly(eventAuthority), readonly(programId)],
    programId,
    data,
  });
}

/** Reject oracle keys that cannot sign top-level settlement instructions. */
function validateOracle(oracle: PublicKey): void {
  if (oracle.equals(zeroPubkey()) || !PublicKey.isOnCurve(oracle.toBytes())) {
    throw ProgramSdkError.invalidOracle();
  }
}

/** Reject zero and off-curve beneficiaries that cannot sign user exits. */
function validateUser(user: PublicKey): void {
  if (user.equals(PublicKey.default) || !PublicKey.isOnCurve(user.toBytes())) {
    throw ProgramSdkError.invalidPubkey(user.toBase58());
  }
}

function zeroPubkey(): PublicKey {
  return new PublicKey(Buffer.alloc(32));
}

interface OrderbookMintInput {
  mint: PublicKey;
  depositMint: PublicKey;
  isBase: boolean;
}

interface CanonicalOrderbookMints {
  mintA: OrderbookMintInput;
  mintB: OrderbookMintInput;
  baseIndex: number;
}

function canonicalOrderbookMints(params: CreateOrderbookParams): CanonicalOrderbookMints {
  if (!Number.isInteger(params.baseIndex) || params.baseIndex < 0 || params.baseIndex > 1) {
    throw ProgramSdkError.invalidOutcomeIndex(params.baseIndex, 1);
  }
  if (params.mintA.equals(params.mintB)) {
    throw ProgramSdkError.invalidMintOrder();
  }

  const left: OrderbookMintInput = {
    mint: params.mintA,
    depositMint: params.mintADepositMint,
    isBase: params.baseIndex === 0,
  };
  const right: OrderbookMintInput = {
    mint: params.mintB,
    depositMint: params.mintBDepositMint,
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

/** Preserve each collateral's binding when sorting the conditional mint pair. */
function tradingDepositTokens(
  params: Pick<MatchOrdersMultiParams, "baseMint" | "quoteMint" | "baseDepositMint" | "quoteDepositMint">,
  programId: PublicKey
): [PublicKey, PublicKey] {
  if (params.baseMint.equals(params.quoteMint)) {
    throw ProgramSdkError.invalidOrderbook();
  }
  if (params.baseDepositMint.equals(params.quoteDepositMint)) {
    throw ProgramSdkError.depositMintMismatch();
  }
  const [depositMintA, depositMintB] =
    Buffer.compare(params.baseMint.toBuffer(), params.quoteMint.toBuffer()) < 0
      ? [params.baseDepositMint, params.quoteDepositMint]
      : [params.quoteDepositMint, params.baseDepositMint];
  return [
    getGlobalDepositTokenPda(depositMintA, programId)[0],
    getGlobalDepositTokenPda(depositMintB, programId)[0],
  ];
}

/** Validate before JavaScript bitwise operators can coerce or truncate the input. */
function participantMaskBytes(mask: number, makerCount: number): Buffer {
  const bytes = toU16Le(mask);
  const allowed = ((1 << makerCount) - 1) | TAKER_MASK;
  if ((mask & ~allowed) !== 0) {
    throw ProgramSdkError.serialization(`Invalid participant mask ${mask} for ${makerCount} makers`);
  }
  return bytes;
}

function validateTradingSignature(order: SignedOrder): void {
  if (order.signature.length !== 64) {
    throw ProgramSdkError.invalidDataLength("signature", 64, order.signature.length);
  }
}

/** Compact orders obtain their market and mint identities from these accounts. */
function validateTradingOrders(
  params: Pick<MatchOrdersMultiParams, "market" | "baseMint" | "quoteMint" | "takerOrder">,
  makers: SignedOrder[]
): void {
  for (const order of [params.takerOrder, ...makers]) {
    validateTradingSignature(order);
    if (order.side !== OrderSide.BID && order.side !== OrderSide.ASK) {
      throw ProgramSdkError.invalidSide(order.side);
    }
    if (!order.market.equals(params.market) || !order.baseMint.equals(params.baseMint) || !order.quoteMint.equals(params.quoteMint)) {
      throw ProgramSdkError.invalidOrderbook();
    }
  }
  if (makers.some((order) => order.side === params.takerOrder.side)) {
    throw ProgramSdkError.serialization("Maker orders must trade the opposite side of the taker");
  }
}

/** Global funding backs the asset offered by the participant's signed side. */
function validateFundingMint(
  order: SignedOrder,
  depositMint: PublicKey,
  params: DepositAndSwapParams
): void {
  if (order.side !== OrderSide.BID && order.side !== OrderSide.ASK) {
    throw ProgramSdkError.invalidSide(order.side);
  }
  const expected = order.side === OrderSide.BID ? params.quoteDepositMint : params.baseDepositMint;
  if (!depositMint.equals(expected)) {
    throw ProgramSdkError.depositMintMismatch();
  }
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

  return publicInstruction(programId, keys, data);
}

/**
 * Build CreateMarket instruction
 * Creates a new market in Pending status. Rejects zero or off-curve oracle keys.
 *
 * Accounts:
 * 0. manager (signer, mut) - Must be exchange manager
 * 1. exchange (mut) - Exchange PDA
 * 2. market (mut) - Market PDA
 * 3. system_program (readonly)
 * 4. condition_tombstone (mut) - Condition uniqueness PDA
 *
 * Data: [discriminator, num_outcomes (u8), oracle (32), question_id (32), maker_fee_bps (i16), taker_fee_bps (i16)]
 */
export function buildCreateMarketIx(
  params: CreateMarketParams,
  marketId: bigint,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  validateOutcomes(params.numOutcomes);
  validateOracle(params.oracle);
  validateFeePair(params.makerFeeBps, params.takerFeeBps);

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
    toI16Le(params.makerFeeBps),
    toI16Le(params.takerFeeBps),
  ]);

  return publicInstruction(programId, keys, data);
}

/**
 * Build AddDepositMint instruction
 *
 * The program rejects the instruction with error 75 (TooManyDepositMints) once
 * the market already holds MAX_DEPOSIT_MINTS_PER_MARKET deposit mints.
 *
 * Accounts (9 + num_outcomes, + 2 trailer):
 * 0. manager (signer)
 * 1. exchange
 * 2. market (mut) - deposit_mint_count is incremented
 * 3. deposit_mint
 * 4. vault
 * 5. mint_authority
 * 6. token_program (SPL Token)
 * 7. system_program
 * 8. global_deposit_token
 * 9+ conditional_mints[0..num_outcomes]
 * + event_authority, program (readonly trailer)
 *
 * Data: [discriminator]
 */
export function buildAddDepositMintIx(
  params: AddDepositMintParams,
  market: PublicKey,
  numOutcomes: number,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  validateOutcomes(numOutcomes);

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
    writable(market),
    readonly(params.depositMint),
    writable(vault),
    readonly(mintAuthority),
    readonly(TOKEN_PROGRAM_ID),
    readonly(SYSTEM_PROGRAM_ID),
    readonly(globalDepositToken),
  ];

  for (const [mint] of conditionalMints) {
    keys.push(writable(mint));
  }

  const data = Buffer.from([INSTRUCTION.ADD_DEPOSIT_MINT]);

  return publicInstruction(programId, keys, data);
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
 * 9. associated_token_program
 * 10. system_program
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

  return publicInstruction(programId, keys, data);
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

  return publicInstruction(programId, keys, data);
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

  return publicInstruction(programId, keys, data);
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

  return publicInstruction(programId, keys, data);
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

  return publicInstruction(programId, keys, data);
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
 * 10. exchange
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
    readonly(exchange),
  ];

  const data = Buffer.concat([
    Buffer.from([INSTRUCTION.REDEEM_WINNINGS]),
    toU64Le(params.amount),
    toU8(outcomeIndex),
  ]);

  return publicInstruction(programId, keys, data);
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

  return publicInstruction(programId, keys, data);
}

/**
 * Build SetOperator instruction
 *
 * Proposes a new exchange operator. The active operator changes only after the
 * proposed operator signs AcceptOperator.
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

  return publicInstruction(programId, keys, data);
}

/**
 * Build WithdrawConditionalFromPosition instruction
 *
 * The conditional mint is derived from `(market, depositMint, outcomeIndex)`.
 *
 * Accounts:
 * 0. user (signer, mut)
 * 1. exchange (readonly)
 * 2. market (readonly)
 * 3. position (readonly)
 * 4. deposit_mint (readonly)
 * 5. conditional_mint (readonly)
 * 6. position_conditional_ata (mut)
 * 7. user_conditional_ata (mut)
 * 8. token_program (readonly)
 *
 * Data: [discriminator, amount (u64), outcome_index (u8)]
 */
export function buildWithdrawConditionalFromPositionIx(
  params: WithdrawConditionalFromPositionParams,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  if (
    !Number.isInteger(params.outcomeIndex) ||
    params.outcomeIndex < 0 ||
    params.outcomeIndex > 0xff
  ) {
    throw ProgramSdkError.invalidOutcomeIndex(params.outcomeIndex, 0xff);
  }

  const [exchange] = getExchangePda(programId);
  const [position] = getPositionPda(params.user, params.market, programId);
  const [conditionalMint] = getConditionalMintPda(
    params.market,
    params.depositMint,
    params.outcomeIndex,
    programId
  );
  const positionConditionalAta = getConditionalTokenAta(conditionalMint, position);
  const userConditionalAta = getConditionalTokenAta(conditionalMint, params.user);

  const keys: AccountMeta[] = [
    signerMut(params.user),
    readonly(exchange),
    readonly(params.market),
    readonly(position),
    readonly(params.depositMint),
    readonly(conditionalMint),
    writable(positionConditionalAta),
    writable(userConditionalAta),
    readonly(TOKEN_PROGRAM_ID),
  ];

  const data = Buffer.concat([
    Buffer.from([INSTRUCTION.WITHDRAW_CONDITIONAL_FROM_POSITION]),
    toU64Le(params.amount),
    toU8(params.outcomeIndex),
  ]);

  return publicInstruction(programId, keys, data);
}

/**
 * Compatibility wrapper for conditional-token position withdrawal.
 */
export function buildWithdrawFromPositionIx(
  params: WithdrawFromPositionParams,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  return buildWithdrawConditionalFromPositionIx(params, programId);
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

  return publicInstruction(programId, keys, data);
}

/**
 * Build a match against 1..11 maker orders using existing conditional balances.
 *
 * The fixed prefix starts with operator, exchange, market, orderbook, GDT A, GDT B.
 * GDTs follow canonical conditional-mint ordering regardless of trade direction.
 * Each clear mask bit includes a writable OrderStatus account. Bit 15 selects the taker.
 * Positions and nonces are read-only. Settlement ATAs are writable.
 *
 * Full instruction data: discriminator, 37-byte taker order, 64-byte signature,
 * maker count:u8, full-fill mask:u16 LE, then 117 bytes per maker.
 * Each maker record holds its compact order, signature, and two u64 fill amounts.
 * Fill amounts use integer units of the asset given by that participant.
 * Business accounts total 18 + 5*M - popcount(fullFillBitmask), followed by two trailers.
 * Eleven makers is a parser ceiling. The outer transaction must fit execution limits.
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

  const maskBytes = participantMaskBytes(params.fullFillBitmask, params.makerOrders.length);
  validateTradingOrders(params, params.makerOrders);
  const [gdtA, gdtB] = tradingDepositTokens(params, programId);

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
  const feeReceiverQuoteAta = getConditionalTokenAta(
    params.quoteMint,
    params.feeReceiver
  );

  const keys: AccountMeta[] = [
    signerMut(params.operator),
    readonly(exchange),
    readonly(params.market),
    readonly(orderbook),
    readonly(gdtA),
    readonly(gdtB),
  ];

  // Taker order status if not fully filled (bit 15 = 0)
  const takerFullFill = (params.fullFillBitmask & TAKER_MASK) !== 0;
  if (!takerFullFill) {
    const [takerOrderStatus] = getOrderStatusPda(takerOrderHash, programId);
    keys.push(writable(takerOrderStatus));
  }

  keys.push(readonly(takerNonce));
  keys.push(readonly(takerPosition));
  keys.push(readonly(params.baseMint));
  keys.push(readonly(params.quoteMint));
  keys.push(writable(takerBaseAta));
  keys.push(writable(takerQuoteAta));
  keys.push(readonly(TOKEN_PROGRAM_ID));
  keys.push(readonly(SYSTEM_PROGRAM_ID));
  keys.push(writable(feeReceiverQuoteAta));
  keys.push(readonly(params.feeReceiver));
  keys.push(readonly(ASSOCIATED_TOKEN_PROGRAM_ID));

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
    keys.push(readonly(makerPosition));
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
    maskBytes,
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

  return publicInstruction(programId, keys, data);
}

/**
 * Build SetAuthority instruction
 *
 * Proposes a new exchange authority. The active authority changes only after
 * the proposed authority signs AcceptAuthority.
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

  return publicInstruction(programId, keys, data);
}

/**
 * Create a book for one market outcome backed by two distinct collateral mints.
 *
 * The builder sorts the supplied conditional mints with their collateral identities.
 * It preserves the requested base orientation and validates conditional PDA derivations.
 * The program validates live registration, mint properties, market state, and outcome bounds.
 *
 * Business accounts: manager, market, mint A, mint B, orderbook, GDT A, GDT B,
 * exchange, System program, collateral A, collateral B, Token program, ATA program,
 * fee receiver, and the fee receiver's quote ATA. Two event trailers follow.
 * Data is exactly [15, canonical baseIndex, outcomeIndex].
 */
export function buildCreateOrderbookIx(
  params: CreateOrderbookParams,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  const canonical = canonicalOrderbookMints(params);
  if (!Number.isInteger(params.outcomeIndex) || params.outcomeIndex < 0 || params.outcomeIndex >= MAX_OUTCOMES) {
    throw ProgramSdkError.invalidOutcomeIndex(params.outcomeIndex, MAX_OUTCOMES - 1);
  }
  if (canonical.mintA.depositMint.equals(canonical.mintB.depositMint)) {
    throw ProgramSdkError.depositMintMismatch();
  }
  for (const { mint, depositMint } of [canonical.mintA, canonical.mintB]) {
    const [expected] = getConditionalMintPda(params.market, depositMint, params.outcomeIndex, programId);
    if (!mint.equals(expected)) {
      throw ProgramSdkError.invalidConditionalMint();
    }
  }
  const [exchange] = getExchangePda(programId);
  const [orderbook] = getOrderbookPda(canonical.mintA.mint, canonical.mintB.mint, programId);
  const quoteMint = canonical.baseIndex === 0 ? canonical.mintB.mint : canonical.mintA.mint;
  const feeReceiverQuoteAta = getConditionalTokenAta(quoteMint, params.feeReceiver);

  const keys: AccountMeta[] = [
    signerMut(params.manager),
    readonly(params.market),
    readonly(canonical.mintA.mint),
    readonly(canonical.mintB.mint),
    writable(orderbook),
    readonly(getGlobalDepositTokenPda(canonical.mintA.depositMint, programId)[0]),
    readonly(getGlobalDepositTokenPda(canonical.mintB.depositMint, programId)[0]),
    readonly(exchange),
    readonly(SYSTEM_PROGRAM_ID),
    readonly(canonical.mintA.depositMint),
    readonly(canonical.mintB.depositMint),
    readonly(TOKEN_PROGRAM_ID),
    readonly(ASSOCIATED_TOKEN_PROGRAM_ID),
    readonly(params.feeReceiver),
    writable(feeReceiverQuoteAta),
  ];

  const data = Buffer.concat([
    Buffer.from([INSTRUCTION.CREATE_ORDERBOOK]),
    toU8(canonical.baseIndex),
    toU8(params.outcomeIndex),
  ]);

  return publicInstruction(programId, keys, data);
}

/**
 * Build SetManager instruction
 *
 * Proposes a new exchange manager. The active manager changes only after the
 * proposed manager signs AcceptManager.
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

  return publicInstruction(programId, keys, data);
}

function buildAcceptRoleIx(
  params: AcceptRoleParams,
  discriminator: number,
  programId: PublicKey
): TransactionInstruction {
  const [exchange] = getExchangePda(programId);
  const keys: AccountMeta[] = [
    signer(params.incomingRole),
    writable(exchange),
  ];

  return publicInstruction(programId, keys, Buffer.from([discriminator]));
}

/**
 * Build AcceptAuthority instruction.
 */
export function buildAcceptAuthorityIx(
  params: AcceptRoleParams,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  return buildAcceptRoleIx(params, INSTRUCTION.ACCEPT_AUTHORITY, programId);
}

/**
 * Build AcceptManager instruction.
 */
export function buildAcceptManagerIx(
  params: AcceptRoleParams,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  return buildAcceptRoleIx(params, INSTRUCTION.ACCEPT_MANAGER, programId);
}

/**
 * Build AcceptOperator instruction.
 */
export function buildAcceptOperatorIx(
  params: AcceptRoleParams,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  return buildAcceptRoleIx(params, INSTRUCTION.ACCEPT_OPERATOR, programId);
}

/**
 * Build SetOracle instruction. Rejects zero or off-curve oracle keys.
 */
export function buildSetOracleIx(
  params: SetOracleParams,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  validateOracle(params.newOracle);

  const [exchange] = getExchangePda(programId);
  const keys: AccountMeta[] = [
    signer(params.authority),
    readonly(exchange),
    writable(params.market),
  ];

  return publicInstruction(
    programId,
    keys,
    Buffer.concat([
      Buffer.from([INSTRUCTION.SET_ORACLE]),
      params.newOracle.toBuffer(),
    ])
  );
}

/**
 * Build SetMarketFees instruction.
 */
export function buildSetMarketFeesIx(
  params: SetMarketFeesParams,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  if (params.updates.length === 0) {
    throw ProgramSdkError.missingField("updates");
  }

  const [exchange] = getExchangePda(programId);
  const keys: AccountMeta[] = [
    signerMut(params.manager),
    readonly(exchange),
  ];
  const buffers: Buffer[] = [Buffer.from([INSTRUCTION.SET_MARKET_FEES])];

  for (const update of params.updates) {
    validateFeePair(update.makerFeeBps, update.takerFeeBps);
    keys.push(writable(update.market));
    buffers.push(toI16Le(update.makerFeeBps));
    buffers.push(toI16Le(update.takerFeeBps));
  }

  return publicInstruction(programId, keys, Buffer.concat(buffers));
}

/**
 * Build SetFeeReceiver instruction.
 */
export function buildSetFeeReceiverIx(
  params: SetFeeReceiverParams,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  if (params.newFeeReceiver.equals(zeroPubkey())) {
    throw ProgramSdkError.invalidFeeReceiver();
  }

  const [exchange] = getExchangePda(programId);
  const keys: AccountMeta[] = [
    signerMut(params.authority),
    writable(exchange),
  ];

  return publicInstruction(
    programId,
    keys,
    Buffer.concat([
      Buffer.from([INSTRUCTION.SET_FEE_RECEIVER]),
      params.newFeeReceiver.toBuffer(),
    ])
  );
}

/**
 * Build SetFeeReceiver instruction with optional ATA creation accounts.
 *
 * Uses the same discriminator and data as buildSetFeeReceiverIx, while
 * appending the optional account block used by the on-chain program to create
 * receiver quote ATAs.
 */
export function buildSetFeeReceiverWithAtasIx(
  params: SetFeeReceiverWithAtasParams,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  if (params.newFeeReceiver.equals(zeroPubkey())) {
    throw ProgramSdkError.invalidFeeReceiver();
  }
  if (params.quoteMints.length === 0) {
    throw ProgramSdkError.missingField("quote_mints");
  }

  const [exchange] = getExchangePda(programId);
  const keys: AccountMeta[] = [
    signerMut(params.authority),
    writable(exchange),
    readonly(params.newFeeReceiver),
    readonly(TOKEN_PROGRAM_ID),
    readonly(ASSOCIATED_TOKEN_PROGRAM_ID),
    readonly(SYSTEM_PROGRAM_ID),
  ];

  for (const quoteMint of params.quoteMints) {
    keys.push(readonly(quoteMint));
    keys.push(writable(getConditionalTokenAta(quoteMint, params.newFeeReceiver)));
  }

  return publicInstruction(
    programId,
    keys,
    Buffer.concat([
      Buffer.from([INSTRUCTION.SET_FEE_RECEIVER]),
      params.newFeeReceiver.toBuffer(),
    ])
  );
}

export function buildCreateConditionalMetadataIx(
  params: ConditionalMetadataParams,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  return buildConditionalMetadataIx(params, true, programId);
}

export function buildUpdateConditionalMetadataIx(
  params: ConditionalMetadataParams,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  return buildConditionalMetadataIx(params, false, programId);
}

function buildConditionalMetadataIx(
  params: ConditionalMetadataParams,
  isCreate: boolean,
  programId: PublicKey
): TransactionInstruction {
  if (
    !Number.isInteger(params.outcomeIndex) ||
    params.outcomeIndex < 0 ||
    params.outcomeIndex >= MAX_OUTCOMES
  ) {
    throw ProgramSdkError.invalidOutcomeIndex(params.outcomeIndex, MAX_OUTCOMES - 1);
  }

  const [exchange] = getExchangePda(programId);
  const [conditionalMint] = getConditionalMintPda(
    params.market,
    params.depositMint,
    params.outcomeIndex,
    programId
  );
  const [mintAuthority] = getMintAuthorityPda(params.market, programId);
  const [metadata] = getMplMetadataPda(conditionalMint);

  const keys: AccountMeta[] = [
    isCreate ? signerMut(params.manager) : signer(params.manager),
    readonly(exchange),
    readonly(params.market),
    readonly(params.depositMint),
    readonly(conditionalMint),
    writable(metadata),
    readonly(mintAuthority),
    readonly(MPL_TOKEN_METADATA_PROGRAM_ID),
  ];

  if (isCreate) {
    keys.push(readonly(SYSTEM_PROGRAM_ID));
    keys.push(readonly(RENT_SYSVAR_ID));
  }

  return publicInstruction(
    programId,
    keys,
    Buffer.concat([
      Buffer.from([
        isCreate
          ? INSTRUCTION.CREATE_CONDITIONAL_METADATA
          : INSTRUCTION.UPDATE_CONDITIONAL_METADATA,
        params.outcomeIndex,
      ]),
      serializeConditionalMetadata(params.name, params.symbol, params.uri),
    ])
  );
}

/**
 * Build WhitelistDepositToken instruction
 *
 * Accounts:
 * 0. authority (signer, mut)
 * 1. exchange (mut) - deposit_token_count is incremented
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

  return publicInstruction(programId, keys, Buffer.from([INSTRUCTION.WHITELIST_DEPOSIT_TOKEN]));
}

/**
 * Build SetDepositTokenStatus instruction.
 *
 * Sets the live trading permission for this registered collateral.
 * Deposits, preparation, splits, merges, and exits retain their existing rules.
 */
export function buildSetDepositTokenStatusIx(
  params: SetDepositTokenStatusParams,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  const [exchange] = getExchangePda(programId);
  const [globalDepositToken] = getGlobalDepositTokenPda(params.mint, programId);

  const keys: AccountMeta[] = [
    signer(params.manager),
    readonly(exchange),
    writable(globalDepositToken),
  ];

  return publicInstruction(
    programId,
    keys,
    Buffer.from([
      INSTRUCTION.SET_DEPOSIT_TOKEN_STATUS,
      params.active ? 1 : 0,
    ])
  );
}

/**
 * Deposit integer collateral units into the user's global custody account.
 *
 * Business accounts: user, GDT, collateral mint, user global custody, source ATA,
 * Token program, System program, and exchange. Two event trailers follow.
 * Data is exactly discriminator 17 and amount:u64 LE. Nonce initialization is separate.
 */
export function buildDepositToGlobalIx(
  params: DepositToGlobalParams,
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
    readonly(SYSTEM_PROGRAM_ID),
    readonly(exchange),
  ];

  const dataBuffers = [
    Buffer.from([INSTRUCTION.DEPOSIT_TO_GLOBAL]),
    toU64Le(params.amount),
  ];

  return publicInstruction(programId, keys, Buffer.concat(dataBuffers));
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
 * 10. ata_program (readonly)
 * 11. system_program (readonly)
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

  return publicInstruction(programId, keys, data);
}

/**
 * Prepare a beneficiary's Position and conditional ATAs without minting balances.
 *
 * A signing payer can prepare initial, partial, repeated, or additional collateral groups.
 * Existing valid accounts remain in place. The beneficiary must be nonzero and on curve.
 * Supply 1..8 distinct mints in increasing global GDT registration-index order.
 * This synchronous builder preserves that order. It does not fetch registration indices.
 *
 * The nine business-prefix accounts are payer, user, exchange, market, position,
 * mint authority, Token program, ATA program, and System program.
 * Each group adds collateral mint, vault, GDT, then each conditional mint and position ATA.
 * The complete reference count is 11 + G*(3 + 2*O), including the two event trailers.
 * Data is exactly [19, groupCount]. Each successful retry emits a preparation event.
 */
export function buildInitPositionTokensIx(
  params: InitPositionTokensParams,
  numOutcomes: number,
  programId: PublicKey = PROGRAM_ID
): TransactionInstruction {
  validateOutcomes(numOutcomes);
  if (params.depositMints.length === 0) {
    throw ProgramSdkError.missingField("deposit_mints");
  }
  validateUser(params.user);
  if (params.depositMints.length > MAX_DEPOSIT_MINTS_PER_IX) {
    throw ProgramSdkError.tooManyDepositMints(params.depositMints.length);
  }

  if (new Set(params.depositMints.map((mint) => mint.toBase58())).size !== params.depositMints.length) {
    throw ProgramSdkError.invalidDepositMintOrder();
  }

  const [exchange] = getExchangePda(programId);
  const [position] = getPositionPda(params.user, params.market, programId);
  const [mintAuthority] = getMintAuthorityPda(params.market, programId);

  const keys: AccountMeta[] = [
    signerMut(params.payer),
    readonly(params.user),
    readonly(exchange),
    readonly(params.market),
    writable(position),
    readonly(mintAuthority),
    readonly(TOKEN_PROGRAM_ID),
    readonly(ASSOCIATED_TOKEN_PROGRAM_ID),
    readonly(SYSTEM_PROGRAM_ID),
  ];

  for (const depositMint of params.depositMints) {
    const [vault] = getVaultPda(depositMint, params.market, programId);
    const [gdt] = getGlobalDepositTokenPda(depositMint, programId);
    keys.push(readonly(depositMint));
    keys.push(readonly(vault));
    keys.push(readonly(gdt));

    for (const [conditionalMint] of getAllConditionalMintPdas(
      params.market, depositMint, numOutcomes, programId
    )) {
      keys.push(readonly(conditionalMint));
      keys.push(writable(getConditionalTokenAta(conditionalMint, position)));
    }
  }

  const data = Buffer.concat([
    Buffer.from([INSTRUCTION.INIT_POSITION_TOKENS]),
    toU8(params.depositMints.length),
  ]);

  return publicInstruction(programId, keys, data);
}

/**
 * Match 1..11 makers with optional global-collateral funding for each participant.
 *
 * Fixed accounts: operator, exchange, market, orderbook, canonical GDT A and GDT B,
 * mint authority, Token program, fee receiver quote ATA, fee receiver, and ATA program.
 * Funding mints must back the participant's signed give side: BUY quote, SELL base.
 * Every funded block includes collateral, vault, GDT, global custody, and every outcome pair.
 * Maker settlement ATAs follow the taker's receive/give order on both funding paths.
 *
 * The full-fill and funding masks are independent u16 LE values, with taker bit 15.
 * Data length is 107 + 117*M. Fill amounts use integer units given by each participant.
 * Business references total 19 + 5*M - F + D*(4 + 2*O), then two event trailers.
 * Repeated GDT and mint addresses retain their positions in this sequence.
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

  validateTradingOrders(params, params.makers.map((maker) => maker.order));
  for (const maker of params.makers) {
    if (maker.isDeposit) validateFundingMint(maker.order, maker.depositMint, params);
  }
  if (params.takerIsDeposit) validateFundingMint(params.takerOrder, params.takerDepositMint, params);
  const [gdtA, gdtB] = tradingDepositTokens(params, programId);

  const [exchange] = getExchangePda(programId);
  const [orderbook] = getOrderbookPda(params.baseMint, params.quoteMint, programId);
  const [mintAuthority] = getMintAuthorityPda(params.market, programId);
  const [takerNonce] = getUserNoncePda(params.takerOrder.maker, programId);
  const [takerPosition] = getPositionPda(params.takerOrder.maker, params.market, programId);
  const feeReceiverQuoteAta = getConditionalTokenAta(
    params.quoteMint,
    params.feeReceiver
  );
  const [receiveMint, giveMint] =
    params.takerOrder.side === OrderSide.BID
      ? [params.baseMint, params.quoteMint]
      : [params.quoteMint, params.baseMint];

  let fullFillBitmask = 0;
  let depositBitmask = 0;

  if (params.takerIsFullFill) {
    fullFillBitmask |= TAKER_MASK;
  }
  if (params.takerIsDeposit) {
    depositBitmask |= TAKER_MASK;
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
    readonly(gdtA),
    readonly(gdtB),
    readonly(mintAuthority),
    readonly(TOKEN_PROGRAM_ID),
    writable(feeReceiverQuoteAta),
    readonly(params.feeReceiver),
    readonly(ASSOCIATED_TOKEN_PROGRAM_ID),
  ];

  if (!params.takerIsFullFill) {
    const takerOrderHash = hashOrder(params.takerOrder);
    const [takerOrderStatus] = getOrderStatusPda(takerOrderHash, programId);
    keys.push(writable(takerOrderStatus));
  }

  keys.push(readonly(takerNonce));
  keys.push(readonly(takerPosition));
  keys.push(readonly(params.baseMint));
  keys.push(readonly(params.quoteMint));
  keys.push(writable(getConditionalTokenAta(receiveMint, takerPosition)));
  keys.push(writable(getConditionalTokenAta(giveMint, takerPosition)));
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
    keys.push(readonly(makerPosition));

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
    participantMaskBytes(fullFillBitmask, params.makers.length),
    participantMaskBytes(depositBitmask, params.makers.length),
  ];

  for (const maker of params.makers) {
    buffers.push(serializeOrder(signedOrderToOrder(maker.order)));
    buffers.push(maker.order.signature);
    buffers.push(toU64Le(maker.makerFillAmount));
    buffers.push(toU64Le(maker.takerFillAmount));
  }

  return publicInstruction(programId, keys, Buffer.concat(buffers));
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

  return publicInstruction(programId, keys, data);
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

  return publicInstruction(programId, keys, data);
}

/**
 * Build ClosePositionTokenAccounts instruction
 *
 * Accounts:
 * 0. operator (signer, mut)
 * 1. exchange (readonly)
 * 2. market (readonly)
 * 3. position (readonly)
 * 4. token_program (readonly)
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
    readonly(TOKEN_PROGRAM_ID),
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

  return publicInstruction(
    programId,
    keys,
    Buffer.from([INSTRUCTION.CLOSE_POSITION_TOKEN_ACCOUNTS])
  );
}

/**
 * Close a resolved orderbook and refund its lamports to the operator.
 *
 * Business accounts are operator (signer, writable), exchange (read-only),
 * orderbook (writable), and market (read-only), followed by the two event trailers.
 * Data is exactly the single discriminator byte 27.
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
  ];

  return publicInstruction(programId, keys, Buffer.from([INSTRUCTION.CLOSE_ORDERBOOK]));
}

// ============================================================================
// TRANSACTION BUILDERS (_tx convenience wrappers)
// Each compiles the corresponding instruction with an explicit v1 context.
// ============================================================================

export function buildInitializeTx(
  params: InitializeParams,
  context: V1TransactionContext,
  programId: PublicKey = PROGRAM_ID
): V1Transaction {
  const ix = buildInitializeIx(params, programId);
  return V1Transaction.compile([ix], params.authority, context);
}

export function buildCreateMarketTx(
  params: CreateMarketParams,
  marketId: bigint,
  context: V1TransactionContext,
  programId: PublicKey = PROGRAM_ID
): V1Transaction {
  const ix = buildCreateMarketIx(params, marketId, programId);
  return V1Transaction.compile([ix], params.manager, context);
}

export function buildAddDepositMintTx(
  params: AddDepositMintParams,
  market: PublicKey,
  numOutcomes: number,
  context: V1TransactionContext,
  programId: PublicKey = PROGRAM_ID
): V1Transaction {
  const ix = buildAddDepositMintIx(params, market, numOutcomes, programId);
  return V1Transaction.compile([ix], params.manager, context);
}

export function buildDepositTx(
  params: BuildDepositParams,
  numOutcomes: number,
  context: V1TransactionContext,
  programId: PublicKey = PROGRAM_ID
): V1Transaction {
  const ix = buildDepositIx(params, numOutcomes, programId);
  return V1Transaction.compile([ix], params.user, context);
}

export function buildMergeTx(
  params: BuildMergeParams,
  numOutcomes: number,
  context: V1TransactionContext,
  programId: PublicKey = PROGRAM_ID
): V1Transaction {
  const ix = buildMergeIx(params, numOutcomes, programId);
  return V1Transaction.compile([ix], params.user, context);
}

export function buildCancelOrderTx(
  operator: PublicKey,
  market: PublicKey,
  order: SignedOrder,
  context: V1TransactionContext,
  programId: PublicKey = PROGRAM_ID
): V1Transaction {
  const ix = buildCancelOrderIx(operator, market, order, programId);
  return V1Transaction.compile([ix], operator, context);
}

export function buildIncrementNonceTx(
  user: PublicKey,
  context: V1TransactionContext,
  programId: PublicKey = PROGRAM_ID
): V1Transaction {
  const ix = buildIncrementNonceIx(user, programId);
  return V1Transaction.compile([ix], user, context);
}

export function buildSettleMarketTx(
  params: SettleMarketParams,
  context: V1TransactionContext,
  programId: PublicKey = PROGRAM_ID
): V1Transaction {
  const ix = buildSettleMarketIx(params, programId);
  return V1Transaction.compile([ix], params.oracle, context);
}

export function buildRedeemWinningsTx(
  params: RedeemWinningsParams,
  outcomeIndex: number,
  context: V1TransactionContext,
  programId: PublicKey = PROGRAM_ID
): V1Transaction {
  const ix = buildRedeemWinningsIx(params, outcomeIndex, programId);
  return V1Transaction.compile([ix], params.user, context);
}

export function buildSetPausedTx(
  authority: PublicKey,
  paused: boolean,
  context: V1TransactionContext,
  programId: PublicKey = PROGRAM_ID
): V1Transaction {
  const ix = buildSetPausedIx(authority, paused, programId);
  return V1Transaction.compile([ix], authority, context);
}

export function buildSetOperatorTx(
  authority: PublicKey,
  newOperator: PublicKey,
  context: V1TransactionContext,
  programId: PublicKey = PROGRAM_ID
): V1Transaction {
  const ix = buildSetOperatorIx(authority, newOperator, programId);
  return V1Transaction.compile([ix], authority, context);
}

export function buildWithdrawConditionalFromPositionTx(
  params: WithdrawConditionalFromPositionParams,
  context: V1TransactionContext,
  programId: PublicKey = PROGRAM_ID
): V1Transaction {
  const ix = buildWithdrawConditionalFromPositionIx(params, programId);
  return V1Transaction.compile([ix], params.user, context);
}

export function buildWithdrawFromPositionTx(
  params: WithdrawFromPositionParams,
  context: V1TransactionContext,
  programId: PublicKey = PROGRAM_ID
): V1Transaction {
  return buildWithdrawConditionalFromPositionTx(params, context, programId);
}

export function buildActivateMarketTx(
  params: ActivateMarketParams,
  context: V1TransactionContext,
  programId: PublicKey = PROGRAM_ID
): V1Transaction {
  const ix = buildActivateMarketIx(params, programId);
  return V1Transaction.compile([ix], params.manager, context);
}

export function buildMatchOrdersMultiTx(
  params: MatchOrdersMultiParams,
  context: V1TransactionContext,
  programId: PublicKey = PROGRAM_ID
): V1Transaction {
  const ix = buildMatchOrdersMultiIx(params, programId);
  return V1Transaction.compile([ix], params.operator, context);
}

export function buildSetAuthorityTx(
  params: SetAuthorityParams,
  context: V1TransactionContext,
  programId: PublicKey = PROGRAM_ID
): V1Transaction {
  const ix = buildSetAuthorityIx(params, programId);
  return V1Transaction.compile([ix], params.currentAuthority, context);
}

export function buildSetManagerTx(
  params: SetManagerParams,
  context: V1TransactionContext,
  programId: PublicKey = PROGRAM_ID
): V1Transaction {
  const ix = buildSetManagerIx(params, programId);
  return V1Transaction.compile([ix], params.authority, context);
}

export function buildAcceptAuthorityTx(
  params: AcceptRoleParams,
  context: V1TransactionContext,
  programId: PublicKey = PROGRAM_ID
): V1Transaction {
  const ix = buildAcceptAuthorityIx(params, programId);
  return V1Transaction.compile([ix], params.incomingRole, context);
}

export function buildAcceptManagerTx(
  params: AcceptRoleParams,
  context: V1TransactionContext,
  programId: PublicKey = PROGRAM_ID
): V1Transaction {
  const ix = buildAcceptManagerIx(params, programId);
  return V1Transaction.compile([ix], params.incomingRole, context);
}

export function buildAcceptOperatorTx(
  params: AcceptRoleParams,
  context: V1TransactionContext,
  programId: PublicKey = PROGRAM_ID
): V1Transaction {
  const ix = buildAcceptOperatorIx(params, programId);
  return V1Transaction.compile([ix], params.incomingRole, context);
}

export function buildSetOracleTx(
  params: SetOracleParams,
  context: V1TransactionContext,
  programId: PublicKey = PROGRAM_ID
): V1Transaction {
  const ix = buildSetOracleIx(params, programId);
  return V1Transaction.compile([ix], params.authority, context);
}

export function buildSetMarketFeesTx(
  params: SetMarketFeesParams,
  context: V1TransactionContext,
  programId: PublicKey = PROGRAM_ID
): V1Transaction {
  const ix = buildSetMarketFeesIx(params, programId);
  return V1Transaction.compile([ix], params.manager, context);
}

export function buildSetFeeReceiverTx(
  params: SetFeeReceiverParams,
  context: V1TransactionContext,
  programId: PublicKey = PROGRAM_ID
): V1Transaction {
  const ix = buildSetFeeReceiverIx(params, programId);
  return V1Transaction.compile([ix], params.authority, context);
}

export function buildSetFeeReceiverWithAtasTx(
  params: SetFeeReceiverWithAtasParams,
  context: V1TransactionContext,
  programId: PublicKey = PROGRAM_ID
): V1Transaction {
  const ix = buildSetFeeReceiverWithAtasIx(params, programId);
  return V1Transaction.compile([ix], params.authority, context);
}

export function buildCreateConditionalMetadataTx(
  params: ConditionalMetadataParams,
  context: V1TransactionContext,
  programId: PublicKey = PROGRAM_ID
): V1Transaction {
  const ix = buildCreateConditionalMetadataIx(params, programId);
  return V1Transaction.compile([ix], params.manager, context);
}

export function buildUpdateConditionalMetadataTx(
  params: ConditionalMetadataParams,
  context: V1TransactionContext,
  programId: PublicKey = PROGRAM_ID
): V1Transaction {
  const ix = buildUpdateConditionalMetadataIx(params, programId);
  return V1Transaction.compile([ix], params.manager, context);
}

export function buildCreateOrderbookTx(
  params: CreateOrderbookParams,
  context: V1TransactionContext,
  programId: PublicKey = PROGRAM_ID
): V1Transaction {
  const ix = buildCreateOrderbookIx(params, programId);
  return V1Transaction.compile([ix], params.manager, context);
}

export function buildWhitelistDepositTokenTx(
  params: WhitelistDepositTokenParams,
  context: V1TransactionContext,
  programId: PublicKey = PROGRAM_ID
): V1Transaction {
  const ix = buildWhitelistDepositTokenIx(params, programId);
  return V1Transaction.compile([ix], params.authority, context);
}

export function buildSetDepositTokenStatusTx(
  params: SetDepositTokenStatusParams,
  context: V1TransactionContext,
  programId: PublicKey = PROGRAM_ID
): V1Transaction {
  const ix = buildSetDepositTokenStatusIx(params, programId);
  return V1Transaction.compile([ix], params.manager, context);
}

export function buildDepositToGlobalTx(
  params: DepositToGlobalParams,
  context: V1TransactionContext,
  programId: PublicKey = PROGRAM_ID
): V1Transaction {
  const ix = buildDepositToGlobalIx(params, programId);
  return V1Transaction.compile([ix], params.user, context);
}

export function buildGlobalToMarketDepositTx(
  params: GlobalToMarketDepositParams,
  numOutcomes: number,
  context: V1TransactionContext,
  programId: PublicKey = PROGRAM_ID
): V1Transaction {
  const ix = buildGlobalToMarketDepositIx(params, numOutcomes, programId);
  return V1Transaction.compile([ix], params.user, context);
}

export function buildInitPositionTokensTx(
  params: InitPositionTokensParams,
  numOutcomes: number,
  context: V1TransactionContext,
  programId: PublicKey = PROGRAM_ID
): V1Transaction {
  const ix = buildInitPositionTokensIx(params, numOutcomes, programId);
  return V1Transaction.compile([ix], params.payer, context);
}

export function buildDepositAndSwapTx(
  params: DepositAndSwapParams,
  context: V1TransactionContext,
  programId: PublicKey = PROGRAM_ID
): V1Transaction {
  const ix = buildDepositAndSwapIx(params, programId);
  return V1Transaction.compile([ix], params.operator, context);
}

export function buildWithdrawFromGlobalTx(
  params: WithdrawFromGlobalParams,
  context: V1TransactionContext,
  programId: PublicKey = PROGRAM_ID
): V1Transaction {
  const ix = buildWithdrawFromGlobalIx(params, programId);
  return V1Transaction.compile([ix], params.user, context);
}

export function buildCloseOrderStatusTx(
  params: CloseOrderStatusParams,
  context: V1TransactionContext,
  programId: PublicKey = PROGRAM_ID
): V1Transaction {
  const ix = buildCloseOrderStatusIx(params, programId);
  return V1Transaction.compile([ix], params.operator, context);
}

export function buildClosePositionTokenAccountsTx(
  params: ClosePositionTokenAccountsParams,
  numOutcomes: number,
  context: V1TransactionContext,
  programId: PublicKey = PROGRAM_ID
): V1Transaction {
  const ix = buildClosePositionTokenAccountsIx(params, numOutcomes, programId);
  return V1Transaction.compile([ix], params.operator, context);
}

export function buildCloseOrderbookTx(
  params: CloseOrderbookParams,
  context: V1TransactionContext,
  programId: PublicKey = PROGRAM_ID
): V1Transaction {
  const ix = buildCloseOrderbookIx(params, programId);
  return V1Transaction.compile([ix], params.operator, context);
}
