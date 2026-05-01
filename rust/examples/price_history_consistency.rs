mod common;

use common::{market_and_orderbook, other, rest_client, ExampleResult};
use futures_util::StreamExt;
use lightcone::domain::price_history::wire::PriceHistory;
use lightcone::prelude::*;
use tokio::time::{timeout_at, Duration, Instant};

/// Compares price history data from the REST API vs WebSocket snapshot
/// for the same orderbook + resolution to check for consistency.
#[tokio::main]
async fn main() -> ExampleResult {
    let client = rest_client()?;
    let (_, orderbook) = market_and_orderbook(&client).await?;
    let orderbook_id = orderbook.orderbook_id.clone();
    let resolution = Resolution::Minute5;

    // 1. Fetch via REST API (same params the app's server function uses)
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64;
    let rest_data = client
        .price_history()
        .get_line_data(
            orderbook_id.as_str(),
            resolution,
            Some(now_ms - (1000 * resolution.seconds() as u64 * 1000)),
            None,
            None,
            Some(1000),
        )
        .await?;

    println!("=== REST API ===");
    println!("  points: {}", rest_data.len());
    if let (Some(first), Some(last)) = (rest_data.first(), rest_data.last()) {
        println!("  first_time: {}", first.time);
        println!("  last_time:  {}", last.time);
        println!(
            "  span_hours: {:.1}",
            (last.time - first.time) as f64 / 3_600_000.0
        );
    }

    // 2. Fetch via WebSocket snapshot
    let mut ws = client.ws_native();
    let mut history = PriceHistoryState::new();

    ws.connect().await?;
    ws.subscribe(SubscribeParams::PriceHistory {
        orderbook_id: orderbook_id.clone(),
        resolution,
        include_ohlcv: false,
    })?;

    {
        let events = ws.events();
        tokio::pin!(events);

        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            let Ok(Some(event)) = timeout_at(deadline, events.next()).await else {
                return Err(other("timed out waiting for WS snapshot").into());
            };
            match event {
                WsEvent::Message(Kind::PriceHistory(PriceHistory::Snapshot(snapshot))) => {
                    history.apply_snapshot(
                        snapshot.orderbook_id.clone(),
                        snapshot.resolution,
                        snapshot.prices.into_iter().map(Into::into).collect(),
                    );
                    break;
                }
                WsEvent::Error(error) => return Err(other(format!("ws error: {error}")).into()),
                _ => continue,
            }
        }
    }
    ws.disconnect().await?;

    let ws_data = history
        .get(&orderbook_id, &resolution)
        .ok_or_else(|| other("no WS data received"))?;

    println!("\n=== WebSocket Snapshot ===");
    println!("  points: {}", ws_data.len());
    if let (Some(first), Some(last)) = (ws_data.first(), ws_data.last()) {
        println!("  first_time: {}", first.time);
        println!("  last_time:  {}", last.time);
        println!(
            "  span_hours: {:.1}",
            (last.time - first.time) as f64 / 3_600_000.0
        );
    }

    // 3. Compare
    println!("\n=== Comparison ===");

    let rest_first = rest_data.first().map(|p| p.time);
    let ws_first = ws_data.first().map(|p| p.time);
    let rest_last = rest_data.last().map(|p| p.time);
    let ws_last = ws_data.last().map(|p| p.time);

    println!(
        "  point count:  REST={} WS={}",
        rest_data.len(),
        ws_data.len()
    );
    println!("  first_time:   REST={:?} WS={:?}", rest_first, ws_first);
    println!("  last_time:    REST={:?} WS={:?}", rest_last, ws_last);

    if rest_data.len() == ws_data.len() && rest_first == ws_first && rest_last == ws_last {
        // Check value consistency for overlapping points
        let mismatches: Vec<_> = rest_data
            .iter()
            .zip(ws_data.iter())
            .filter(|(r, w)| r.time != w.time || r.value != w.value)
            .take(5)
            .collect();

        if mismatches.is_empty() {
            println!("\n  CONSISTENT: REST and WS return identical data");
        } else {
            println!(
                "\n  INCONSISTENT: {} value mismatches (showing first 5):",
                mismatches.len()
            );
            for (rest_point, ws_point) in &mismatches {
                println!(
                    "    time={} REST_val={} WS_val={}",
                    rest_point.time, rest_point.value, ws_point.value
                );
            }
        }
    } else {
        println!("\n  INCONSISTENT: Different ranges or point counts");
        if let (Some(rf), Some(wf)) = (rest_first, ws_first) {
            println!(
                "    first_time diff: {} ms ({:.1} min)",
                (wf - rf).abs(),
                (wf - rf).abs() as f64 / 60_000.0
            );
        }
        if let (Some(rl), Some(wl)) = (rest_last, ws_last) {
            println!(
                "    last_time diff:  {} ms ({:.1} min)",
                (wl - rl).abs(),
                (wl - rl).abs() as f64 / 60_000.0
            );
        }
    }

    Ok(())
}
