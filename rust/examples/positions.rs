mod common;

use common::{login, market, rest_client, wallet, ExampleResult};

#[tokio::main]
async fn main() -> ExampleResult {
    let client = rest_client()?;
    let keypair = wallet()?;
    let user = login(&client, &keypair, false).await?;
    let market = market(&client).await?;

    let all = client.positions().get(&user.wallet_address).await?;
    let per_market = client
        .positions()
        .get_for_market(&user.wallet_address, market.pubkey.as_str())
        .await?;

    println!("wallet: {}", user.wallet_address);
    println!("markets with positions: {}", all.total_markets);
    println!(
        "positions in {}: {}",
        market.slug,
        per_market.positions.len()
    );
    Ok(())
}
