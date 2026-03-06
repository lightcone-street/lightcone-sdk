mod common;

use common::{login, market, other, rest_client, wallet, ExampleResult};
use futures_util::StreamExt;
use lightcone::prelude::*;
use solana_signer::Signer;
use tokio::time::{timeout, Duration};

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

        while !(saw_auth && saw_user) {
            let Some(event) = timeout(Duration::from_secs(15), events.next())
                .await
                .map_err(|_| other("timed out waiting for websocket data"))?
            else {
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
    println!("market event received: {saw_market}");
    Ok(())
}
