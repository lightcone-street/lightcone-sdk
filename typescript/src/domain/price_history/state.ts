import type { OrderBookId, Resolution } from "../../shared";
import type { LineData } from "./index";

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
