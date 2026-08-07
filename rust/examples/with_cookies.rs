//! Per-call cookie forwarding for SSR / server-function consumers.
//!
//! Demonstrates the `_with_cookies` variants on `Positions`, `Notifications`,
//! `Referrals`, `Orders`, `Metrics`, and `Markets`. These bypass the SDK's process-wide
//! `auth_token` store and forward the supplied raw `Cookie` header for that
//! single call only — so a server can relay whatever auth cookies the browser
//! sent (`privy-token` and/or `lightcone-token`).
//!
//! In a real SSR / server-function context the header would be built from the
//! incoming HTTP request's cookie jar. Here we mimic that by:
//!   1. Logging in once (the SDK captures the `lightcone-token` internally).
//!   2. Reading the token off the client via `auth_token()` and wrapping it in a
//!      `Cookie` header.
//!   3. Clearing the SDK's internal token to prove the `_with_cookies` path
//!      doesn't depend on it.
//!   4. Calling each `_with_cookies` method with the captured header.

mod common;

use common::{get_keypair, login, rest_client, ExampleResult};

#[tokio::main]
async fn main() -> ExampleResult {
    let client = rest_client()?;
    let keypair = get_keypair()?;
    let _ = login(&client, &keypair, false).await?;

    let auth_token = client
        .auth_token()
        .await
        .ok_or("auth_token not set after login — SDK should have captured it")?;
    client.clear_auth_token().await;

    // Native consumers authenticate with a lightcone-token; build the `Cookie`
    // header the same way a browser would send it.
    let cookie_header = format!("lightcone-token={auth_token}");

    let positions = client
        .positions()
        .positions_with_cookies(&cookie_header)
        .await?;
    println!("markets with positions: {}", positions.total_markets);

    let balances = client
        .positions()
        .deposit_token_balances_with_cookies(None, &cookie_header)
        .await?;
    println!("tracked deposit balances: {}", balances.balances.len());

    let notifications = client
        .notifications()
        .fetch_with_cookies(&cookie_header)
        .await?;
    println!("notifications: {}", notifications.len());

    let status = client
        .referrals()
        .get_status_with_cookies(&cookie_header)
        .await?;
    println!("referral codes: {}", status.referral_codes.len());

    let orders = client
        .orders()
        .get_user_orders_with_cookies(Some(50), None, &cookie_header)
        .await?;
    println!("open orders: {}", orders.orders.len());

    let fills = client
        .orders()
        .get_user_order_fills_with_cookies(None, Some(50), None, &cookie_header)
        .await?;
    println!("order fills: {}", fills.orders.len());

    let user_metrics = client.metrics().user_with_cookies(&cookie_header).await?;
    println!(
        "user metrics: volume_usd={} outcomes_traded={}",
        user_metrics.total_volume_usd, user_metrics.total_outcomes_traded
    );

    let mut favorite_market_pubkeys = Vec::new();
    let mut favorite_cursor = None;
    loop {
        let favorite_page = client
            .markets()
            .favorite_markets_with_cookies(Some(1000), favorite_cursor, &cookie_header)
            .await?;
        favorite_market_pubkeys.extend(favorite_page.market_pubkeys);
        if !favorite_page.has_more {
            break;
        }
        favorite_cursor = match favorite_page.next_cursor {
            Some(next_cursor) => Some(next_cursor),
            None => {
                return Err(std::io::Error::other("Favorite page is missing next_cursor").into())
            }
        };
    }
    println!("favorite markets: {}", favorite_market_pubkeys.len());

    if let Some(market) = client.markets().get(None, Some(1)).await?.markets.first() {
        let was_favorited = favorite_market_pubkeys
            .iter()
            .any(|pubkey| pubkey == market.pubkey.as_str());
        if was_favorited {
            client
                .markets()
                .remove_favorite_market_with_cookies(market.pubkey.as_str(), &cookie_header)
                .await?;
            client
                .markets()
                .add_favorite_market_with_cookies(market.pubkey.as_str(), &cookie_header)
                .await?;
        } else {
            client
                .markets()
                .add_favorite_market_with_cookies(market.pubkey.as_str(), &cookie_header)
                .await?;
            client
                .markets()
                .remove_favorite_market_with_cookies(market.pubkey.as_str(), &cookie_header)
                .await?;
        }
        println!("restored favorite state for {}", market.pubkey);
    }

    Ok(())
}
