mod common;

use common::{market, optional_var, rest_client, ExampleResult};

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

    let query = optional_var("LIGHTCONE_SEARCH_QUERY").unwrap_or_else(|| market.slug.clone());
    let results = client.markets().search(&query, Some(5)).await?;
    println!("search '{query}': {} result(s)", results.len());
    Ok(())
}
