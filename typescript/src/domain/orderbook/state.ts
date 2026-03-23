import BTree from "sorted-btree";
import Decimal from "decimal.js";
import type { OrderBookId } from "../../shared";
import type { OrderBook } from "./wire";

const decimalCompare = (a: string, b: string): number =>
  new Decimal(a).cmp(new Decimal(b));

export class OrderbookSnapshot {
  readonly orderbookId: OrderBookId;
  seq: number;
  private bidsTree: BTree<string, string>;
  private asksTree: BTree<string, string>;

  constructor(orderbookId: OrderBookId) {
    this.orderbookId = orderbookId;
    this.seq = 0;
    this.bidsTree = new BTree<string, string>(undefined, decimalCompare);
    this.asksTree = new BTree<string, string>(undefined, decimalCompare);
  }

  apply(book: OrderBook): void {
    if (book.is_snapshot) {
      this.bidsTree.clear();
      this.asksTree.clear();
    }

    this.seq = book.seq ?? this.seq;

    for (const level of book.bids) {
      if (new Decimal(level.size).isZero()) {
        this.bidsTree.delete(level.price);
      } else {
        this.bidsTree.set(level.price, level.size);
      }
    }

    for (const level of book.asks) {
      if (new Decimal(level.size).isZero()) {
        this.asksTree.delete(level.price);
      } else {
        this.asksTree.set(level.price, level.size);
      }
    }
  }

  bids(): BTree<string, string> {
    return this.bidsTree;
  }

  asks(): BTree<string, string> {
    return this.asksTree;
  }

  bestBid(): string | undefined {
    return this.bidsTree.maxKey();
  }

  bestAsk(): string | undefined {
    return this.asksTree.minKey();
  }

  midPrice(): string | undefined {
    const bid = this.bestBid();
    const ask = this.bestAsk();
    if (!bid || !ask) {
      return undefined;
    }

    return new Decimal(bid).plus(ask).div(2).toString();
  }

  spread(): string | undefined {
    const bid = this.bestBid();
    const ask = this.bestAsk();
    if (!bid || !ask) {
      return undefined;
    }

    return new Decimal(ask).minus(bid).toString();
  }

  isEmpty(): boolean {
    return this.bidsTree.size === 0 && this.asksTree.size === 0;
  }

  clear(): void {
    this.bidsTree.clear();
    this.asksTree.clear();
    this.seq = 0;
  }
}
