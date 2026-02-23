//! WebSocket: authenticated user stream + market lifecycle events.
//!
//! Demonstrates:
//! - Authenticating via `sign_login_message` before connecting WS
//! - User stream: order updates, balance changes, nonce updates
//! - Market stream: settled, created, opened, paused, orderbook_created
//! - Auth status confirmation via `Kind::Auth`
//! - Graceful shutdown via Ctrl+C
//!
//! Set KEYPAIR_PATH and MARKET_PUBKEY in .env.
//!
//! ```bash
//! cargo run --example ws_user_and_market --features native
//! ```

use futures_util::StreamExt;
use lightcone_sdk_v2::auth::native::sign_login_message;
use lightcone_sdk_v2::domain::market::wire::MarketEvent;
use lightcone_sdk_v2::prelude::*;
use solana_signer::Signer;
use std::time::{SystemTime, UNIX_EPOCH};

fn load_keypair() -> solana_keypair::Keypair {
    dotenvy::dotenv().ok();
    let path = std::env::var("KEYPAIR_PATH").unwrap_or_else(|_| {
        let home = std::env::var("HOME").unwrap_or_default();
        format!("{}/.config/solana/id.json", home)
    });
    let bytes: Vec<u8> =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("keypair file not found"))
            .expect("invalid keypair JSON");
    solana_keypair::Keypair::try_from(bytes.as_slice()).expect("invalid keypair bytes")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let keypair = load_keypair();
    let wallet = keypair.pubkey().to_string();
    let client = LightconeClient::builder().build()?;

    let market_pubkey = std::env::var("MARKET_PUBKEY").expect("Set MARKET_PUBKEY in .env");

    // ── 1. Authenticate (user stream requires valid session) ─────────
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let signed = sign_login_message(&keypair, timestamp);
    let user = client
        .auth()
        .login_with_message(
            &signed.message,
            &signed.signature_bs58,
            &signed.pubkey_bytes,
            None,
        )
        .await?;
    println!("Authenticated: {} ({})", user.wallet_address, user.id);
    println!("Market: {}", market_pubkey);
    println!("Press Ctrl+C to stop\n");

    // ── 2. Connect and subscribe ─────────────────────────────────────
    let mut ws = client.ws_native();
    ws.connect().await?;

    ws.send(MessageOut::subscribe_user(PubkeyStr::new(&wallet)))?;
    ws.send(MessageOut::subscribe_market(PubkeyStr::new(&market_pubkey)))?;

    // ── 3. Stream events ─────────────────────────────────────────────
    tokio::select! {
        _ = async {
            let mut stream = ws.events();
            while let Some(event) = stream.next().await {
                match event {
                    WsEvent::Connected => println!("[connected]"),

                    WsEvent::Message(Kind::Auth(auth)) => {
                        println!("[auth] {:?}", auth);
                    }

                    WsEvent::Message(Kind::User(update)) => {
                        println!("[user] {:?}", update);
                    }

                    WsEvent::Message(Kind::Market(event)) => {
                        match &event {
                            MarketEvent::Settled { market_pubkey } => {
                                println!("[market] settled: {}", market_pubkey);
                            }
                            MarketEvent::Created { market_pubkey } => {
                                println!("[market] created: {}", market_pubkey);
                            }
                            MarketEvent::Opened { market_pubkey } => {
                                println!("[market] opened: {}", market_pubkey);
                            }
                            MarketEvent::Paused { market_pubkey } => {
                                println!("[market] paused: {}", market_pubkey);
                            }
                            MarketEvent::OrderbookCreated {
                                market_pubkey,
                                orderbook_id,
                            } => {
                                println!(
                                    "[market] orderbook created: {} ({})",
                                    orderbook_id, market_pubkey
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
    client.auth().logout().await?;
    println!("Logged out.");

    Ok(())
}
