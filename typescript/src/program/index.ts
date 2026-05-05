/**
 * On-chain program interaction module for Lightcone.
 */

// ============================================================================
// ENVELOPE / BUILDER
// ============================================================================
export { LimitOrderEnvelope, TriggerOrderEnvelope, type OrderEnvelope } from "./envelope";
export { ProgramSdkError, ProgramSdkError as SdkError } from "./error";
export type { ProgramResult, ProgramResult as SdkResult } from "./error";

// ============================================================================
// TYPES
// ============================================================================
export {
  MarketStatus,
  OrderSide,
} from "./types";

export type {
  Exchange,
  GlobalDepositToken,
  Market,
  PayoutNumerators,
  Position,
  OrderStatus,
  UserNonce,
  Orderbook,
  SignedOrder,
  OrderPayload,
  Order,
  InitializeParams,
  CreateMarketParams,
  OutcomeMetadata,
  AddDepositMintParams,
  BuildDepositParams,
  BuildMergeParams,
  CancelOrderParams,
  IncrementNonceParams,
  SettleMarketParams,
  ScalarResolutionParams,
  RedeemWinningsParams,
  SetPausedParams,
  SetOperatorParams,
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
  MakerFill,
  DepositAndSwapParams,
  WithdrawFromGlobalParams,
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
export { PROGRAM_ID } from "../env";
export {
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
  winnerTakesAllPayoutNumerators,
  scalarToPayoutNumerators,
} from "./utils";

// ============================================================================
// PDA FUNCTIONS
// ============================================================================
export {
  getExchangePda,
  getMarketPda,
  getConditionTombstonePda,
  getVaultPda,
  getMintAuthorityPda,
  getConditionalMintPda,
  getAllConditionalMintPdas,
  getOrderStatusPda,
  getUserNoncePda,
  getPositionPda,
  canonicalMintPair,
  getOrderbookPda,
  getAltPda,
  getGlobalDepositTokenPda,
  getUserGlobalDepositPda,
  getPositionAltPda,
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
  deserializeGlobalDepositToken,
  isExchangeAccount,
  isMarketAccount,
  isPositionAccount,
  isOrderStatusAccount,
  isUserNonceAccount,
  isOrderbookAccount,
  isGlobalDepositTokenAccount,
} from "./accounts";

// ============================================================================
// INSTRUCTION BUILDERS
// ============================================================================
export {
  buildInitializeIx,
  buildCreateMarketIx,
  buildAddDepositMintIx,
  buildDepositIx,
  buildMergeIx,
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
  buildSetManagerIx,
  buildCreateOrderbookIx,
  buildWhitelistDepositTokenIx,
  buildDepositToGlobalIx,
  buildDepositToGlobalIxWithAlt,
  buildGlobalToMarketDepositIx,
  buildInitPositionTokensIx,
  buildExtendPositionTokensIx,
  buildDepositAndSwapIx,
  buildWithdrawFromGlobalIx,
  buildInitializeTx,
  buildCreateMarketTx,
  buildAddDepositMintTx,
  buildDepositTx,
  buildMergeTx,
  buildCancelOrderTx,
  buildIncrementNonceTx,
  buildSettleMarketTx,
  buildRedeemWinningsTx,
  buildSetPausedTx,
  buildSetOperatorTx,
  buildWithdrawFromPositionTx,
  buildActivateMarketTx,
  buildMatchOrdersMultiTx,
  buildSetAuthorityTx,
  buildSetManagerTx,
  buildCreateOrderbookTx,
  buildWhitelistDepositTokenTx,
  buildDepositToGlobalTx,
  buildDepositToGlobalTxWithAlt,
  buildGlobalToMarketDepositTx,
  buildInitPositionTokensTx,
  buildExtendPositionTokensTx,
  buildDepositAndSwapTx,
  buildWithdrawFromGlobalTx,
} from "./instructions";

// ============================================================================
// ORDER UTILITIES
// ============================================================================
export {
  generateSalt,
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
  applySignature,
  orderToSigned,
  deriveOrderbookId,
  toSubmitRequest,
  cancelOrderMessage,
  signCancelOrder,
  cancelTriggerOrderMessage,
  signCancelTriggerOrder,
  cancelAllMessage,
  generateCancelAllSalt,
  signCancelAll,
  is_order_expired,
  orders_can_cross,
  calculate_taker_fill,
  cancel_order_message,
  cancel_trigger_order_message,
  cancel_all_message,
  generate_cancel_all_salt,
  derive_condition_id,
  apply_signature,
  order_to_signed,
} from "./orders";

// ============================================================================
// ORDER BUILDER
// ============================================================================
export { OrderBuilder } from "./builder";
