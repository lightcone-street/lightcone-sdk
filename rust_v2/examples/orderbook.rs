//! Fetch orderbook depth and decimal precision for a market.
//!
//! Set MARKET_PUBKEY in .env.
//!
//! ```bash
//! cargo run --example orderbook --features native
//! ```

use lightcone_sdk_v2::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let client = LightconeClient::builder().build()?;

    let market_pubkey = std::env::var("MARKET_PUBKEY").expect("Set MARKET_PUBKEY in .env");
    let market = client.markets().get_by_pubkey(&market_pubkey).await?;

    println!("Market: {} ({})\n", market.pubkey, market.slug);

    for pair in &market.orderbook_pairs {
        let ob_id = pair.orderbook_id.as_str();
        println!("━━ Orderbook: {} (outcome #{})", ob_id, pair.outcome_index);
        println!("  Base:  {} ({})", pair.base.symbol(), pair.base.pubkey());
        println!("  Quote: {} ({})", pair.quote.symbol(), pair.quote.pubkey());

        // ── 1. Orderbook depth ───────────────────────────────────────
        let book = client.orderbooks().get(ob_id, Some(5)).await?;
        println!("  Best bid: {:?}", book.best_bid);
        println!("  Best ask: {:?}", book.best_ask);
        println!("  Bids: {} levels", book.bids.len());
        for level in &book.bids {
            println!("    {} @ {}", level.size, level.price);
        }
        println!("  Asks: {} levels", book.asks.len());
        for level in &book.asks {
            println!("    {} @ {}", level.size, level.price);
        }

        // ── 2. Decimals ──────────────────────────────────────────────
        let decimals = client.orderbooks().decimals(ob_id).await?;
        println!("  Decimals: base={}, quote={}, price={}",
            decimals.base_decimals, decimals.quote_decimals, decimals.price_decimals);
        println!();
    }

    Ok(())
}
