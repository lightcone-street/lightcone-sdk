mod common;

use common::{get_keypair, login, rest_client, ExampleResult};
use lightcone::prelude::*;

#[tokio::main]
async fn main() -> ExampleResult {
    let client = rest_client()?;
    let keypair = get_keypair()?;
    login(&client, &keypair, false).await?;

    let snapshot = client.orders().get_user_orders(Some(50), None).await?;

    println!("orders: {} limit", snapshot.orders.len());
    println!("market balances: {}", snapshot.market_balances.len());
    println!("has more: {}", snapshot.has_more);

    if let Some(order) = snapshot.orders.first() {
        match order {
            UserSnapshotOrder::Limit { common, .. } => {
                println!(
                    "first limit: {} {} @ {}",
                    common.order_hash, common.side, common.price
                );
            }
        }
    }

    if let Some(cursor) = snapshot.next_cursor.as_deref() {
        let next = client
            .orders()
            .get_user_orders(Some(50), Some(cursor))
            .await?;
        println!("next page: {} order(s)", next.orders.len());
    }

    Ok(())
}
