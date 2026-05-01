import type { OrderBookId, PubkeyStr } from "../../shared";
import type { OrderBookPair } from "../orderbook";
import type { Outcome } from "./outcome";
import type {
  ConditionalToken,
  DepositAsset,
  DepositAssetPair,
  TokenMetadata,
} from "./tokens";

export * from "./client";
export * from "./wire";
export * from "./outcome";
export * from "./tokens";
export { globalDepositAssetFromWire, marketFromWire, tryMarketFromWire } from "./convert";

export enum Status {
  Pending = "Pending",
  Active = "Active",
  Resolved = "Resolved",
  Cancelled = "Cancelled",
}

export function statusFromWire(value: string): Status | undefined {
  switch (value) {
    case Status.Pending:
      return Status.Pending;
    case Status.Active:
      return Status.Active;
    case Status.Resolved:
      return Status.Resolved;
    case Status.Cancelled:
      return Status.Cancelled;
    default:
      return undefined;
  }
}

export interface Market {
  id: number;
  pubkey: PubkeyStr;
  name: string;
  bannerImageUrlLow: string;
  bannerImageUrlMedium: string;
  bannerImageUrlHigh: string;
  iconUrlLow: string;
  iconUrlMedium: string;
  iconUrlHigh: string;
  featuredRank?: number;
  volume: string;
  slug: string;
  status: Status;
  createdAt: Date;
  activatedAt?: Date;
  settledAt?: Date;
  winningOutcome?: number;
  description: string;
  definition: string;
  category?: string;
  tags: string[];
  depositAssets: DepositAsset[];
  /**
   * Unique base/quote deposit-asset pairs derived from `orderbookPairs`
   * during wire→domain conversion. Deduplicated by `(base, quote)` pubkey.
   */
  depositAssetPairs: DepositAssetPair[];
  conditionalTokens: ConditionalToken[];
  outcomes: Outcome[];
  orderbookPairs: OrderBookPair[];
  orderbookIds: OrderBookId[];
  tokenMetadata: Record<string, TokenMetadata>;
}

export class MarketValidationError extends Error {
  readonly marketPubkey: string;
  readonly details: string[];

  constructor(marketPubkey: string, details: string[]) {
    super(`Market validation errors (${marketPubkey}): ${details.join("; ")}`);
    this.name = "MarketValidationError";
    this.marketPubkey = marketPubkey;
    this.details = details;
  }
}
