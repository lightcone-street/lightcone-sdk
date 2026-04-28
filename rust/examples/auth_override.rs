//! Per-call cookie override for server-side cookie forwarding.
//!
//! Demonstrates the `_with_auth_override` variants on `Positions`,
//! `Notifications`, `Referrals`, and `Orders`. These bypass the SDK's
//! process-wide `auth_token` store and pass the supplied token as a
//! `Cookie: auth_token=…` header for that single call only.
//!
//! In a real SSR / server-function context the token would be extracted from
//! the incoming HTTP request's cookie jar. Here we mimic that by:
//!   1. Logging in once (the SDK captures the token internally).
//!   2. Reading the token off the client via `auth_token()`.
//!   3. Clearing the SDK's internal token to prove the override path doesn't
//!      depend on it.
//!   4. Calling each `_with_auth_override` method with the captured token.

mod common;

use common::{get_keypair, login, rest_client, ExampleResult};

#[tokio::main]
async fn main() -> ExampleResult {
    let client = rest_client()?;
    let keypair = get_keypair()?;
    let user = login(&client, &keypair, false).await?;

    let auth_token = client
        .auth_token()
        .await
        .ok_or("auth_token not set after login — SDK should have captured it")?;
    client.clear_auth_token().await;

    let positions = client
        .positions()
        .positions_with_auth_override(&auth_token)
        .await?;
    println!("markets with positions: {}", positions.total_markets);

    let balances = client
        .positions()
        .deposit_token_balances_with_auth_override(&auth_token)
        .await?;
    println!("tracked deposit balances: {}", balances.len());

    let notifications = client
        .notifications()
        .fetch_with_auth_override(&auth_token)
        .await?;
    println!("notifications: {}", notifications.len());

    let status = client
        .referrals()
        .get_status_with_auth_override(&auth_token)
        .await?;
    println!("referral codes: {}", status.referral_codes.len());

    let orders = client
        .orders()
        .get_user_orders_with_auth_override(&user.wallet_address, Some(50), None, &auth_token)
        .await?;
    println!("open orders: {}", orders.orders.len());

    let fills = client
        .orders()
        .get_user_order_fills_with_auth_override(
            &user.wallet_address,
            None,
            Some(50),
            None,
            &auth_token,
        )
        .await?;
    println!("order fills: {}", fills.orders.len());

    Ok(())
}
