mod common;

use common::{market_and_orderbook, rest_client, unix_timestamp, ExampleResult};
use lightcone::prelude::*;

#[tokio::main]
async fn main() -> ExampleResult {
    let client = rest_client()?;
    let (_, orderbook) = market_and_orderbook(&client).await?;
    let to = unix_timestamp()? as u64;
    let from = to.saturating_sub(7 * 24 * 60 * 60);

    let history = client
        .price_history()
        .get(
            orderbook.orderbook_id.as_str(),
            Resolution::Hour1,
            Some(from),
            Some(to),
        )
        .await?;

    println!("{}", serde_json::to_string_pretty(&history)?);
    Ok(())
}
