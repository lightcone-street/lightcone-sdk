mod common;

use common::{market_and_orderbook, other, rest_client, ExampleResult};
use lightcone::prelude::BookAggregation;

#[tokio::main]
async fn main() -> ExampleResult {
    let client = rest_client()?;
    let (market, orderbook) = market_and_orderbook(&client).await?;

    // Depth is capped server-side at 20 levels per side.
    let depth = client
        .orderbooks()
        .get(
            orderbook.orderbook_id.as_str(),
            Some(10),
            BookAggregation::FULL,
        )
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
    if let Some(depth_decimals) = depth.decimals {
        println!(
            "depth decimals: price={}, size={}",
            depth_decimals.price, depth_decimals.size
        );
    }

    // Hyperliquid-style aggregation: 5 significant figures, 1/2/5 mantissa
    // sub-steps. Bids bucket by flooring, asks by ceiling.
    let grouped_aggregation =
        BookAggregation::validate(Some(5), Some(2)).map_err(|message| other(message))?;
    let grouped = client
        .orderbooks()
        .get(orderbook.orderbook_id.as_str(), None, grouped_aggregation)
        .await?;
    println!(
        "grouped ({}): {} bids / {} asks",
        grouped_aggregation.key_suffix(),
        grouped.bids.len(),
        grouped.asks.len()
    );
    Ok(())
}
