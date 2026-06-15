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

    // One connection can hold multiple aggregation views of the same book.
    // Key the local state by the frame's aggregation: full precision for
    // pricing, a grouped view (5 sig figs, mantissa 2) for display.
    let grouped_aggregation =
        BookAggregation::validate(Some(5), Some(2)).map_err(|message| other(message))?;
    let mut full_book = OrderbookState::new(orderbook_id.clone());
    let mut grouped_book = OrderbookState::new(orderbook_id.clone());
    let mut trades = TradeHistory::new(orderbook_id.clone(), 20);

    ws.connect().await?;
    ws.send(MessageOut::subscribe_books(
        vec![orderbook_id.clone()],
        BookAggregation::FULL,
    ))?;
    ws.send(MessageOut::subscribe_books(
        vec![orderbook_id.clone()],
        grouped_aggregation,
    ))?;
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
                    // Untagged frames are the full-precision view; frames from
                    // the grouped subscription carry n_sig_figs/mantissa.
                    let aggregation = update.aggregation();
                    if update.resync {
                        // Refresh exactly the affected view: re-subscribe with
                        // the SAME aggregation. The fresh snapshot arrives with
                        // seq 0 and replaces the book (last-write-wins).
                        ws.send(MessageOut::unsubscribe_books(
                            vec![update.id.clone()],
                            aggregation,
                        ))?;
                        ws.send(MessageOut::subscribe_books(
                            vec![update.id.clone()],
                            aggregation,
                        ))?;
                        continue;
                    }
                    let book = if aggregation == grouped_aggregation {
                        &mut grouped_book
                    } else {
                        &mut full_book
                    };
                    book.apply(&update);
                    println!(
                        "book[{}]: seq={} bid={:?} ask={:?}",
                        aggregation.key_suffix(),
                        book.seq,
                        book.best_bid(),
                        book.best_ask()
                    );
                    hits += 1;
                }
                WsEvent::Message(Kind::Trade(trade)) => {
                    println!(
                        "trade: {} {} @ {} seq={}",
                        trade.size, trade.side, trade.price, trade.sequence
                    );
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
