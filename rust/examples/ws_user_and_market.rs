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
    let observed: ExampleResult = {
        let events = ws.events();
        tokio::pin!(events);

        let subscriptions: ExampleResult = (|| {
            ws.subscribe(SubscribeParams::User {
                wallet_address: wallet.clone(),
            })?;
            ws.subscribe(SubscribeParams::Market {
                market_pubkey: market_pubkey.clone(),
            })?;
            ws.subscribe(SubscribeParams::WalletDepositBalances {
                wallet_address: wallet.clone(),
            })?;
            Ok(())
        })();

        match subscriptions {
            Err(error) => Err(error),
            Ok(()) => {
                let deadline = Instant::now() + Duration::from_secs(30);
                loop {
                    let Ok(Some(event)) = timeout_at(deadline, events.next()).await else {
                        break Err(other(
                            "timed out waiting for a complete wallet balance snapshot",
                        )
                        .into());
                    };

                    match event {
                        WsEvent::Message(Kind::WalletDepositBalances(update)) => {
                            let complete =
                                matches!(update, WalletDepositBalancesEvent::Snapshot { .. });
                            if balances.apply_event(&update)
                                == WalletDepositBalancesApplyResult::Applied
                                && complete
                            {
                                break Ok(());
                            }
                        }
                        WsEvent::Message(Kind::Error(error)) => {
                            break Err(other(format!("WebSocket error: {}", error.error)).into());
                        }
                        WsEvent::Error(error) => {
                            break Err(other(format!("WebSocket transport error: {error}")).into());
                        }
                        WsEvent::MaxReconnectReached => {
                            break Err(other("WebSocket reconnect attempts exhausted").into());
                        }
                        _ => {}
                    }
                }
            }
        }
    };

    let mut unsubscribe_error = None;
    for params in [
        UnsubscribeParams::User {
            wallet_address: wallet.clone(),
        },
        UnsubscribeParams::Market { market_pubkey },
        UnsubscribeParams::WalletDepositBalances {
            wallet_address: wallet.clone(),
        },
    ] {
        if let Err(error) = ws.unsubscribe(params) {
            unsubscribe_error.get_or_insert(error);
        }
    }
    let disconnected = ws.disconnect().await;
    observed?;
    if let Some(error) = unsubscribe_error {
        return Err(error.into());
    }
    disconnected?;

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
