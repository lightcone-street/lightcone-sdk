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
