import { PublicKey } from "@solana/web3.js";

/**
 * Lightcone Pinocchio Program ID (Devnet deployment)
 */
export const PROGRAM_ID = new PublicKey(
  "Aumw7EC9nnxDjQFzr1fhvXvnG3Rn3Bb5E3kbcbLrBdEk"
);

/**
 * SPL Token Program ID
 */
export const TOKEN_PROGRAM_ID = new PublicKey(
  "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA"
);

/**
 * Token-2022 Program ID (for conditional tokens)
 */
export const TOKEN_2022_PROGRAM_ID = new PublicKey(
  "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb"
);

/**
 * Associated Token Account Program ID
 */
export const ASSOCIATED_TOKEN_PROGRAM_ID = new PublicKey(
  "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL"
);

/**
 * System Program ID
 */
export const SYSTEM_PROGRAM_ID = new PublicKey(
  "11111111111111111111111111111111"
);

/**
 * Rent Sysvar ID
 */
export const RENT_SYSVAR_ID = new PublicKey(
  "SysvarRent111111111111111111111111111111111"
);

/**
 * Instructions Sysvar ID (for Ed25519 verification)
 */
export const INSTRUCTIONS_SYSVAR_ID = new PublicKey(
  "Sysvar1nstructions1111111111111111111111111"
);

/**
 * Ed25519 Program ID (for signature verification)
 */
export const ED25519_PROGRAM_ID = new PublicKey(
  "Ed25519SigVerify111111111111111111111111111"
);

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
} as const;

/**
 * Account discriminators (8 bytes each)
 * These are the prefixes used to identify account types
 */
export const DISCRIMINATOR = {
  EXCHANGE: Buffer.from("exchange"),
  MARKET: Buffer.from("market\0\0"),
  ORDER_STATUS: Buffer.from("ordstat\0"),
  USER_NONCE: Buffer.from("usrnonce"),
  POSITION: Buffer.from("position"),
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
} as const;

/**
 * Order sizes in bytes
 */
export const ORDER_SIZE = {
  FULL: 225,
  COMPACT: 65,
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
  ORDER_STATUS: "order_status",
  USER_NONCE: "user_nonce",
  POSITION: "position",
} as const;
