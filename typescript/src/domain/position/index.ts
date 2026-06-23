import Decimal from "decimal.js";
import { asOrderBookId } from "../../shared";
import type { OrderBookId, PubkeyStr } from "../../shared";
import { display } from "../../shared/fmt/decimal";
import { userOutcomeBalanceIsZero } from "../order/wire";
import type { UserMarketBalance, UserOutcomeBalance } from "../order/wire";

export * from "./builders";
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
  short_symbol: string;
  name: string;
  deposit_asset: PubkeyStr;
  icon_url_low: string;
  icon_url_medium: string;
  icon_url_high: string;
  description: string | null;
  decimals: number;
}

export interface DepositTokenBalance {
  mint: PubkeyStr;
  idle: string;
  symbol: string;
  name: string;
  icon_url_low?: string;
  icon_url_medium?: string;
  icon_url_high?: string;
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

// ─── ConditionalBalanceDelta ─────────────────────────────────────────────────

/** An incremental change to a user's balance for one conditional token. */
export interface ConditionalBalanceDelta {
  marketPubkey: PubkeyStr;
  orderbookId?: OrderBookId;
  outcomeIndex: number;
  conditionalToken: PubkeyStr;
  idle: string;
  onBook: string;
}

/** Full-precision sum of idle + on-book (mirrors the `balance` field). */
export function conditionalDeltaTotal(delta: ConditionalBalanceDelta): string {
  return new Decimal(delta.idle).plus(delta.onBook).toString();
}

/** True when the delta holds nothing idle and nothing resting on the book. */
export function conditionalDeltaIsZero(delta: ConditionalBalanceDelta): boolean {
  return !(new Decimal(delta.idle).gt(0) || new Decimal(delta.onBook).gt(0));
}

export function conditionalDeltaToTokenBalance(delta: ConditionalBalanceDelta): TokenBalance {
  return {
    mint: delta.conditionalToken,
    idle: delta.idle,
    onBook: delta.onBook,
    tokenType: {
      kind: "ConditionalToken",
      orderbookId: delta.orderbookId ?? asOrderBookId(""),
      marketPubkey: delta.marketPubkey,
      outcomeIndex: delta.outcomeIndex,
    },
  };
}

export function conditionalDeltaToOutcomeBalance(
  delta: ConditionalBalanceDelta
): UserOutcomeBalance {
  return {
    outcome_index: delta.outcomeIndex,
    conditional_token: delta.conditionalToken,
    balance: conditionalDeltaTotal(delta),
    balance_idle: delta.idle,
    balance_on_book: delta.onBook,
  };
}

// ─── UserMarketBalanceIndex ──────────────────────────────────────────────────

export type ConditionalTokenBalanceIndex = Map<PubkeyStr, UserOutcomeBalance>;
export type DepositAssetBalanceIndex = Map<PubkeyStr, ConditionalTokenBalanceIndex>;

/**
 * Nested index of a user's conditional-token balances, keyed
 * `market → deposit_asset → conditional_token`. Zero balances are dropped when
 * building from wire records.
 */
export class UserMarketBalanceIndex {
  readonly index: Map<PubkeyStr, DepositAssetBalanceIndex>;

  constructor() {
    this.index = new Map();
  }

  get(marketPubkey: PubkeyStr): DepositAssetBalanceIndex | undefined {
    return this.index.get(marketPubkey);
  }

  insert(marketPubkey: PubkeyStr, marketEntry: DepositAssetBalanceIndex): void {
    this.index.set(marketPubkey, marketEntry);
  }

  extend(other: UserMarketBalanceIndex): void {
    for (const [marketPubkey, marketEntry] of other.index.entries()) {
      const existing = this.index.get(marketPubkey);
      if (!existing) {
        this.index.set(marketPubkey, marketEntry);
        continue;
      }
      for (const [depositAsset, outcomes] of marketEntry.entries()) {
        existing.set(depositAsset, outcomes);
      }
    }
  }

  remove(marketPubkey: PubkeyStr): void {
    this.index.delete(marketPubkey);
  }

  inner(): Map<PubkeyStr, DepositAssetBalanceIndex> {
    return this.index;
  }

  marketPubkeys(): PubkeyStr[] {
    return [...this.index.keys()].sort();
  }

  isEmpty(): boolean {
    return this.index.size === 0;
  }

  static fromUserMarketBalance(
    marketBalance: UserMarketBalance
  ): UserMarketBalanceIndex | undefined {
    const marketEntry: DepositAssetBalanceIndex = new Map();

    for (const depositAssetBalance of marketBalance.deposit_assets) {
      const outcomes: ConditionalTokenBalanceIndex = new Map();
      for (const outcome of depositAssetBalance.outcomes) {
        if (!userOutcomeBalanceIsZero(outcome)) {
          outcomes.set(outcome.conditional_token, outcome);
        }
      }
      if (outcomes.size > 0) {
        marketEntry.set(depositAssetBalance.deposit_asset, outcomes);
      }
    }

    if (marketEntry.size === 0) {
      return undefined;
    }

    const index = new UserMarketBalanceIndex();
    index.insert(marketBalance.market_pubkey, marketEntry);
    return index;
  }

  static fromUserMarketBalances(
    marketBalances: UserMarketBalance[]
  ): UserMarketBalanceIndex {
    const index = new UserMarketBalanceIndex();
    for (const marketBalance of marketBalances) {
      const marketIndex = UserMarketBalanceIndex.fromUserMarketBalance(marketBalance);
      if (marketIndex) {
        index.extend(marketIndex);
      }
    }
    return index;
  }
}
