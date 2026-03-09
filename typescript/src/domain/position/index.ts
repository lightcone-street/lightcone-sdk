import type { OrderBookId, PubkeyStr } from "../../shared";

export * from "./client";
export * from "./wire";

export interface Portfolio {
  userAddress: PubkeyStr;
  walletHoldings: WalletHolding[];
  positions: Position[];
  totalWalletValue: string;
  totalPositionsValue: string;
}

export interface Position {
  eventPubkey: PubkeyStr;
  eventName: string;
  eventImgSrc: string;
  outcomes: PositionOutcome[];
  totalValue: string;
  createdAt: Date;
}

export interface PositionOutcome {
  conditionId: number;
  conditionName: string;
  tokenMint: PubkeyStr;
  amount: string;
  usdValue: string;
}

export interface WalletHolding {
  tokenMint: PubkeyStr;
  symbol: string;
  amount: string;
  decimals: number;
  usdValue: string;
  imgSrc: string;
}

export type TokenBalanceTokenType =
  | { kind: "DepositAsset" }
  | {
      kind: "ConditionalToken";
      orderbookId: OrderBookId;
      marketPubkey: PubkeyStr;
      outcomeIndex: number;
    };

export interface TokenBalance {
  mint: PubkeyStr;
  idle: string;
  onBook: string;
  tokenType: TokenBalanceTokenType;
}

export interface TokenBalanceComputedBase {
  value: string;
  size: string;
  price: string;
}

export interface DepositAssetMetadata {
  symbol: string;
  name: string;
  iconUrl: string;
  value: string;
}

export interface DepositTokenBalance {
  mint: PubkeyStr;
  idle: string;
  symbol: string;
  name: string;
  iconUrl: string;
}
