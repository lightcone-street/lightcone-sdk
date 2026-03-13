import type { OrderBookId, Resolution } from "../../shared";
import type { LineData } from "./index";
import type { DepositTokenCandle } from "./wire";

export class PriceHistoryState {
  private readonly data: Map<string, LineData[]>;

  constructor() {
    this.data = new Map();
  }

  applySnapshot(orderbookId: OrderBookId, resolution: Resolution, prices: LineData[]): void {
    this.data.set(keyFor(orderbookId, resolution), prices);
  }

  applyUpdate(orderbookId: OrderBookId, resolution: Resolution, point: LineData): void {
    const key = keyFor(orderbookId, resolution);
    const existing = this.data.get(key) ?? [];

    const last = existing.at(-1);
    if (last && last.time === point.time) {
      last.value = point.value;
      this.data.set(key, existing);
      return;
    }

    existing.push(point);
    this.data.set(key, existing);
  }

  get(orderbookId: OrderBookId, resolution: Resolution): readonly LineData[] | undefined {
    return this.data.get(keyFor(orderbookId, resolution));
  }

  clear(): void {
    this.data.clear();
  }
}

function keyFor(orderbookId: OrderBookId, resolution: Resolution): string {
  return `${orderbookId}:${resolution}`;
}

export interface LatestDepositPrice {
  price: string;
  eventTime: number;
}

export class DepositPriceState {
  private readonly candles: Map<string, DepositTokenCandle[]>;
  private readonly latestPrice: Map<string, LatestDepositPrice>;

  constructor() {
    this.candles = new Map();
    this.latestPrice = new Map();
  }

  applySnapshot(depositAsset: string, resolution: Resolution, prices: DepositTokenCandle[]): void {
    this.candles.set(depositKeyFor(depositAsset, resolution), prices);
  }

  applyCandleUpdate(depositAsset: string, resolution: Resolution, candle: DepositTokenCandle): void {
    const key = depositKeyFor(depositAsset, resolution);
    const existing = this.candles.get(key) ?? [];

    const last = existing.at(-1);
    if (last && last.t === candle.t) {
      last.c = candle.c;
      last.tc = candle.tc;
      this.candles.set(key, existing);
      return;
    }

    existing.push(candle);
    this.candles.set(key, existing);
  }

  applyPriceTick(depositAsset: string, price: string, eventTime: number): void {
    this.latestPrice.set(depositAsset, { price, eventTime });
  }

  getCandles(depositAsset: string, resolution: Resolution): readonly DepositTokenCandle[] | undefined {
    return this.candles.get(depositKeyFor(depositAsset, resolution));
  }

  getLatestPrice(depositAsset: string): LatestDepositPrice | undefined {
    return this.latestPrice.get(depositAsset);
  }

  clear(): void {
    this.candles.clear();
    this.latestPrice.clear();
  }
}

function depositKeyFor(depositAsset: string, resolution: Resolution): string {
  return `${depositAsset}:${resolution}`;
}
