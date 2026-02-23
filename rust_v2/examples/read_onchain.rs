//! Read on-chain state: exchange, market, nonce, and PDAs.
//!
//! Uses `LightconePinocchioClient` for direct Solana RPC queries.
//!
//! ```bash
//! cargo run --example read_onchain --features native
//! ```

use lightcone_sdk_v2::program::client::LightconePinocchioClient;
use solana_pubkey::Pubkey;
use std::str::FromStr;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();
    let rpc_url =
        std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());

    let client = LightconePinocchioClient::new(&rpc_url);
    println!("Connected to: {}\n", rpc_url);

    // ── 1. Exchange (singleton) ──────────────────────────────────────────
    let exchange_pda = client.get_exchange_pda();
    println!("Exchange PDA: {}", exchange_pda);

    match client.get_exchange().await {
        Ok(exchange) => {
            println!("  authority:    {}", exchange.authority);
            println!("  operator:     {}", exchange.operator);
            println!("  market_count: {}", exchange.market_count);
            println!("  paused:       {}", exchange.paused);
        }
        Err(e) => println!("  (not found: {})", e),
    }

    // ── 2. Market ────────────────────────────────────────────────────────
    let market_pubkey = std::env::var("MARKET_PUBKEY").ok();
    if let Some(pk) = &market_pubkey {
        let pubkey = Pubkey::from_str(pk)?;
        println!("\nMarket: {}", pk);
        match client.get_market_by_pubkey(&pubkey).await {
            Ok(market) => {
                println!("  market_id:    {}", market.market_id);
                println!("  num_outcomes: {}", market.num_outcomes);
                println!("  status:       {:?}", market.status);
                println!("  oracle:       {}", market.oracle);
            }
            Err(e) => println!("  (not found: {})", e),
        }
    }

    // ── 3. User nonce ────────────────────────────────────────────────────
    let user_pubkey = std::env::var("USER_PUBKEY").ok();
    if let Some(pk) = &user_pubkey {
        let pubkey = Pubkey::from_str(pk)?;
        let nonce_pda = client.get_user_nonce_pda(&pubkey);
        println!("\nUser nonce PDA: {}", nonce_pda);

        let nonce = client.get_user_nonce(&pubkey).await?;
        println!("  current nonce (u64): {}", nonce);

        let current = client.get_current_nonce(&pubkey).await?;
        println!("  current nonce (u32): {}", current);
    }

    // ── 4. PDAs ──────────────────────────────────────────────────────────
    println!("\nPDA derivations:");
    let market_0 = client.get_market_pda(0);
    println!("  market(0):    {}", market_0);

    let dummy = Pubkey::new_unique();
    let position = client.get_position_pda(&dummy, &dummy);
    println!("  position:     {}", position);

    let orderbook = client.get_orderbook_pda(&dummy, &dummy);
    println!("  orderbook:    {}", orderbook);

    let gdt = client.get_global_deposit_token_pda(&dummy);
    println!("  global_dep:   {}", gdt);

    Ok(())
}
