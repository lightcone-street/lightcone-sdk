mod common;

use common::{market_and_orderbook, other, rest_client, ExampleResult};
use futures_util::StreamExt;
use lightcone::prelude::*;
use tokio::time::{timeout_at, Duration, Instant};

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

    let mut hits = 0;
    {
        let events = ws.events();
        tokio::pin!(events);

        let deadline = Instant::now() + Duration::from_secs(30);
        while hits < 4 {
            let Ok(Some(event)) = timeout_at(deadline, events.next()).await else {
                println!("no more websocket data (timeout or stream ended)");
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
    if hits == 0 {
        return Err(other("received no websocket events — connection may be broken").into());
    }
    println!("buffered trades: {}", trades.len());
    Ok(())
}
