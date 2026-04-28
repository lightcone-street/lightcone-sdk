import type { OrderBookId, Resolution } from "../../shared";

export interface MidpointPriceCandle {
  t: number;
  m: string | null;
}

export interface OhlcvPriceCandle extends MidpointPriceCandle {
  o: string | null;
  h: string | null;
  l: string | null;
  c: string | null;
  v: string;
  bb: string | null;
  ba: string | null;
}

export type PriceCandle = MidpointPriceCandle | OhlcvPriceCandle;

export interface PriceHistorySnapshot {
  orderbook_id: OrderBookId;
  resolution: Resolution;
  prices: PriceCandle[];
  last_timestamp?: number;
  server_time?: number;
  include_ohlcv?: boolean;
}

export interface PriceHistoryUpdate {
  orderbook_id: OrderBookId;
  resolution: Resolution;
  t: number;
  m?: string;
  o?: string;
  h?: string;
  l?: string;
  c?: string;
  v?: string;
  bb?: string;
  ba?: string;
}

export interface PriceHistoryHeartbeat {
  server_time: number;
  last_processed?: number;
}

export type PriceHistory =
  | ({ event_type: "snapshot" } & PriceHistorySnapshot)
  | ({ event_type: "update" } & PriceHistoryUpdate)
  | ({ event_type: "heartbeat" } & PriceHistoryHeartbeat);

export interface DepositTokenCandle {
  t: number;
  tc: number;
  c: string;
}

/** Initial batch of historical candles sent on subscription. */
export interface DepositPriceSnapshot {
  event_type: "snapshot";
  deposit_asset: string;
  resolution: Resolution;
  prices: DepositTokenCandle[];
}

/** Real-time spot price tick, broadcast to all resolutions. */
export interface DepositPriceTick {
  event_type: "price";
  deposit_asset: string;
  price: string;
  event_time: number;
}

/** A single candle update for a specific resolution (e.g. a 1m candle closed). */
export interface DepositPriceCandleUpdate {
  event_type: "candle";
  deposit_asset: string;
  resolution: Resolution;
  t: number;
  tc: number;
  c: string;
}

export type DepositPrice =
  | DepositPriceSnapshot
  | DepositPriceTick
  | DepositPriceCandleUpdate;

interface OrderbookPriceHistoryResponseBase<TCandle extends PriceCandle> {
  orderbook_id: string;
  resolution: Resolution;
  prices: TCandle[];
  next_cursor: number | null;
  has_more: boolean;
  decimals: { price: number; volume: number };
}

export type OrderbookPriceHistoryResponse =
  | (OrderbookPriceHistoryResponseBase<MidpointPriceCandle> & { include_ohlcv: false })
  | (OrderbookPriceHistoryResponseBase<OhlcvPriceCandle> & { include_ohlcv: true });

export type PriceHistoryRestResponse = OrderbookPriceHistoryResponse;

export interface DepositTokenPriceHistoryResponse {
  deposit_asset: string;
  binance_symbol: string;
  resolution: Resolution;
  prices: DepositTokenCandle[];
  next_cursor: number | null;
  has_more: boolean;
}

/**
 * REST response for `GET /api/deposit-asset-prices-snapshot`.
 *
 * Map of mint -> latest price (Decimal-as-string). Covers every active mint
 * in `global_deposit_tokens` with a row in `deposit_token_prices` (live tick)
 * or a recent 1m candle close (fallback). Mints with neither are absent.
 */
export interface DepositAssetPricesSnapshotResponse {
  prices: Record<string, string>;
}

/** Snapshot payload sent on subscribe to `deposit_asset_price` for one asset. */
export interface DepositAssetPriceSnapshot {
  event_type: "snapshot";
  deposit_asset: string;
  price: string;
}

/** Live price tick for one deposit asset. */
export interface DepositAssetPriceTick {
  event_type: "price";
  deposit_asset: string;
  price: string;
  event_time: number;
}

export type DepositAssetPriceEvent =
  | DepositAssetPriceSnapshot
  | DepositAssetPriceTick;
