mod common;

use common::{other, rest_client, ExampleResult};
use futures_util::StreamExt;
use lightcone::domain::price_history::wire::DepositAssetPriceEvent;
use lightcone::prelude::*;
use lightcone::shared::PubkeyStr;
use tokio::time::{timeout_at, Duration, Instant};

#[tokio::main]
async fn main() -> ExampleResult {
    let client = rest_client()?;

    // REST: snapshot of current prices for every active mint in `global_deposit_tokens`.
    let snapshot = client
        .price_history()
        .get_deposit_asset_prices_snapshot()
        .await?;
    println!(
        "REST /api/deposit-asset-prices-snapshot ({} entries):",
        snapshot.prices.len()
    );
    for (mint, price) in snapshot.prices.iter().take(10) {
        println!("  {} -> {}", mint, price);
    }

    // Pick the first asset and subscribe via WS for live updates.
    let Some((mint, _)) = snapshot.prices.iter().next() else {
        return Err(other("snapshot has no entries — backend has no priced assets").into());
    };
    let deposit_asset = PubkeyStr::from(mint.clone());

    let mut ws = client.ws_native();
    let mut state = DepositPriceState::new();

    ws.connect().await?;
    ws.subscribe(SubscribeParams::DepositAssetPrice {
        deposit_asset: deposit_asset.clone(),
    })?;

    let mut hits = 0;
    {
        let events = ws.events();
        tokio::pin!(events);
        let deadline = Instant::now() + Duration::from_secs(30);

        while hits < 2 {
            let Ok(Some(event)) = timeout_at(deadline, events.next()).await else {
                println!("no more websocket data (timeout or stream ended)");
                break;
            };

            match event {
                WsEvent::Message(Kind::DepositAssetPrice(payload)) => match payload {
                    DepositAssetPriceEvent::Snapshot(snap) => {
                        state.apply_deposit_asset_price_snapshot(
                            PubkeyStr::from(snap.deposit_asset.clone()),
                            snap.price.clone(),
                        );
                        println!("WS snapshot: {} -> {}", snap.deposit_asset, snap.price);
                        hits += 1;
                    }
                    DepositAssetPriceEvent::Price(tick) => {
                        state.apply_price_tick(
                            PubkeyStr::from(tick.deposit_asset.clone()),
                            tick.price.clone(),
                            tick.event_time,
                        );
                        println!(
                            "WS tick: {} -> {} @ {}",
                            tick.deposit_asset, tick.price, tick.event_time
                        );
                        hits += 1;
                    }
                },
                WsEvent::Error(err) => eprintln!("ws error: {err}"),
                _ => {}
            }
        }
    }

    ws.unsubscribe(UnsubscribeParams::DepositAssetPrice {
        deposit_asset: deposit_asset.clone(),
    })?;
    ws.disconnect().await?;
    if hits == 0 {
        return Err(other("received no websocket events — connection may be broken").into());
    }
    Ok(())
}
