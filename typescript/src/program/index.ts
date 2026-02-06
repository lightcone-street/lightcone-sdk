/**
 * On-chain program interaction module for Lightcone.
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
  Orderbook,
  SignedOrder,
  Order,
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
  SetAuthorityParams,
  CreateOrderbookParams,
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
  ALT_PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  TOKEN_2022_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  SYSTEM_PROGRAM_ID,
  RENT_SYSVAR_ID,
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
  toU32Le,
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
  getOrderbookPda,
  getAltPda,
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
  deserializeOrderbook,
  isExchangeAccount,
  isMarketAccount,
  isPositionAccount,
  isOrderStatusAccount,
  isUserNonceAccount,
  isOrderbookAccount,
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
  buildSetAuthorityIx,
  buildCreateOrderbookIx,
} from "./instructions";

// ============================================================================
// ORDER UTILITIES
// ============================================================================
export {
  hashOrder,
  hashOrderHex,
  getOrderMessage,
  signOrder,
  signOrderFull,
  verifyOrderSignature,
  serializeSignedOrder,
  deserializeSignedOrder,
  serializeOrder,
  deserializeOrder,
  signedOrderToOrder,
  createBidOrder,
  createAskOrder,
  createSignedBidOrder,
  createSignedAskOrder,
  isOrderExpired,
  ordersCanCross,
  calculateTakerFill,
  signatureHex,
  isSigned,
  deriveOrderbookId,
  toSubmitRequest,
} from "./orders";

// ============================================================================
// ORDER BUILDER
// ============================================================================
export { OrderBuilder } from "./builder";
