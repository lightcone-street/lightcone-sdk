import { PublicKey, Transaction } from "@solana/web3.js";

// ============================================================================
// ENUMS
// ============================================================================

/**
 * Market status enum matching on-chain representation
 */
export enum MarketStatus {
  Pending = 0,
  Active = 1,
  Resolved = 2,
  Cancelled = 3,
}

/**
 * Order side enum
 * BID = buyer wants base tokens (pays quote)
 * ASK = seller offers base tokens (receives quote)
 */
export enum OrderSide {
  BID = 0,
  ASK = 1,
}

// ============================================================================
// ACCOUNT TYPES
// ============================================================================

/**
 * Exchange account - singleton central state
 * PDA: ["central_state"]
 * Size: 88 bytes
 */
export interface Exchange {
  discriminator: Buffer; // 8 bytes
  authority: PublicKey; // 32 bytes - initial admin
  operator: PublicKey; // 32 bytes - can perform operational tasks
  marketCount: bigint; // u64 - incremented for each market
  paused: boolean; // u8 - 0 = active, 1 = paused
  bump: number; // u8
}

/**
 * Market account
 * PDA: ["market", market_id (u64)]
 * Size: 120 bytes
 */
export interface Market {
  discriminator: Buffer; // 8 bytes
  marketId: bigint; // u64 - auto-assigned, sequential
  numOutcomes: number; // u8 - 2-6 outcomes supported
  status: MarketStatus; // u8
  winningOutcome: number; // u8
  hasWinningOutcome: boolean; // u8
  bump: number; // u8
  oracle: PublicKey; // 32 bytes - who can settle the market
  questionId: Buffer; // 32 bytes
  conditionId: Buffer; // 32 bytes - derived from oracle + questionId + numOutcomes
}

/**
 * Order status account - tracks partial fills and cancellations
 * PDA: ["order_status", order_hash (32 bytes)]
 * Size: 24 bytes
 */
export interface OrderStatus {
  discriminator: Buffer; // 8 bytes
  remaining: bigint; // u64 - maker_amount not yet filled
  isCancelled: boolean; // u8
}

/**
 * User nonce account - replay protection
 * PDA: ["user_nonce", user_pubkey (32 bytes)]
 * Size: 16 bytes
 */
export interface UserNonce {
  discriminator: Buffer; // 8 bytes
  nonce: bigint; // u64 - incremented per order
}

/**
 * Position account - user's state in a market
 * PDA: ["position", owner (32 bytes), market (32 bytes)]
 * Size: 80 bytes
 */
export interface Position {
  discriminator: Buffer; // 8 bytes
  owner: PublicKey; // 32 bytes
  market: PublicKey; // 32 bytes
  bump: number; // u8
}

/**
 * Orderbook account - links market to token pair and lookup table
 * PDA: ["orderbook", mint_a (32 bytes), mint_b (32 bytes)]
 * Size: 144 bytes
 */
export interface Orderbook {
  discriminator: Buffer; // 8 bytes
  market: PublicKey; // 32 bytes
  mintA: PublicKey; // 32 bytes
  mintB: PublicKey; // 32 bytes
  lookupTable: PublicKey; // 32 bytes
  bump: number; // u8
}

// ============================================================================
// ORDER TYPES
// ============================================================================

/**
 * Signed order format (225 bytes)
 * Full order with all fields for submission, cancellation, and hashing
 */
export interface SignedOrder {
  nonce: bigint; // u64 - order ID + replay protection
  maker: PublicKey; // 32 bytes - signer
  market: PublicKey; // 32 bytes
  baseMint: PublicKey; // 32 bytes - token being bought/sold
  quoteMint: PublicKey; // 32 bytes - payment token
  side: OrderSide; // u8 - 0=BID, 1=ASK
  makerAmount: bigint; // u64 - what maker gives
  takerAmount: bigint; // u64 - what maker receives
  expiration: bigint; // i64 - unix timestamp, 0=no expiration
  signature: Buffer; // 64 bytes - Ed25519 signature
}

/**
 * Compact order format (29 bytes)
 * Transaction-optimized version: nonce is u32, no maker field (derived on-chain from Position PDA)
 */
export interface Order {
  nonce: number; // u32 (4 bytes) - truncated from SignedOrder's u64 nonce
  side: OrderSide; // u8
  makerAmount: bigint; // u64
  takerAmount: bigint; // u64
  expiration: bigint; // i64
}

// ============================================================================
// PARAMETER TYPES
// ============================================================================

/**
 * Parameters for initialize instruction
 */
export interface InitializeParams {
  authority: PublicKey;
}

/**
 * Parameters for createMarket instruction
 */
export interface CreateMarketParams {
  authority: PublicKey;
  numOutcomes: number; // 2-6
  oracle: PublicKey;
  questionId: Buffer; // 32 bytes
}

/**
 * Metadata for a single outcome token
 */
export interface OutcomeMetadata {
  name: string;
  symbol: string;
  uri: string;
}

/**
 * Parameters for addDepositMint instruction
 */
export interface AddDepositMintParams {
  authority: PublicKey;
  marketId: bigint;
  depositMint: PublicKey;
  outcomeMetadata: OutcomeMetadata[];
}

/**
 * Parameters for mintCompleteSet instruction
 */
export interface MintCompleteSetParams {
  user: PublicKey;
  market: PublicKey;
  depositMint: PublicKey;
  amount: bigint;
}

/**
 * Parameters for mergeCompleteSet instruction
 */
export interface MergeCompleteSetParams {
  user: PublicKey;
  market: PublicKey;
  depositMint: PublicKey;
  amount: bigint;
}

/**
 * Parameters for cancelOrder instruction
 */
export interface CancelOrderParams {
  maker: PublicKey;
  order: SignedOrder;
}

/**
 * Parameters for incrementNonce instruction
 */
export interface IncrementNonceParams {
  user: PublicKey;
}

/**
 * Parameters for settleMarket instruction
 */
export interface SettleMarketParams {
  oracle: PublicKey;
  marketId: bigint;
  winningOutcome: number;
}

/**
 * Parameters for redeemWinnings instruction
 */
export interface RedeemWinningsParams {
  user: PublicKey;
  market: PublicKey;
  depositMint: PublicKey;
  amount: bigint;
}

/**
 * Parameters for setPaused instruction
 */
export interface SetPausedParams {
  authority: PublicKey;
  paused: boolean;
}

/**
 * Parameters for setOperator instruction
 */
export interface SetOperatorParams {
  authority: PublicKey;
  newOperator: PublicKey;
}

/**
 * Parameters for withdrawFromPosition instruction
 */
export interface WithdrawFromPositionParams {
  user: PublicKey;
  market: PublicKey;
  mint: PublicKey; // Can be deposit mint or conditional mint
  amount: bigint;
  outcomeIndex: number; // u8 outcome index
}

/**
 * Parameters for activateMarket instruction
 */
export interface ActivateMarketParams {
  authority: PublicKey;
  marketId: bigint;
}

/**
 * Parameters for matchOrdersMulti instruction
 */
export interface MatchOrdersMultiParams {
  operator: PublicKey;
  market: PublicKey;
  baseMint: PublicKey;
  quoteMint: PublicKey;
  takerOrder: SignedOrder;
  makerOrders: SignedOrder[];
  makerFillAmounts: bigint[]; // Per maker - what each maker gives
  takerFillAmounts: bigint[]; // Per maker - what taker gives to each maker
  fullFillBitmask: number; // u8 bitmask: bit 7 = taker, bits 0..n = makers
}

/**
 * Parameters for setAuthority instruction
 */
export interface SetAuthorityParams {
  currentAuthority: PublicKey;
  newAuthority: PublicKey;
}

/**
 * Parameters for createOrderbook instruction
 */
export interface CreateOrderbookParams {
  payer: PublicKey;
  market: PublicKey;
  mintA: PublicKey;
  mintB: PublicKey;
  recentSlot: bigint;
}

// ============================================================================
// BUILDER RESULT TYPES
// ============================================================================

/**
 * Result from transaction builders
 */
export interface BuildResult<T = Record<string, PublicKey>> {
  /** Unsigned transaction ready for signing */
  transaction: Transaction;
  /** Key accounts involved in the transaction */
  accounts: T;
  /** Serialize transaction to base64 */
  serialize: () => string;
}

/**
 * Accounts returned from initialize
 */
export interface InitializeAccounts {
  exchange: PublicKey;
}

/**
 * Accounts returned from createMarket
 */
export interface CreateMarketAccounts {
  exchange: PublicKey;
  market: PublicKey;
}

/**
 * Accounts returned from addDepositMint
 */
export interface AddDepositMintAccounts {
  market: PublicKey;
  vault: PublicKey;
  mintAuthority: PublicKey;
  conditionalMints: PublicKey[];
}

/**
 * Accounts returned from mintCompleteSet
 */
export interface MintCompleteSetAccounts {
  position: PublicKey;
  vault: PublicKey;
  conditionalMints: PublicKey[];
}

/**
 * Accounts returned from mergeCompleteSet
 */
export interface MergeCompleteSetAccounts {
  position: PublicKey;
  vault: PublicKey;
  conditionalMints: PublicKey[];
}

/**
 * Accounts returned from cancelOrder
 */
export interface CancelOrderAccounts {
  orderStatus: PublicKey;
}

/**
 * Accounts returned from incrementNonce
 */
export interface IncrementNonceAccounts {
  userNonce: PublicKey;
}

/**
 * Accounts returned from settleMarket
 */
export interface SettleMarketAccounts {
  exchange: PublicKey;
  market: PublicKey;
}

/**
 * Accounts returned from redeemWinnings
 */
export interface RedeemWinningsAccounts {
  position: PublicKey;
  vault: PublicKey;
  winningMint: PublicKey;
}

/**
 * Accounts returned from activateMarket
 */
export interface ActivateMarketAccounts {
  exchange: PublicKey;
  market: PublicKey;
}

/**
 * Accounts returned from matchOrdersMulti
 */
export interface MatchOrdersMultiAccounts {
  takerOrderStatus: PublicKey;
  takerPosition: PublicKey;
  makerOrderStatuses: PublicKey[];
  makerPositions: PublicKey[];
}

// ============================================================================
// ORDER CREATION TYPES
// ============================================================================

/**
 * Parameters for creating a bid order
 */
export interface BidOrderParams {
  nonce: bigint;
  maker: PublicKey;
  market: PublicKey;
  baseMint: PublicKey; // Token to buy
  quoteMint: PublicKey; // Token to pay with
  makerAmount: bigint; // Quote tokens to give
  takerAmount: bigint; // Base tokens to receive
  expiration?: bigint; // 0 = no expiration
}

/**
 * Parameters for creating an ask order
 */
export interface AskOrderParams {
  nonce: bigint;
  maker: PublicKey;
  market: PublicKey;
  baseMint: PublicKey; // Token to sell
  quoteMint: PublicKey; // Token to receive
  makerAmount: bigint; // Base tokens to give
  takerAmount: bigint; // Quote tokens to receive
  expiration?: bigint; // 0 = no expiration
}
