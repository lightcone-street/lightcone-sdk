/**
 * On-chain program interaction module for Lightcone.
 *
 * This module provides the client and utilities for interacting with
 * the Lightcone smart contract on Solana.
 */

// ============================================================================
// CLIENT
// ============================================================================
export { LightconePinocchioClient } from "./client";

// ============================================================================
// TYPES
// ============================================================================
export {
  MarketStatus,
  OrderSide,
} from "./types";

export type {
  Exchange,
  Market,
  Position,
  OrderStatus,
  UserNonce,
  FullOrder,
  CompactOrder,
  InitializeParams,
  CreateMarketParams,
  OutcomeMetadata,
  AddDepositMintParams,
  MintCompleteSetParams,
  MergeCompleteSetParams,
  CancelOrderParams,
  IncrementNonceParams,
  SettleMarketParams,
  RedeemWinningsParams,
  SetPausedParams,
  SetOperatorParams,
  WithdrawFromPositionParams,
  ActivateMarketParams,
  MatchOrdersMultiParams,
  BuildResult,
  InitializeAccounts,
  CreateMarketAccounts,
  AddDepositMintAccounts,
  MintCompleteSetAccounts,
  MergeCompleteSetAccounts,
  CancelOrderAccounts,
  IncrementNonceAccounts,
  SettleMarketAccounts,
  RedeemWinningsAccounts,
  ActivateMarketAccounts,
  MatchOrdersMultiAccounts,
  BidOrderParams,
  AskOrderParams,
} from "./types";

// ============================================================================
// CONSTANTS
// ============================================================================
export {
  PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  TOKEN_2022_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  SYSTEM_PROGRAM_ID,
  RENT_SYSVAR_ID,
  INSTRUCTIONS_SYSVAR_ID,
  ED25519_PROGRAM_ID,
  INSTRUCTION,
  DISCRIMINATOR,
  ACCOUNT_SIZE,
  ORDER_SIZE,
  MAX_OUTCOMES,
  MIN_OUTCOMES,
  MAX_MAKERS,
  SEEDS,
} from "./constants";

// ============================================================================
// UTILITIES
// ============================================================================
export {
  toLeBytes,
  fromLeBytes,
  toBeBytes,
  fromBeBytes,
  toU8,
  toU64Le,
  toI64Le,
  fromI64Le,
  keccak256,
  deriveConditionId,
  getAssociatedTokenAddress,
  getConditionalTokenAta,
  getDepositTokenAta,
  serializeString,
  deserializeString,
  validateOutcomes,
  validateOutcomeIndex,
  validate32Bytes,
} from "./utils";

// ============================================================================
// PDA FUNCTIONS
// ============================================================================
export {
  getExchangePda,
  getMarketPda,
  getVaultPda,
  getMintAuthorityPda,
  getConditionalMintPda,
  getAllConditionalMintPdas,
  getOrderStatusPda,
  getUserNoncePda,
  getPositionPda,
  pda,
} from "./pda";

// ============================================================================
// ACCOUNT DESERIALIZATION
// ============================================================================
export {
  deserializeExchange,
  deserializeMarket,
  deserializePosition,
  deserializeOrderStatus,
  deserializeUserNonce,
  isExchangeAccount,
  isMarketAccount,
  isPositionAccount,
  isOrderStatusAccount,
  isUserNonceAccount,
} from "./accounts";

// ============================================================================
// INSTRUCTION BUILDERS
// ============================================================================
export {
  buildInitializeIx,
  buildCreateMarketIx,
  buildAddDepositMintIx,
  buildMintCompleteSetIx,
  buildMergeCompleteSetIx,
  buildCancelOrderIx,
  buildIncrementNonceIx,
  buildSettleMarketIx,
  buildRedeemWinningsIx,
  buildSetPausedIx,
  buildSetOperatorIx,
  buildWithdrawFromPositionIx,
  buildActivateMarketIx,
  buildMatchOrdersMultiIx,
} from "./instructions";

// ============================================================================
// ORDER UTILITIES
// ============================================================================
export {
  hashOrder,
  getOrderMessage,
  signOrder,
  signOrderFull,
  verifyOrderSignature,
  serializeFullOrder,
  deserializeFullOrder,
  serializeCompactOrder,
  deserializeCompactOrder,
  createBidOrder,
  createAskOrder,
  createSignedBidOrder,
  createSignedAskOrder,
  isOrderExpired,
  ordersCanCross,
  calculateTakerFill,
} from "./orders";

// ============================================================================
// ED25519 SIGNATURE HELPERS
// ============================================================================
export type { Ed25519VerifyParams } from "./ed25519";
export {
  createEd25519VerifyInstruction,
  createEd25519VerifyInstructions,
  createOrderVerifyInstruction,
  buildMatchOrdersTransaction,
  orderToVerifyParams,
  createBatchEd25519VerifyInstruction,
  buildCompactMatchOrdersTransaction,
  createCrossRefEd25519Instructions,
  buildCrossRefMatchOrdersTransaction,
} from "./ed25519";
