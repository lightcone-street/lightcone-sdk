/**
 * Lightcone SDK - TypeScript SDK for the Lightcone protocol on Solana.
 *
 * This SDK provides three main modules:
 * - `program`: On-chain program interaction (smart contract)
 * - `api`: REST API client (coming soon)
 * - `websocket`: Real-time data streaming (coming soon)
 *
 * @example
 * ```typescript
 * import { LightconePinocchioClient, PROGRAM_ID } from "@lightcone/sdk";
 *
 * // Or import from specific modules
 * import { LightconePinocchioClient } from "@lightcone/sdk/program";
 * import { PROGRAM_ID } from "@lightcone/sdk/shared";
 * ```
 */

// ============================================================================
// MODULE EXPORTS
// ============================================================================

/**
 * On-chain program interaction module.
 * Contains the client and utilities for interacting with the Lightcone smart contract.
 */
export * from "./program";

/**
 * Shared utilities, types, and constants.
 * Used across all SDK modules.
 */
export * from "./shared";

/**
 * REST API client module (coming soon).
 */
export * as api from "./api";

/**
 * WebSocket client module (coming soon).
 */
export * as websocket from "./websocket";

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
