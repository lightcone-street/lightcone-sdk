import { PublicKey } from "@solana/web3.js";
import { PROGRAM_ID, SEEDS } from "./constants";
import { toU64Le, toU8 } from "./utils";

/**
 * Derive Exchange PDA (singleton central state)
 * Seeds: ["central_state"]
 */
export function getExchangePda(
  programId: PublicKey = PROGRAM_ID
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(SEEDS.CENTRAL_STATE)],
    programId
  );
}

/**
 * Derive Market PDA
 * Seeds: ["market", market_id (u64 little-endian)]
 */
export function getMarketPda(
  marketId: bigint,
  programId: PublicKey = PROGRAM_ID
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(SEEDS.MARKET), toU64Le(marketId)],
    programId
  );
}

/**
 * Derive Vault PDA (deposit token storage)
 * Seeds: ["market_deposit_token_account", deposit_mint, market]
 */
export function getVaultPda(
  depositMint: PublicKey,
  market: PublicKey,
  programId: PublicKey = PROGRAM_ID
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [
      Buffer.from(SEEDS.MARKET_DEPOSIT_TOKEN_ACCOUNT),
      depositMint.toBuffer(),
      market.toBuffer(),
    ],
    programId
  );
}

/**
 * Derive Mint Authority PDA (signs mint/burn operations)
 * Seeds: ["market_mint_authority", market]
 */
export function getMintAuthorityPda(
  market: PublicKey,
  programId: PublicKey = PROGRAM_ID
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(SEEDS.MARKET_MINT_AUTHORITY), market.toBuffer()],
    programId
  );
}

/**
 * Derive Conditional Mint PDA (per outcome)
 * Seeds: ["conditional_mint", market, deposit_mint, outcome_index (u8)]
 */
export function getConditionalMintPda(
  market: PublicKey,
  depositMint: PublicKey,
  outcomeIndex: number,
  programId: PublicKey = PROGRAM_ID
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [
      Buffer.from(SEEDS.CONDITIONAL_MINT),
      market.toBuffer(),
      depositMint.toBuffer(),
      toU8(outcomeIndex),
    ],
    programId
  );
}

/**
 * Derive all Conditional Mint PDAs for a market
 * @param market - The market pubkey
 * @param depositMint - The deposit mint pubkey
 * @param numOutcomes - Number of outcomes (2-6)
 * @returns Array of [conditionalMint, bump] tuples
 */
export function getAllConditionalMintPdas(
  market: PublicKey,
  depositMint: PublicKey,
  numOutcomes: number,
  programId: PublicKey = PROGRAM_ID
): [PublicKey, number][] {
  const pdas: [PublicKey, number][] = [];
  for (let i = 0; i < numOutcomes; i++) {
    pdas.push(getConditionalMintPda(market, depositMint, i, programId));
  }
  return pdas;
}

/**
 * Derive Order Status PDA
 * Seeds: ["order_status", order_hash (32 bytes)]
 */
export function getOrderStatusPda(
  orderHash: Buffer,
  programId: PublicKey = PROGRAM_ID
): [PublicKey, number] {
  if (orderHash.length !== 32) {
    throw new Error("Order hash must be 32 bytes");
  }
  return PublicKey.findProgramAddressSync(
    [Buffer.from(SEEDS.ORDER_STATUS), orderHash],
    programId
  );
}

/**
 * Derive User Nonce PDA
 * Seeds: ["user_nonce", user (32 bytes)]
 */
export function getUserNoncePda(
  user: PublicKey,
  programId: PublicKey = PROGRAM_ID
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(SEEDS.USER_NONCE), user.toBuffer()],
    programId
  );
}

/**
 * Derive Position PDA
 * Seeds: ["position", owner (32 bytes), market (32 bytes)]
 * Note: Does NOT include deposit_mint - one Position per market per user
 */
export function getPositionPda(
  owner: PublicKey,
  market: PublicKey,
  programId: PublicKey = PROGRAM_ID
): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [Buffer.from(SEEDS.POSITION), owner.toBuffer(), market.toBuffer()],
    programId
  );
}

/**
 * Collection of all PDA functions for easy access
 */
export const pda = {
  getExchangePda,
  getMarketPda,
  getVaultPda,
  getMintAuthorityPda,
  getConditionalMintPda,
  getAllConditionalMintPdas,
  getOrderStatusPda,
  getUserNoncePda,
  getPositionPda,
};
