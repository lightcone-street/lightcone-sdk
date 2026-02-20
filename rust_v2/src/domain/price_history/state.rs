//! Price history state containers — app-owned, SDK-provided update logic.

use super::LineData;
use crate::shared::{OrderBookId, Resolution};
use std::collections::HashMap;

/// Live price history state for one orderbook + resolution.
///
/// The app owns instances of this type. The SDK provides update methods.
#[derive(Debug, Clone, Default)]
pub struct PriceHistoryState {
    data: HashMap<(OrderBookId, Resolution), Vec<LineData>>,
}

impl PriceHistoryState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Apply a WS price history snapshot (replaces all data for this key).
    pub fn apply_snapshot(
        &mut self,
        orderbook_id: OrderBookId,
        resolution: Resolution,
        prices: Vec<LineData>,
    ) {
        self.data.insert((orderbook_id, resolution), prices);
    }

    /// Apply a WS price history update (appends or updates last candle).
    pub fn apply_update(
        &mut self,
        orderbook_id: OrderBookId,
        resolution: Resolution,
        point: LineData,
    ) {
        let entry = self
            .data
            .entry((orderbook_id, resolution))
            .or_default();

        if let Some(last) = entry.last_mut() {
            if last.time == point.time {
                last.value = point.value;
                return;
            }
        }
        entry.push(point);
    }

    pub fn get(&self, orderbook_id: &OrderBookId, resolution: &Resolution) -> Option<&Vec<LineData>> {
        self.data.get(&(orderbook_id.clone(), *resolution))
    }

    pub fn clear(&mut self) {
        self.data.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line_data(time: u64, value: &str) -> LineData {
        LineData {
            time,
            value: value.to_string(),
        }
    }

    #[test]
    fn test_apply_snapshot() {
        let mut state = PriceHistoryState::new();
        let ob = OrderBookId::from("ob1");
        let res = Resolution::Minute1;
        state.apply_snapshot(ob.clone(), res, vec![line_data(100, "50.0"), line_data(200, "51.0")]);
        let data = state.get(&ob, &res).unwrap();
        assert_eq!(data.len(), 2);
        assert_eq!(data[0].value, "50.0");
        assert_eq!(data[1].value, "51.0");
    }

    #[test]
    fn test_apply_update_appends() {
        let mut state = PriceHistoryState::new();
        let ob = OrderBookId::from("ob1");
        let res = Resolution::Minute1;
        state.apply_snapshot(ob.clone(), res, vec![line_data(100, "50.0")]);
        state.apply_update(ob.clone(), res, line_data(200, "51.0"));
        let data = state.get(&ob, &res).unwrap();
        assert_eq!(data.len(), 2);
        assert_eq!(data[1].value, "51.0");
    }

    #[test]
    fn test_apply_update_same_time_overwrites() {
        let mut state = PriceHistoryState::new();
        let ob = OrderBookId::from("ob1");
        let res = Resolution::Minute1;
        state.apply_snapshot(ob.clone(), res, vec![line_data(100, "50.0")]);
        state.apply_update(ob.clone(), res, line_data(100, "50.5"));
        let data = state.get(&ob, &res).unwrap();
        assert_eq!(data.len(), 1);
        assert_eq!(data[0].value, "50.5");
    }
}
