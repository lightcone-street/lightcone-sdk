mod common;

use common::{market_and_orderbook, rest_client, ExampleResult};

#[tokio::main]
async fn main() -> ExampleResult {
    let client = rest_client()?;
    let (market, orderbook) = market_and_orderbook(&client).await?;

    let depth = client
        .orderbooks()
        .get(orderbook.orderbook_id.as_str(), Some(10))
        .await?;
    let decimals = orderbook.decimals();

    println!("market: {}", market.slug);
    println!("orderbook: {}", orderbook.orderbook_id);
    println!(
        "best bid: {:?}, best ask: {:?}",
        depth.best_bid, depth.best_ask
    );
    println!(
        "levels: {} bids / {} asks",
        depth.bids.len(),
        depth.asks.len()
    );
    println!(
        "decimals: price={}, base={}, quote={}",
        decimals.price_decimals, decimals.base_decimals, decimals.quote_decimals
    );
    Ok(())
}
