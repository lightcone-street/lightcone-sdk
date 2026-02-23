//! WebSocket: live orderbook depth + trade feed with local state management.
//!
//! Demonstrates:
//! - `OrderbookSnapshot` for maintaining local orderbook state (snapshots + deltas)
//! - `TradeHistory` as a rolling trade buffer
//! - Graceful shutdown via Ctrl+C
//! - Auto-reconnect handling (the client reconnects automatically)
//!
//! Set ORDERBOOK_ID in .env.
//!
//! ```bash
//! cargo run --example ws_book_and_trades --features native
//! ```

use futures_util::StreamExt;
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

    // Local state containers
    let mut book = OrderbookSnapshot::new(ob_id.clone());
    let mut trades = TradeHistory::new(ob_id.clone(), 100);

    let mut ws = client.ws_native();
    ws.connect().await?;

    ws.send(MessageOut::subscribe_books(vec![ob_id.clone()]))?;
    ws.send(MessageOut::subscribe_trades(vec![ob_id]))?;

    // Stream events until Ctrl+C
    tokio::select! {
        _ = async {
            let mut stream = ws.events();
            while let Some(event) = stream.next().await {
                match event {
                    WsEvent::Connected => println!("[connected]"),

                    WsEvent::Message(Kind::BookUpdate(update)) => {
                        let label = if update.is_snapshot { "snapshot" } else { "delta" };
                        book.apply(&update);
                        println!(
                            "[book {label}] seq={} bids={} asks={} | best_bid={} best_ask={} spread={} mid={}",
                            book.seq,
                            book.bids().len(),
                            book.asks().len(),
                            fmt_opt(book.best_bid()),
                            fmt_opt(book.best_ask()),
                            fmt_opt(book.spread()),
                            fmt_opt(book.mid_price()),
                        );
                    }

                    WsEvent::Message(Kind::Trade(ws_trade)) => {
                        let trade: Trade = ws_trade.into();
                        println!(
                            "[trade] {} {} @ {} ({})",
                            trade.side, trade.size, trade.price, trade.trade_id
                        );
                        trades.push(trade);
                        println!("  trade history: {} trades buffered", trades.len());
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

    println!("\nFinal orderbook: {} bids, {} asks", book.bids().len(), book.asks().len());
    println!("Total trades captured: {}", trades.len());
    if let Some(latest) = trades.latest() {
        println!("Last trade: {} {} @ {}", latest.side, latest.size, latest.price);
    }

    Ok(())
}

fn fmt_opt(v: Option<rust_decimal::Decimal>) -> String {
    v.map(|d| d.to_string()).unwrap_or_else(|| "—".into())
}
