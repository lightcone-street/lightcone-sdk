//! State container for websocket deposit-price data.

use super::wire::DepositTokenCandle;
use crate::shared::{PubkeyStr, Resolution};
use std::collections::HashMap;

/// Latest tick price for a deposit asset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LatestDepositPrice {
    pub price: String,
    pub event_time: i64,
}

/// Live deposit-price state keyed by deposit asset + resolution.
#[derive(Debug, Clone, Default)]
pub struct DepositPriceState {
    candles: HashMap<(PubkeyStr, Resolution), Vec<DepositTokenCandle>>,
    latest_price: HashMap<PubkeyStr, LatestDepositPrice>,
}

impl DepositPriceState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Apply a websocket snapshot, replacing all candles for one key.
    pub fn apply_snapshot(
        &mut self,
        deposit_asset: PubkeyStr,
        resolution: Resolution,
        prices: Vec<DepositTokenCandle>,
    ) {
        self.candles.insert((deposit_asset, resolution), prices);
    }

    /// Apply a websocket candle update, appending or overwriting the last candle.
    pub fn apply_candle_update(
        &mut self,
        deposit_asset: PubkeyStr,
        resolution: Resolution,
        candle: DepositTokenCandle,
    ) {
        let entry = self.candles.entry((deposit_asset, resolution)).or_default();

        if let Some(last) = entry.last_mut() {
            if last.t == candle.t {
                last.tc = candle.tc;
                last.c = candle.c;
                return;
            }
        }

        entry.push(candle);
    }

    /// Apply an ongoing websocket price tick for one deposit asset.
    pub fn apply_price_tick(
        &mut self,
        deposit_asset: PubkeyStr,
        price: String,
        event_time: i64,
    ) {
        self.latest_price
            .insert(deposit_asset, LatestDepositPrice { price, event_time });
    }

    pub fn get_candles(
        &self,
        deposit_asset: &PubkeyStr,
        resolution: &Resolution,
    ) -> Option<&Vec<DepositTokenCandle>> {
        self.candles.get(&(deposit_asset.clone(), *resolution))
    }

    pub fn get_latest_price(&self, deposit_asset: &PubkeyStr) -> Option<&LatestDepositPrice> {
        self.latest_price.get(deposit_asset)
    }

    pub fn clear(&mut self) {
        self.candles.clear();
        self.latest_price.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn candle(t: i64, tc: i64, c: &str) -> DepositTokenCandle {
        DepositTokenCandle {
            t,
            tc,
            c: c.to_string(),
        }
    }

    #[test]
    fn test_apply_snapshot() {
        let mut state = DepositPriceState::new();
        let asset = PubkeyStr::from("asset1");
        let resolution = Resolution::Minute1;

        state.apply_snapshot(
            asset.clone(),
            resolution,
            vec![candle(100, 160, "1.00"), candle(160, 220, "1.01")],
        );

        let candles = state.get_candles(&asset, &resolution).unwrap();
        assert_eq!(candles.len(), 2);
        assert_eq!(candles[1].c, "1.01");
    }

    #[test]
    fn test_apply_candle_update_overwrites_last() {
        let mut state = DepositPriceState::new();
        let asset = PubkeyStr::from("asset1");
        let resolution = Resolution::Minute1;

        state.apply_snapshot(asset.clone(), resolution, vec![candle(100, 160, "1.00")]);
        state.apply_candle_update(asset.clone(), resolution, candle(100, 170, "1.05"));

        let candles = state.get_candles(&asset, &resolution).unwrap();
        assert_eq!(candles.len(), 1);
        assert_eq!(candles[0].tc, 170);
        assert_eq!(candles[0].c, "1.05");
    }

    #[test]
    fn test_apply_price_tick() {
        let mut state = DepositPriceState::new();
        let asset = PubkeyStr::from("asset1");

        state.apply_price_tick(asset.clone(), "1.23".to_string(), 1234);

        let latest = state.get_latest_price(&asset).unwrap();
        assert_eq!(latest.price, "1.23");
        assert_eq!(latest.event_time, 1234);
    }
}
