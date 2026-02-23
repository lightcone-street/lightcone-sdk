//! Fetch recent trade history for an orderbook.
//!
//! Set ORDERBOOK_ID in .env.
//!
//! ```bash
//! cargo run --example trades --features native
//! ```

use lightcone_sdk_v2::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let client = LightconeClient::builder().build()?;

    let orderbook_id = std::env::var("ORDERBOOK_ID").expect("Set ORDERBOOK_ID in .env");

    println!("Trades for orderbook: {}\n", orderbook_id);

    let trades = client.trades().get(&orderbook_id, Some(10), None).await?;

    println!("Recent trades: {}", trades.len());
    for trade in &trades {
        println!(
            "  {} {:>10} @ {:>8} (id={}, {})",
            trade.side, trade.size, trade.price, trade.trade_id, trade.timestamp
        );
    }

    // Paginate using the trade ID as cursor
    if let Some(last) = trades.last() {
        let cursor: i64 = last.trade_id.parse().unwrap_or(0);
        let older = client
            .trades()
            .get(&orderbook_id, Some(5), Some(cursor))
            .await?;
        println!(
            "\nOlder trades (before id {}): {}",
            cursor,
            older.len()
        );
    }

    Ok(())
}
