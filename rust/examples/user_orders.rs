mod common;

use common::{get_keypair, login, rest_client, ExampleResult};

#[tokio::main]
async fn main() -> ExampleResult {
    let client = rest_client()?;
    let keypair = get_keypair()?;
    login(&client, &keypair, false).await?;

    let snapshot = client.orders().get_user_orders(Some(50), None).await?;

    println!(
        "orders: {} (revision {}, more: {})",
        snapshot.orders.len(),
        snapshot.committed_revision,
        snapshot.has_more
    );
    println!("funding accounts: {}", snapshot.funding_accounts.len());

    if let Some(order) = snapshot.orders.first() {
        let open = order.state.as_ref().map_or_else(
            || "unknown".to_string(),
            |state| state.open_base.to_string(),
        );
        println!(
            "first order: {} {} @ {} open={} tif={:?}",
            order.order_hash, order.side, order.price, open, order.tif
        );
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
