//! Fetch user positions — all markets and per-market.
//!
//! Set USER_PUBKEY in .env or pass as argument.
//!
//! ```bash
//! cargo run --example positions --features native
//! ```

use lightcone_sdk_v2::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let client = LightconeClient::builder().build()?;

    let user_pubkey =
        std::env::var("USER_PUBKEY").unwrap_or_else(|_| "YourWalletPubkeyHere".to_string());

    // ── 1. All positions ─────────────────────────────────────────────────
    println!("Fetching positions for: {}\n", user_pubkey);
    let response = client.positions().get(&user_pubkey).await?;

    println!("Owner: {}", response.owner);
    println!("Total markets: {}", response.total_markets);
    println!("Positions: {}\n", response.positions.len());

    for pos in &response.positions {
        println!("  Market: {}", pos.market_pubkey);
        println!("  Position PDA: {}", pos.position_pubkey);
        for outcome in &pos.outcomes {
            println!(
                "    outcome #{}: balance={} (idle={}, on_book={}) token={}",
                outcome.outcome_index,
                outcome.balance,
                outcome.balance_idle,
                outcome.balance_on_book,
                outcome.conditional_token,
            );
        }
    }

    // ── 2. Positions for a specific market ───────────────────────────────
    if let Some(first_pos) = response.positions.first() {
        let market_pubkey = &first_pos.market_pubkey;
        println!("\n━━ Per-market positions: {}", market_pubkey);

        let market_response = client
            .positions()
            .get_for_market(&user_pubkey, market_pubkey)
            .await?;

        for pos in &market_response.positions {
            for outcome in &pos.outcomes {
                println!(
                    "  outcome #{}: balance={} (idle={}, on_book={})",
                    outcome.outcome_index,
                    outcome.balance,
                    outcome.balance_idle,
                    outcome.balance_on_book,
                );
            }
        }
    }

    Ok(())
}
