mod common;

use common::{market_and_orderbook, other, rest_client, ExampleResult};
use futures_util::StreamExt;
use lightcone::prelude::*;
use tokio::time::{timeout, Duration};

#[tokio::main]
async fn main() -> ExampleResult {
    let client = rest_client()?;
    let (_, orderbook) = market_and_orderbook(&client).await?;
    let mut ws = client.ws_native();
    let orderbook_id = orderbook.orderbook_id.clone();
    let mut book = OrderbookSnapshot::new(orderbook_id.clone());
    let mut trades = TradeHistory::new(orderbook_id.clone(), 20);

    ws.connect().await?;
    ws.subscribe(SubscribeParams::Books {
        orderbook_ids: vec![orderbook_id.clone()],
    })?;
    ws.subscribe(SubscribeParams::Trades {
        orderbook_ids: vec![orderbook_id.clone()],
    })?;

    {
        let events = ws.events();
        tokio::pin!(events);
        let mut hits = 0;

        while hits < 4 {
            let Some(event) = timeout(Duration::from_secs(15), events.next())
                .await
                .map_err(|_| other("timed out waiting for websocket data"))?
            else {
                break;
            };

            match event {
                WsEvent::Message(Kind::BookUpdate(update)) => {
                    book.apply(&update);
                    println!(
                        "book: seq={} bid={:?} ask={:?}",
                        book.seq,
                        book.best_bid(),
                        book.best_ask()
                    );
                    hits += 1;
                }
                WsEvent::Message(Kind::Trade(trade)) => {
                    println!("trade: {} {} @ {}", trade.size, trade.side, trade.price);
                    trades.push(trade.into());
                    hits += 1;
                }
                WsEvent::Error(err) => eprintln!("ws error: {err}"),
                _ => {}
            }
        }
    }

    ws.disconnect().await?;
    println!("buffered trades: {}", trades.len());
    Ok(())
}
