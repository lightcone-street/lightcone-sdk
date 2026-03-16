import Decimal from "decimal.js";
import type { OrderBookId, PubkeyStr } from "../../shared";
import { display } from "../../shared/fmt/decimal";

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
  value: Decimal;
}

export interface DepositTokenBalance {
  mint: PubkeyStr;
  idle: string;
  symbol: string;
  name: string;
  iconUrl: string;
}

export function computedBase(
  balance: TokenBalance,
  conditionalPrice: Decimal
): TokenBalanceComputedBase {
  const size = new Decimal(balance.idle).plus(balance.onBook);
  const value = size.mul(conditionalPrice);
  return {
    value: display(value),
    size: display(size),
    price: display(conditionalPrice),
  };
}

export function computedQuote(balance: TokenBalance): string {
  const size = new Decimal(balance.idle).plus(balance.onBook);
  return display(size);
}
