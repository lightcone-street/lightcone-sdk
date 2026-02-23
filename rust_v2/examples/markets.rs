//! Fetch, search, and browse markets.
//!
//! ```bash
//! cargo run --example markets --features native
//! ```

use lightcone_sdk_v2::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = LightconeClient::builder().build()?;

    // ── 1. Featured markets ──────────────────────────────────────────────
    let featured = client.markets().featured().await?;
    println!("Featured markets: {}", featured.len());
    for m in featured.iter().take(3) {
        println!("  {} ({})", m.market_name, m.slug);
    }

    // ── 2. Paginated listing ─────────────────────────────────────────────
    let result = client.markets().get(None, Some(5)).await?;
    println!(
        "\nFirst page: {} markets, {} validation errors",
        result.markets.len(),
        result.validation_errors.len()
    );
    for m in &result.markets {
        println!("  [{}] {} — {}", m.status.as_str(), m.pubkey, m.slug);
    }

    // ── 3. Get by slug ───────────────────────────────────────────────────
    if let Some(first) = result.markets.first() {
        let slug = &first.slug;
        let market = client.markets().get_by_slug(slug).await?;
        println!("\nMarket by slug '{}':", slug);
        println!("  pubkey:   {}", market.pubkey);
        println!("  outcomes: {}", market.outcomes.len());
        for outcome in &market.outcomes {
            println!("    #{}: {}", outcome.index, outcome.name);
        }

        // ── 4. Get by pubkey ─────────────────────────────────────────────
        let market2 = client.markets().get_by_pubkey(market.pubkey.as_str()).await?;
        println!(
            "\nMarket by pubkey: {} (slug: {})",
            market2.pubkey, market2.slug
        );
    }

    // ── 5. Search ────────────────────────────────────────────────────────
    let results = client.markets().search("iran", Some(5)).await?;
    println!("\nSearch 'iran': {} results", results.len());
    for r in &results {
        println!("  {} — {}", r.slug, r.market_status.as_str());
    }

    Ok(())
}
