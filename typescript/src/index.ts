/**
 * Lightcone SDK v2 - TypeScript implementation mirroring Rust SDK v2 layers.
 */

// Layer 1: Core
export * as shared from "./shared";
export * as domain from "./domain";
export * as program from "./program";
export * from "./error";
export * from "./env";
export * from "./network";

// Layer 2: Auth + Privy
export * as auth from "./auth";
export * as privy from "./privy";

// Layer 3: HTTP
export * as http from "./http";

// Layer 4: WebSocket
export * as ws from "./ws";

// Layer 5: High-level client
export { LightconeClient, LightconeClientBuilder } from "./client";
export { Rpc } from "./rpc";
export type { ClientContext, ConfirmedTransaction } from "./context";
export {
  requireConnection,
  requireSigningStrategy,
  resolveDepositSource,
  signAndSubmitTx,
  signAndSubmitTxConfirmed,
  signAndSubmitTxConfirmedWithSlot,
  signAndSubmitPreparedTxConfirmedWithSlot,
} from "./context";

// Convenience top-level type exports
export type {
  AuthCredentials,
  AuthMethod,
  ChainType,
  EmailAccountData,
  GoogleAccountData,
  LinkedIdentity,
  LinkedIdentitySelector,
  LinkedIdentityType,
  MaxSlippagePreferenceBody,
  PrivyEmbeddedWallet,
  RegisterPrivyRequest,
  SessionResponse,
  User,
  UserIdentity,
  UserPrivyData,
  XAccountData,
} from "./auth";
export {
  avatarUrl,
  displayName,
  identityText,
  tradingWallet,
  userPrivy,
  userXAccount,
  walletDisplayName,
} from "./auth";

export type {
  FavoriteMarkets,
  FavoriteMarketUpdate,
  GlobalDepositAssetsResult,
  MarketsClient,
  MarketsResult,
  MetricsClient,
  OrdersClient,
  OrderbooksClient,
  PositionsClient,
  PriceHistorySubClient,
  ReferralsClient,
  TradesClient,
} from "./prelude";

export * from "./prelude";
