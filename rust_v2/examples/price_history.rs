//! Fetch OHLCV price history at various resolutions.
//!
//! Set ORDERBOOK_ID in .env.
//!
//! ```bash
//! cargo run --example price_history --features native
//! ```

use lightcone_sdk_v2::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let client = LightconeClient::builder().build()?;

    let orderbook_id = std::env::var("ORDERBOOK_ID").expect("Set ORDERBOOK_ID in .env");

    println!("Price history for: {}\n", orderbook_id);

    // ── 1-hour candles, no time filter ───────────────────────────────────
    let data = client
        .price_history()
        .get(&orderbook_id, Resolution::Hour1, None, None)
        .await?;

    println!("1h candles (raw JSON):");
    if let Some(prices) = data.get("prices").and_then(|v| v.as_array()) {
        println!("  {} data points", prices.len());
        for p in prices.iter().take(3) {
            println!(
                "    ts={} mid={}",
                p.get("timestamp").unwrap_or(&serde_json::Value::Null),
                p.get("midpoint").unwrap_or(&serde_json::Value::Null),
            );
        }
    }

    // ── 1-day candles ────────────────────────────────────────────────────
    let daily = client
        .price_history()
        .get(&orderbook_id, Resolution::Day1, None, None)
        .await?;

    if let Some(prices) = daily.get("prices").and_then(|v| v.as_array()) {
        println!("\n1d candles: {} data points", prices.len());
    }

    Ok(())
}
