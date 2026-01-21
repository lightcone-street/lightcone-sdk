/**
 * Price history candle resolution.
 * Used by both REST API and WebSocket for price history queries.
 */
export enum Resolution {
  /** 1 minute candles */
  OneMinute = "1m",
  /** 5 minute candles */
  FiveMinutes = "5m",
  /** 15 minute candles */
  FifteenMinutes = "15m",
  /** 1 hour candles */
  OneHour = "1h",
  /** 4 hour candles */
  FourHours = "4h",
  /** 1 day candles */
  OneDay = "1d",
}
