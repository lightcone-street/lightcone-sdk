/**
 * Shared utilities, types, and constants used across all Lightcone SDK modules.
 */

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
