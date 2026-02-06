/**
 * Lightcone SDK - TypeScript SDK for the Lightcone protocol on Solana.
 *
 * This SDK provides three main modules:
 * - `program`: On-chain program interaction (smart contract)
 * - `api`: REST API client for market data, orders, and positions
 * - `websocket`: Real-time data streaming for orderbooks, trades, and user events
 *
 * @example
 * ```typescript
 * import { LightconePinocchioClient, PROGRAM_ID, api, websocket } from "@lightcone/sdk";
 *
 * // On-chain program interaction
 * const programClient = new LightconePinocchioClient(connection, payer);
 *
 * // REST API client
 * const apiClient = new api.LightconeApiClient();
 * const markets = await apiClient.getMarkets();
 *
 * // WebSocket client for real-time data
 * const wsClient = await websocket.LightconeWebSocketClient.connectDefault();
 * await wsClient.subscribeBookUpdates(["market1:ob1"]);
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
 * REST API client module.
 * Provides HTTP client functionality for market data, orders, and positions.
 */
export * as api from "./api";

/**
 * WebSocket client module.
 * Provides real-time data streaming for orderbooks, trades, and user events.
 */
export * as websocket from "./websocket";

/**
 * Authentication module.
 * Provides authentication functionality shared by API and WebSocket clients.
 */
export * as auth from "./auth";

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
