// ============================================================================
// LIGHTCONE PINOCCHIO SDK
// ============================================================================

// Client
export { LightconePinocchioClient } from "./client";

// ============================================================================
// TYPES - Enums
// ============================================================================
export { MarketStatus, OrderSide } from "./types";

// ============================================================================
// TYPES - Accounts
// ============================================================================
export type {
  Exchange,
  Market,
  Position,
  OrderStatus,
  UserNonce,
} from "./types";

// ============================================================================
// TYPES - Orders
// ============================================================================
export type { FullOrder, CompactOrder } from "./types";

// ============================================================================
// TYPES - Parameters
// ============================================================================
export type {
  InitializeParams,
  CreateMarketParams,
  AddDepositMintParams,
  OutcomeMetadata,
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
  BidOrderParams,
  AskOrderParams,
} from "./types";

// ============================================================================
// TYPES - Build Results
// ============================================================================
export type {
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

// ============================================================================
// UTILITY FUNCTIONS
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
// RE-EXPORTS FROM DEPENDENCIES
// ============================================================================
export {
  PublicKey,
  Connection,
  Keypair,
  Transaction,
  TransactionInstruction,
} from "@solana/web3.js";
