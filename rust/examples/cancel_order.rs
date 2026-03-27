mod common;

use common::{get_keypair, login, rest_client, unix_timestamp, ExampleResult};
use lightcone::prelude::*;
use solana_signer::Signer;

#[tokio::main]
async fn main() -> ExampleResult {
    let client = rest_client()?;
    let keypair = get_keypair()?;
    login(&client, &keypair, false).await?;

    let snapshot = client
        .orders()
        .get_user_orders(&keypair.pubkey().to_string(), Some(50), None)
        .await?;

    let Some((order_hash, orderbook_id)) = snapshot.orders.iter().find_map(|order| match order {
        UserSnapshotOrder::Limit { common, .. } => {
            Some((common.order_hash.clone(), common.orderbook_id.clone()))
        }
        UserSnapshotOrder::Trigger { .. } => None,
    }) else {
        println!("No open limit orders to cancel.");
        return Ok(());
    };

    let cancel = CancelBody::signed(order_hash, keypair.pubkey().into(), &keypair);
    let salt = generate_cancel_all_salt();
    let cancel_all = CancelAllBody::signed(
        keypair.pubkey().into(),
        orderbook_id,
        unix_timestamp()?,
        salt,
        &keypair,
    );

    let cancelled = client.orders().cancel(&cancel).await?;
    let cleared = client.orders().cancel_all(&cancel_all).await?;

    println!(
        "cancelled: {} remaining={}",
        cancelled.order_hash, cancelled.remaining
    );
    println!(
        "cancel-all removed {} order(s) in {}",
        cleared.count, cleared.orderbook_id
    );
    Ok(())
}
