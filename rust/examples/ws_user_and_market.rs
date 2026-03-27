mod common;

use common::{login, market, other, rest_client, wallet, ExampleResult};
use futures_util::StreamExt;
use lightcone::prelude::*;
use solana_signer::Signer;
use tokio::time::{timeout_at, Duration, Instant};

#[tokio::main]
async fn main() -> ExampleResult {
    let client = rest_client()?;
    let keypair = wallet()?;
    let market = market(&client).await?;
    login(&client, &keypair, false).await?;

    let mut ws = client.ws_native();
    ws.connect().await?;
    ws.subscribe(SubscribeParams::User {
        wallet_address: keypair.pubkey().into(),
    })?;
    ws.subscribe(SubscribeParams::Market {
        market_pubkey: market.pubkey.clone(),
    })?;

    let mut saw_auth = false;
    let mut saw_user = false;
    let mut saw_market = false;

    {
        let events = ws.events();
        tokio::pin!(events);

        let deadline = Instant::now() + Duration::from_secs(30);
        while !(saw_auth && saw_user) {
            let Ok(Some(event)) = timeout_at(deadline, events.next()).await else {
                println!("no more websocket data (timeout or stream ended)");
                break;
            };

            match event {
                WsEvent::Message(Kind::Auth(update)) => {
                    println!("auth: {:?}", update);
                    saw_auth = true;
                }
                WsEvent::Message(Kind::User(update)) => {
                    println!("user: {:?}", update);
                    saw_user = true;
                }
                WsEvent::Message(Kind::Market(event)) => {
                    println!("market: {:?}", event);
                    saw_market = true;
                }
                WsEvent::Error(err) => eprintln!("ws error: {err}"),
                _ => {}
            }
        }
    }

    ws.disconnect().await?;
    if !saw_auth && !saw_user {
        return Err(other("received no websocket events — connection may be broken").into());
    }
    println!("market event received: {saw_market}");
    Ok(())
}
