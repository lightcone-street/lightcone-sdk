//! WebSocket: best bid/ask ticker + price history with local candle state.
//!
//! Demonstrates:
//! - Ticker stream for real-time best bid/ask/mid updates
//! - `PriceHistoryState` for maintaining local candle data
//! - Handling the `PriceHistory` tagged enum (Snapshot, Update, Heartbeat)
//! - Graceful shutdown via Ctrl+C
//!
//! Set ORDERBOOK_ID in .env.
//!
//! ```bash
//! cargo run --example ws_ticker_and_prices --features native
//! ```

use futures_util::StreamExt;
use lightcone_sdk_v2::domain::price_history::wire::PriceHistory;
use lightcone_sdk_v2::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let client = LightconeClient::builder().build()?;

    let ob_id = OrderBookId::new(
        std::env::var("ORDERBOOK_ID").expect("Set ORDERBOOK_ID in .env"),
    );

    println!("Orderbook: {}", ob_id);
    println!("Press Ctrl+C to stop\n");

    let mut price_state = PriceHistoryState::new();
    let resolution = Resolution::Hour1;

    let mut ws = client.ws_native();
    ws.connect().await?;

    ws.send(MessageOut::subscribe_ticker(vec![ob_id.clone()]))?;
    ws.send(MessageOut::subscribe_price_history(ob_id.clone(), resolution))?;

    tokio::select! {
        _ = async {
            let mut stream = ws.events();
            while let Some(event) = stream.next().await {
                match event {
                    WsEvent::Connected => println!("[connected]"),

                    WsEvent::Message(Kind::Ticker(ticker)) => {
                        println!(
                            "[ticker] bid={} ask={} mid={}",
                            ticker.best_bid.map(|d| d.to_string()).unwrap_or_else(|| "—".into()),
                            ticker.best_ask.map(|d| d.to_string()).unwrap_or_else(|| "—".into()),
                            ticker.mid.map(|d| d.to_string()).unwrap_or_else(|| "—".into()),
                        );
                    }

                    WsEvent::Message(Kind::PriceHistory(ph)) => {
                        match ph {
                            PriceHistory::Snapshot(snap) => {
                                let prices: Vec<LineData> =
                                    snap.prices.into_iter().map(LineData::from).collect();
                                let count = prices.len();
                                price_state.apply_snapshot(
                                    ob_id.clone(),
                                    snap.resolution,
                                    prices,
                                );
                                println!(
                                    "[price snapshot] {} candles loaded ({})",
                                    count, snap.resolution
                                );
                                if let Some(data) = price_state.get(&ob_id, &snap.resolution) {
                                    if let Some(last) = data.last() {
                                        println!("  latest: ts={} value={}", last.time, last.value);
                                    }
                                }
                            }
                            PriceHistory::Update(upd) => {
                                let mid = upd.m.clone().unwrap_or_default();
                                let point = LineData {
                                    time: upd.t,
                                    value: mid.clone(),
                                };
                                price_state.apply_update(
                                    ob_id.clone(),
                                    upd.resolution,
                                    point,
                                );
                                let candle_count = price_state
                                    .get(&ob_id, &upd.resolution)
                                    .map(|d| d.len())
                                    .unwrap_or(0);
                                println!(
                                    "[price update] ts={} mid={} ({}) — {} candles total",
                                    upd.t, mid, upd.resolution, candle_count
                                );
                            }
                            PriceHistory::Heartbeat(hb) => {
                                println!(
                                    "[heartbeat] server_time={} last_processed={:?}",
                                    hb.server_time, hb.last_processed
                                );
                            }
                        }
                    }

                    WsEvent::Message(Kind::Pong(_)) => {}

                    WsEvent::Disconnected { code, reason } => {
                        println!("[disconnected] code={:?} reason={}", code, reason);
                    }

                    WsEvent::MaxReconnectReached => {
                        eprintln!("[fatal] max reconnect attempts reached");
                        break;
                    }

                    WsEvent::Error(e) => eprintln!("[error] {}", e),
                    WsEvent::Message(Kind::Error(e)) => eprintln!("[server error] {}", e),
                    _ => {}
                }
            }
        } => {}
        _ = tokio::signal::ctrl_c() => {
            println!("\nShutting down...");
        }
    }

    ws.disconnect().await?;

    if let Some(data) = price_state.get(&ob_id, &resolution) {
        println!("\nFinal candle count ({}): {}", resolution, data.len());
    }

    Ok(())
}
