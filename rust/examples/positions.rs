mod common;

use common::{get_keypair, login, market, rest_client, ExampleResult};

#[tokio::main]
async fn main() -> ExampleResult {
    let client = rest_client()?;
    let keypair = get_keypair()?;
    let session = login(&client, &keypair, false).await?;
    let wallet = session.user.trading_wallet(session.auth_method);
    let market = market(&client).await?;

    let all = client.positions().get(wallet).await?;
    let per_market = client
        .positions()
        .get_for_market(wallet, market.pubkey.as_str())
        .await?;

    println!("wallet: {}", wallet);
    println!("markets with positions: {}", all.total_markets);
    println!(
        "positions in {}: {}",
        market.slug,
        per_market.positions.len()
    );
    Ok(())
}
