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
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use std::str::FromStr;
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
    let market_pubkey = std::env::var("MARKET_PUBKEY").expect("Set MARKET_PUBKEY in .env");
    let orderbook_id = std::env::var("ORDERBOOK_ID").expect("Set ORDERBOOK_ID in .env");
    let base_token = std::env::var("BASE_TOKEN").expect("Set BASE_TOKEN in .env");
    let quote_token = std::env::var("QUOTE_TOKEN").expect("Set QUOTE_TOKEN in .env");

    // ── 3. Fetch decimals for scaling ────────────────────────────────────
    let dec = client.orderbooks().decimals(&orderbook_id).await?;
    let decimals = OrderbookDecimals {
        orderbook_id: orderbook_id.clone(),
        base_decimals: dec.base_decimals,
        quote_decimals: dec.quote_decimals,
        price_decimals: dec.price_decimals,
    };
    println!(
        "Decimals: base={}, quote={}, price={}",
        dec.base_decimals, dec.quote_decimals, dec.price_decimals
    );

    // ── 4. Build and sign order with auto-scaling ────────────────────────
    let request = OrderBuilder::new()
        .nonce(0)
        .maker(keypair.pubkey())
        .market(Pubkey::from_str(&market_pubkey)?)
        .base_mint(Pubkey::from_str(&base_token)?)
        .quote_mint(Pubkey::from_str(&quote_token)?)
        .bid()
        .price("0.50")
        .size("10")
        .apply_scaling(&decimals)?
        .to_submit_request(&keypair, &orderbook_id);

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

    Ok(())
}
