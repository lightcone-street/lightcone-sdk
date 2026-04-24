mod common;

use common::{get_keypair, login, rest_client, ExampleResult};

#[tokio::main]
async fn main() -> ExampleResult {
    let client = rest_client()?;
    let keypair = get_keypair()?;
    let user = login(&client, &keypair, false).await?;

    let balances = client.positions().deposit_token_balances().await?;

    println!("wallet: {}", user.wallet_address);
    println!("tracked balances: {}", balances.len());

    let mut entries: Vec<_> = balances.values().collect();
    entries.sort_by(|a, b| a.symbol.cmp(&b.symbol));
    for balance in entries {
        println!(
            "  {:>8}  {:<42}  idle={}",
            balance.symbol, balance.mint, balance.idle
        );
    }

    client.auth().logout().await?;
    Ok(())
}
