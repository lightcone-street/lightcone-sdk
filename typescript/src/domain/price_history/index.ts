import type { OrderBookId, Resolution } from "../../shared";

export * from "./client";
export * from "./state";
export * from "./wire";

export interface LineData {
  time: number;
  value: string;
}

export function lineDataFromCandle(candle: import("./wire").PriceCandle): LineData {
  return {
    time: candle.t,
    value: candle.m ?? "",
  };
}

export interface PriceHistoryKey {
  orderbookId: OrderBookId;
  resolution: Resolution;
}
