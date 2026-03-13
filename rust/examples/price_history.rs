mod common;

use common::{market_and_orderbook, rest_client, unix_timestamp_ms, ExampleResult};
use lightcone::{
    domain::price_history::{DepositPriceHistoryQuery, OrderbookPriceHistoryQuery},
    prelude::*,
};

#[tokio::main]
async fn main() -> ExampleResult {
    let client = rest_client()?;
    let (market, orderbook) = market_and_orderbook(&client).await?;
    let to = unix_timestamp_ms()? as u64;
    let from = to.saturating_sub(7 * 24 * 60 * 60 * 1000);

    let Some(deposit_asset) = market.deposit_assets.first() else {
        return Err(common::other("selected market has no deposit assets").into());
    };

    let orderbook_history = client
        .price_history()
        .get_with_query(
            orderbook.orderbook_id.as_str(),
            OrderbookPriceHistoryQuery {
                resolution: Resolution::Hour1,
                from: Some(from),
                to: Some(to),
                limit: Some(10),
                include_ohlcv: true,
                ..OrderbookPriceHistoryQuery::default()
            },
        )
        .await?;
    let deposit_history = client
        .price_history()
        .get_deposit_asset(
            deposit_asset.deposit_asset.as_str(),
            DepositPriceHistoryQuery {
                resolution: Resolution::Hour1,
                from: Some(from),
                to: Some(to),
                limit: Some(10),
                ..DepositPriceHistoryQuery::default()
            },
        )
        .await?;

    println!("orderbook:");
    println!("{}", serde_json::to_string_pretty(&orderbook_history)?);
    println!("deposit asset:");
    println!("{}", serde_json::to_string_pretty(&deposit_history)?);
    Ok(())
}
