import type { Resolution } from "../../shared";
import type { DepositTokenCandle } from "./wire";

export class DepositPriceState {
  private readonly candles: Map<string, DepositTokenCandle[]>;
  private readonly latestPrice: Map<string, { price: string; eventTime: number }>;

  constructor() {
    this.candles = new Map();
    this.latestPrice = new Map();
  }

  applySnapshot(depositAsset: string, resolution: Resolution, prices: DepositTokenCandle[]): void {
    this.candles.set(keyFor(depositAsset, resolution), prices);
  }

  applyCandleUpdate(depositAsset: string, resolution: Resolution, candle: DepositTokenCandle): void {
    const key = keyFor(depositAsset, resolution);
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
    return this.candles.get(keyFor(depositAsset, resolution));
  }

  getLatestPrice(depositAsset: string): { price: string; eventTime: number } | undefined {
    return this.latestPrice.get(depositAsset);
  }

  clear(): void {
    this.candles.clear();
    this.latestPrice.clear();
  }
}

function keyFor(depositAsset: string, resolution: Resolution): string {
  return `${depositAsset}:${resolution}`;
}
