mod common;

use common::{get_keypair, login, rest_client, ExampleResult};
use lightcone::prelude::*;

#[tokio::main]
async fn main() -> ExampleResult {
    let client = rest_client()?;
    let keypair = get_keypair()?;
    login(&client, &keypair, false).await?;

    let snapshot = client.orders().get_user_orders(Some(50), None).await?;

    let (limit_orders, trigger_orders) =
        snapshot
            .orders
            .iter()
            .fold((0usize, 0usize), |(limits, triggers), order| match order {
                UserSnapshotOrder::Limit { .. } => (limits + 1, triggers),
                UserSnapshotOrder::Trigger { .. } => (limits, triggers + 1),
            });

    println!(
        "orders: {} limit / {} trigger",
        limit_orders, trigger_orders
    );
    println!("balances: {} market", snapshot.balances.len());
    println!("has more: {}", snapshot.has_more);

    if let Some(order) = snapshot.orders.first() {
        match order {
            UserSnapshotOrder::Limit { common, .. } => {
                println!(
                    "first limit: {} {} @ {}",
                    common.order_hash, common.side, common.price
                );
            }
            UserSnapshotOrder::Trigger {
                common,
                trigger_order_id,
                trigger_price,
                ..
            } => {
                println!(
                    "first trigger: {} {} @ {} (trigger {})",
                    trigger_order_id, common.side, common.price, trigger_price
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
