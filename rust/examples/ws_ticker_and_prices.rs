mod common;

use common::{market_and_orderbook, other, rest_client, ExampleResult};
use futures_util::StreamExt;
use lightcone::domain::price_history::wire::PriceHistory;
use lightcone::prelude::*;
use tokio::time::{timeout, Duration};

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
                }
                WsEvent::Error(err) => eprintln!("ws error: {err}"),
                _ => {}
            }
        }
    }

    ws.disconnect().await?;
    Ok(())
}
