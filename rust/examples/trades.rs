mod common;

use common::{market_and_orderbook, rest_client, ExampleResult};
use lightcone::domain::trade::Trade;

fn print_trades(page_label: &str, trades: &[Trade]) {
    println!("{page_label}: {} trade(s)", trades.len());
    for trade in trades {
        println!(
            "  {} {} {} {} @ {}",
            trade.trade_id,
            trade.timestamp.to_rfc3339(),
            trade.size,
            trade.side,
            trade.price
        );
    }
}

#[tokio::main]
async fn main() -> ExampleResult {
    let client = rest_client()?;
    let (_, orderbook) = market_and_orderbook(&client).await?;

    let first_page = client
        .trades()
        .get(orderbook.orderbook_id.as_str(), Some(10), None)
        .await?;
    print_trades("page 1", &first_page.trades);
    if let Some(trade) = first_page.trades.first() {
        println!("latest: {} {} @ {}", trade.size, trade.side, trade.price);
    }

    if let Some(cursor) = first_page.next_cursor {
        let next_page = client
            .trades()
            .get(orderbook.orderbook_id.as_str(), Some(10), Some(cursor))
            .await?;
        print_trades("page 2", &next_page.trades);
    }

    Ok(())
}
