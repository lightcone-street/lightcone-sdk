//! Build, sign, and submit an order using OrderBuilder with auto-scaling.
//!
//! Requires a valid Solana keypair and authentication.
//!
//! ```bash
//! cargo run --example submit_order --features native
//! ```

use lightcone_sdk_v2::auth::native::sign_login_message;
use lightcone_sdk_v2::prelude::*;
use lightcone_sdk_v2::program::builder::OrderBuilder;
use lightcone_sdk_v2::shared::OrderbookDecimals;
use solana_signer::Signer;
use chrono::Utc;
use std::time::{SystemTime, UNIX_EPOCH};

fn load_keypair() -> solana_keypair::Keypair {
    dotenvy::dotenv().ok();
    let path = std::env::var("KEYPAIR_PATH")
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_default();
            format!("{}/.config/solana/id.json", home)
        });
    let bytes: Vec<u8> =
        serde_json::from_str(&std::fs::read_to_string(&path).expect("keypair file not found"))
            .expect("invalid keypair JSON");
    solana_keypair::Keypair::try_from(bytes.as_slice()).expect("invalid keypair bytes")
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let keypair = load_keypair();
    let client = LightconeClient::builder().build()?;

    // ── 1. Authenticate ──────────────────────────────────────────────────
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let signed = sign_login_message(&keypair, timestamp);
    client
        .auth()
        .login_with_message(
            &signed.message,
            &signed.signature_bs58,
            &signed.pubkey_bytes,
            None,
        )
        .await?;
    println!("Authenticated as: {}", keypair.pubkey());

    // ── 2. Discover market and orderbook ─────────────────────────────────
    let market_slug = std::env::var("MARKET_SLUG").expect("Set MARKET_SLUG in .env");
    let market = client.markets().get_by_slug(&market_slug).await?;
    let pair = market.orderbook_pairs.first().expect("market has no orderbooks");
    let orderbook_id = pair.orderbook_id.as_str();
    let base_mint = pair.base.pubkey().to_pubkey().expect("invalid base mint");
    let quote_mint = pair.quote.pubkey().to_pubkey().expect("invalid quote mint");

    println!("Market:    {} ({})", market.name, market.pubkey);
    println!("Orderbook: {}", orderbook_id);

    // ── 3. Fetch decimals for scaling ────────────────────────────────────
    let dec = client.orderbooks().decimals(orderbook_id).await?;
    let decimals = OrderbookDecimals {
        orderbook_id: orderbook_id.to_string(),
        base_decimals: dec.base_decimals,
        quote_decimals: dec.quote_decimals,
        price_decimals: dec.price_decimals,
    };
    println!(
        "Decimals:  base={}, quote={}, price={}",
        dec.base_decimals, dec.quote_decimals, dec.price_decimals
    );

    // ── 4. Build and sign order with auto-scaling ────────────────────────
    let request = OrderBuilder::new()
        .nonce(0)
        .maker(keypair.pubkey())
        .market(market.pubkey.to_pubkey().expect("invalid market pubkey"))
        .base_mint(base_mint)
        .quote_mint(quote_mint)
        .bid()
        .price("0.50")
        .size("10")
        .apply_scaling(&decimals)?
        .to_submit_request(&keypair, orderbook_id)?;

    println!("\nOrder built:");
    println!("  side:       BID");
    println!("  price:      0.50");
    println!("  size:       10");
    println!("  amount_in:  {}", request.amount_in);
    println!("  amount_out: {}", request.amount_out);
    println!("  nonce:      {}", request.nonce);

    // ── 5. Submit ────────────────────────────────────────────────────────
    let response = client.orders().submit(&request).await?;
    println!("\nOrder submitted:");
    println!("  order_hash: {}", response.order_hash);
    println!("  remaining:  {}", response.remaining);
    println!("  filled:     {}", response.filled);
    println!("  fills:      {}", response.fills.len());
    for fill in &response.fills {
        println!(
            "    {} @ {} (counterparty: {})",
            fill.fill_amount, fill.price, fill.counterparty
        );
    }

    // ── 6. Example: ASK order with expiration ─────────────────────────────
    // Build an ask (sell base for quote) that expires in 60 seconds.
    // Not submitted — for demonstration only.
    let ask_request = OrderBuilder::new()
        .nonce(0)
        .maker(keypair.pubkey())
        .market(market.pubkey.to_pubkey().expect("invalid market pubkey"))
        .base_mint(base_mint)
        .quote_mint(quote_mint)
        .ask()
        .price("0.55")
        .size("10")
        .expiration(Utc::now().timestamp() + 60) // expires in 60s
        .apply_scaling(&decimals)?
        .to_submit_request(&keypair, orderbook_id)?;

    println!("\nASK order with expiration (not submitted):");
    println!("  side:       ASK");
    println!("  price:      0.55");
    println!("  size:       10");
    println!("  expiration: {}", ask_request.expiration);
    println!("  amount_in:  {}", ask_request.amount_in);
    println!("  amount_out: {}", ask_request.amount_out);

    // ── 7. Example: raw lamport amounts (skip scaling) ────────────────────
    // When you already have raw u64 amounts, skip price/size/apply_scaling.
    let raw_request = OrderBuilder::new()
        .nonce(0)
        .maker(keypair.pubkey())
        .market(market.pubkey.to_pubkey().expect("invalid market pubkey"))
        .base_mint(base_mint)
        .quote_mint(quote_mint)
        .bid()
        .amount_in(65_000_000)   // quote lamports (what maker gives)
        .amount_out(100_000_000) // base lamports (what maker receives)
        .to_submit_request(&keypair, orderbook_id)?;

    println!("\nRaw-amount order (not submitted):");
    println!("  amount_in:  {}", raw_request.amount_in);
    println!("  amount_out: {}", raw_request.amount_out);

    Ok(())
}
