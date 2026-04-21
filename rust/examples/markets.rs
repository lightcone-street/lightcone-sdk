mod common;

use common::{market, rest_client, ExampleResult};
use lightcone::prelude::Token;

#[tokio::main]
async fn main() -> ExampleResult {
    let client = rest_client()?;

    let featured = client.markets().featured().await?;
    println!("featured markets: {}", featured.len());
    if let Some(first) = featured.first() {
        println!("featured: {} ({})", first.market_name, first.slug);
    }

    let page = client.markets().get(None, Some(5)).await?;
    println!(
        "paginated listing: {} markets, {} validation errors",
        page.markets.len(),
        page.validation_errors.len()
    );

    let market = market(&client).await?;
    println!("by slug: {} -> {}", market.slug, market.pubkey);
    println!(
        "by pubkey: {}",
        client
            .markets()
            .get_by_pubkey(market.pubkey.as_str())
            .await?
            .name
    );

    let query = market
        .name
        .split_whitespace()
        .find(|word| word.len() > 3)
        .unwrap_or("market")
        .to_string();
    let results = client.markets().search(&query, Some(5)).await?;
    println!("search '{query}': {} result(s)", results.len());
    for result in &results {
        println!("  - {}", result.slug);
    }

    println!("deposit asset pairs for {}:", market.slug);
    for pair in &market.deposit_asset_pairs {
        println!(
            "  - {} ({}/{})",
            pair.id, pair.base.symbol, pair.quote.symbol
        );
    }

    let global_assets = client.markets().global_deposit_assets().await?;
    println!(
        "global deposit assets: {} ({} validation errors)",
        global_assets.assets.len(),
        global_assets.validation_errors.len()
    );
    for asset in &global_assets.assets {
        println!("  - {} ({})", asset.symbol(), asset.pubkey());
    }
    Ok(())
}
