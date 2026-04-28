mod common;

use common::{rest_client, ExampleResult};

#[tokio::main]
async fn main() -> ExampleResult {
    let client = rest_client()?;
    let deposit_asset = std::env::args().nth(1);

    let response = client
        .metrics()
        .orderbook_tickers(deposit_asset.as_deref())
        .await?;

    println!("orderbooks with tickers: {}", response.tickers.len());
    for entry in response.tickers.iter().take(10) {
        let mid = entry
            .midpoint
            .as_ref()
            .map(|m| m.to_string())
            .unwrap_or_else(|| "—".to_string());
        println!(
            "  {} (market {}, outcome {:?}) mid={}",
            entry.orderbook_id.as_str(),
            entry.market_pubkey.as_str(),
            entry.outcome_index,
            mid
        );
    }
    Ok(())
}
