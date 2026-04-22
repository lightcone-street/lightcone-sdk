//! List the deposit assets registered for a specific market.
//!
//! Usage:
//!
//! ```bash
//! API_URL=http://localhost:3001 cargo run -p lightcone --example market_deposit_assets --features native
//! ```

mod common;

use common::{market, rest_client, ExampleResult};

#[tokio::main]
async fn main() -> ExampleResult {
    let client = rest_client()?;
    let market = market(&client).await?;

    let response = client.markets().deposit_assets(&market.pubkey).await?;
    println!(
        "market {} ({}): {} deposit assets",
        market.slug, response.market_pubkey, response.total
    );
    for asset in &response.deposit_assets {
        println!(
            "  - {} ({}) — {} conditional mints",
            asset.symbol.as_deref().unwrap_or("?"),
            asset.deposit_asset,
            asset.conditional_mints.len()
        );
    }
    Ok(())
}
