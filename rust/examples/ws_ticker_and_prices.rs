mod common;

use common::{market_and_orderbook, other, rest_client, ExampleResult};
use futures_util::StreamExt;
use lightcone::domain::price_history::wire::PriceHistory;
use lightcone::prelude::*;
use tokio::time::{timeout_at, Duration, Instant};

#[tokio::main]
async fn main() -> ExampleResult {
    let client = rest_client()?;
    let (_, orderbook) = market_and_orderbook(&client).await?;
    let mut ws = client.ws_native();
    let orderbook_id = orderbook.orderbook_id.clone();
    let mut history = PriceHistoryState::new();

    ws.connect().await?;
    ws.subscribe(SubscribeParams::Ticker {
        orderbook_ids: vec![orderbook_id.clone()],
    })?;
    ws.subscribe(SubscribeParams::PriceHistory {
        orderbook_id: orderbook_id.clone(),
        resolution: Resolution::Minute1,
        include_ohlcv: false,
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
                WsEvent::Message(Kind::Ticker(ticker)) => {
                    println!(
                        "ticker: bid={:?} ask={:?} mid={:?}",
                        ticker.best_bid, ticker.best_ask, ticker.mid
                    );
                    hits += 1;
                }
                WsEvent::Message(Kind::PriceHistory(PriceHistory::Snapshot(snapshot))) => {
                    history.apply_snapshot(
                        snapshot.orderbook_id.clone(),
                        snapshot.resolution,
                        snapshot.prices.into_iter().map(Into::into).collect(),
                    );
                    let candles = history
                        .get(&orderbook_id, &Resolution::Minute1)
                        .map_or(0, |points| points.len());
                    println!("price snapshot: {candles} candle(s)");
                    hits += 1;
                }
                WsEvent::Message(Kind::PriceHistory(PriceHistory::Update(update))) => {
                    history.apply_update(
                        update.orderbook_id.clone(),
                        update.resolution,
                        LineData {
                            time: update.t,
                            value: update.m.unwrap_or_default(),
                        },
                    );
                    println!(
                        "latest candle: {:?}",
                        history
                            .get(&orderbook_id, &Resolution::Minute1)
                            .and_then(|points| points.last())
                    );
                    hits += 1;
                }
                WsEvent::Message(Kind::PriceHistory(PriceHistory::Heartbeat(heartbeat))) => {
                    println!("heartbeat: {}", heartbeat.server_time);
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
    Ok(())
}
