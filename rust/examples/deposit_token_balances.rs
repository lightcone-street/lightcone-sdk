mod common;

use common::{get_keypair, login, rest_client, ExampleResult};

#[tokio::main]
async fn main() -> ExampleResult {
    let client = rest_client()?;
    let keypair = get_keypair()?;
    let session = login(&client, &keypair, false).await?;
    let wallet = session.user.trading_wallet(session.auth_method);

    let snapshot = client.positions().deposit_token_balances(None).await?;

    println!("wallet: {}", wallet);
    println!("context slot: {}", snapshot.context_slot);
    println!("tracked balances: {}", snapshot.balances.len());

    let mut entries: Vec<_> = snapshot.balances.values().collect();
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
