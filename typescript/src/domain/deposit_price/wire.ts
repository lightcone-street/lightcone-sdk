export interface DepositTokenCandle {
  t: number;
  tc: number;
  c: string;
}

export interface DepositPriceSnapshot {
  event_type: "snapshot";
  deposit_asset: string;
  resolution: string;
  prices: DepositTokenCandle[];
}

export interface DepositPriceTick {
  event_type: "price";
  deposit_asset: string;
  price: string;
  event_time: number;
}

export interface DepositPriceCandleUpdate {
  event_type: "candle";
  deposit_asset: string;
  resolution: string;
  t: number;
  tc: number;
  c: string;
}

export type DepositPrice =
  | DepositPriceSnapshot
  | DepositPriceTick
  | DepositPriceCandleUpdate;

export interface DepositTokenPriceHistoryResponse {
  deposit_asset: string;
  binance_symbol: string;
  quote_asset: string;
  resolution: string;
  prices: DepositTokenCandle[];
  next_cursor: number | null;
  has_more: boolean;
}
