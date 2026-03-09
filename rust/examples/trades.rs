mod common;

use common::{market_and_orderbook, rest_client, ExampleResult};

#[tokio::main]
async fn main() -> ExampleResult {
    let client = rest_client()?;
    let (_, orderbook) = market_and_orderbook(&client).await?;

    let first_page = client
        .trades()
        .get(orderbook.orderbook_id.as_str(), Some(10), None)
        .await?;
    println!("page 1: {} trade(s)", first_page.trades.len());
    if let Some(trade) = first_page.trades.first() {
        println!("latest: {} {} @ {}", trade.size, trade.side, trade.price);
    }

    if let Some(cursor) = first_page.next_cursor {
        let next_page = client
            .trades()
            .get(orderbook.orderbook_id.as_str(), Some(10), Some(cursor))
            .await?;
        println!("page 2: {} trade(s)", next_page.trades.len());
    }

    Ok(())
}
