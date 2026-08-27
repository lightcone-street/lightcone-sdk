mod common;

use common::{get_keypair, login, market, other, rest_client, ExampleResult};
use futures_util::StreamExt;
use lightcone::prelude::*;
use tokio::time::{timeout_at, Duration, Instant};

#[tokio::main]
async fn main() -> ExampleResult {
    let client = rest_client()?;
    let keypair = get_keypair()?;
    let market = market(&client).await?;
    let session = login(&client, &keypair, false).await?;
    let wallet = PubkeyStr::from(session.user.trading_wallet(session.auth_method).to_owned());
    let market_pubkey = market.pubkey.clone();
    let mut balances = WalletDepositBalancesState::default();

    let mut ws = client.ws_native();
    ws.connect().await?;
    let observed = {
        let events = ws.events();
        tokio::pin!(events);

        ws.subscribe(SubscribeParams::User {
            wallet_address: wallet.clone(),
        })?;
        ws.subscribe(SubscribeParams::Market {
            market_pubkey: market_pubkey.clone(),
        })?;
        ws.subscribe(SubscribeParams::WalletDepositBalances {
            wallet_address: wallet.clone(),
        })?;

        let deadline = Instant::now() + Duration::from_secs(30);
        let result = loop {
            let Ok(Some(event)) = timeout_at(deadline, events.next()).await else {
                break Err(other(
                    "timed out waiting for a complete wallet balance snapshot",
                ));
            };

            match event {
                WsEvent::Message(Kind::WalletDepositBalances(update)) => {
                    let complete = matches!(update, WalletDepositBalancesEvent::Snapshot { .. });
                    if balances.apply_event(&update) == WalletDepositBalancesApplyResult::Applied
                        && complete
                    {
                        break Ok(());
                    }
                }
                WsEvent::Message(Kind::Error(error)) => {
                    break Err(other(format!("WebSocket error: {}", error.error)));
                }
                WsEvent::Error(error) => {
                    break Err(other(format!("WebSocket transport error: {error}")));
                }
                WsEvent::MaxReconnectReached => {
                    break Err(other("WebSocket reconnect attempts exhausted"));
                }
                _ => {}
            }
        };
        result
    };

    ws.unsubscribe(UnsubscribeParams::User {
        wallet_address: wallet.clone(),
    })?;
    ws.unsubscribe(UnsubscribeParams::Market { market_pubkey })?;
    ws.unsubscribe(UnsubscribeParams::WalletDepositBalances {
        wallet_address: wallet.clone(),
    })?;
    ws.disconnect().await?;
    observed?;

    println!(
        "wallet={} slot={} count={}",
        wallet,
        balances
            .context_slot
            .ok_or_else(|| other("complete snapshot did not establish a slot"))?,
        balances.balances.len()
    );
    Ok(())
}
