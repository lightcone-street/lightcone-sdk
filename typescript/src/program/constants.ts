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
  WHITELIST_DEPOSIT_TOKEN: 16,
  DEPOSIT_TO_GLOBAL: 17,
  GLOBAL_TO_MARKET_DEPOSIT: 18,
  INIT_POSITION_TOKENS: 19,
  DEPOSIT_AND_SWAP: 20,
  EXTEND_POSITION_TOKENS: 21,
  WITHDRAW_FROM_GLOBAL: 22,
  CLOSE_POSITION_ALT: 23,
  CLOSE_ORDER_STATUS: 24,
  CLOSE_POSITION_TOKEN_ACCOUNTS: 25,
  CLOSE_ORDERBOOK_ALT: 26,
  CLOSE_ORDERBOOK: 27,
  SET_MANAGER: 28,
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
  GLOBAL_DEPOSIT_TOKEN: Buffer.from([0x25, 0xbe, 0xa1, 0xe8, 0x7b, 0x92, 0x2a, 0x57]),
} as const;

/**
 * Account sizes in bytes
 */
export const ACCOUNT_SIZE = {
  EXCHANGE: 120,
  MARKET: 148,
  ORDER_STATUS: 32,
  USER_NONCE: 16,
  POSITION: 80,
  ORDERBOOK: 144,
  GLOBAL_DEPOSIT_TOKEN: 48,
} as const;

/**
 * Order sizes in bytes
 */
export const ORDER_SIZE = {
  SIGNED_ORDER: 233,
  ORDER: 37,
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
export const MAX_MAKERS = 5;

/**
 * PDA Seeds
 */
export const SEEDS = {
  CENTRAL_STATE: "central_state",
  MARKET: "market",
  MARKET_DEPOSIT_TOKEN_ACCOUNT: "market_deposit_token_account",
  MARKET_MINT_AUTHORITY: "market_mint_authority",
  CONDITIONAL_MINT: "conditional_mint",
  CONDITION: "condition",
  ORDER_STATUS: "order_status",
  USER_NONCE: "user_nonce",
  POSITION: "position",
  ORDERBOOK: "orderbook",
  GLOBAL_DEPOSIT: "global_deposit",
} as const;
