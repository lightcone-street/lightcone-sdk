import {
  PublicKey,
  SystemProgram,
  SYSVAR_RENT_PUBKEY,
} from "@solana/web3.js";
import {
  TOKEN_PROGRAM_ID as SPL_TOKEN_PROGRAM_ID,
  TOKEN_2022_PROGRAM_ID as SPL_TOKEN_2022_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID as SPL_ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";

/**
 * Lightcone Pinocchio Program ID
 */
export const PROGRAM_ID = new PublicKey(
  "9cCFQnmWqWmZF3LNdAVWTh7ECGJK4tCVPtgPMcYum81A"
);

/**
 * Address Lookup Table Program ID
 */
export const ALT_PROGRAM_ID = new PublicKey(
  "AddressLookupTab1e1111111111111111111111111"
);

/**
 * SPL Token Program ID
 */
export const TOKEN_PROGRAM_ID = SPL_TOKEN_PROGRAM_ID;

/**
 * Token-2022 Program ID (for conditional tokens)
 */
export const TOKEN_2022_PROGRAM_ID = SPL_TOKEN_2022_PROGRAM_ID;

/**
 * Associated Token Account Program ID
 */
export const ASSOCIATED_TOKEN_PROGRAM_ID = SPL_ASSOCIATED_TOKEN_PROGRAM_ID;

/**
 * System Program ID
 */
export const SYSTEM_PROGRAM_ID = SystemProgram.programId;

/**
 * Rent Sysvar ID
 */
export const RENT_SYSVAR_ID = SYSVAR_RENT_PUBKEY;

/**
 * Instruction discriminators (single byte indices)
 */
export const INSTRUCTION = {
  INITIALIZE: 0,
  CREATE_MARKET: 1,
  ADD_DEPOSIT_MINT: 2,
  MINT_COMPLETE_SET: 3,
  MERGE_COMPLETE_SET: 4,
  CANCEL_ORDER: 5,
  INCREMENT_NONCE: 6,
  SETTLE_MARKET: 7,
  REDEEM_WINNINGS: 8,
  SET_PAUSED: 9,
  SET_OPERATOR: 10,
  WITHDRAW_FROM_POSITION: 11,
  ACTIVATE_MARKET: 12,
  MATCH_ORDERS_MULTI: 13,
  SET_AUTHORITY: 14,
  CREATE_ORDERBOOK: 15,
} as const;

/**
 * Account discriminators (8 bytes each)
 * SHA-256 hash bytes matching the on-chain program
 */
export const DISCRIMINATOR = {
  EXCHANGE: Buffer.from([0x1e, 0xc8, 0xdc, 0x95, 0x03, 0x3d, 0x68, 0x32]),
  MARKET: Buffer.from([0xdb, 0xbe, 0xd5, 0x37, 0x00, 0xe3, 0xc6, 0x9a]),
  ORDER_STATUS: Buffer.from([0x2e, 0x5a, 0xf1, 0x49, 0xb2, 0x68, 0x41, 0x03]),
  USER_NONCE: Buffer.from([0xeb, 0x85, 0x01, 0xf3, 0x12, 0x87, 0x58, 0xe0]),
  POSITION: Buffer.from([0xaa, 0xbc, 0x8f, 0xe4, 0x7a, 0x40, 0xf7, 0xd0]),
  ORDERBOOK: Buffer.from([0x2b, 0x22, 0x19, 0x71, 0xc3, 0x45, 0x48, 0x07]),
} as const;

/**
 * Account sizes in bytes
 */
export const ACCOUNT_SIZE = {
  EXCHANGE: 88,
  MARKET: 120,
  ORDER_STATUS: 24,
  USER_NONCE: 16,
  POSITION: 80,
  ORDERBOOK: 144,
} as const;

/**
 * Order sizes in bytes
 */
export const ORDER_SIZE = {
  SIGNED_ORDER: 225,
  ORDER: 29,
  SIGNATURE: 64,
} as const;

/**
 * Maximum number of outcomes per market (2-6)
 */
export const MAX_OUTCOMES = 6;

/**
 * Minimum number of outcomes per market
 */
export const MIN_OUTCOMES = 2;

/**
 * Maximum number of makers per match_orders_multi instruction
 */
export const MAX_MAKERS = 3;

/**
 * PDA Seeds
 */
export const SEEDS = {
  CENTRAL_STATE: "central_state",
  MARKET: "market",
  MARKET_DEPOSIT_TOKEN_ACCOUNT: "market_deposit_token_account",
  MARKET_MINT_AUTHORITY: "market_mint_authority",
  CONDITIONAL_MINT: "conditional_mint",
  ORDER_STATUS: "order_status",
  USER_NONCE: "user_nonce",
  POSITION: "position",
  ORDERBOOK: "orderbook",
} as const;
