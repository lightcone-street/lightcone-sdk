//! Cancel a single order and cancel all orders in an orderbook.
//!
//! Requires a valid Solana keypair and authentication.
//!
//! ```bash
//! cargo run --example cancel_order --features native
//! ```

use lightcone_sdk_v2::auth::native::sign_login_message;
use lightcone_sdk_v2::prelude::*;
use solana_signer::Signer;
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

    // Authenticate
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
    println!("Authenticated as: {}\n", keypair.pubkey());

    // ── 1. Cancel a single order ─────────────────────────────────────────
    let order_hash = std::env::var("ORDER_HASH")
        .unwrap_or_else(|_| "abc123...your_order_hash_hex".to_string());

    let cancel_body = CancelBody::signed(
        order_hash.clone(),
        keypair.pubkey().to_string(),
        &keypair,
    );

    println!("Cancelling order: {}", order_hash);
    match client.orders().cancel(&cancel_body).await {
        Ok(result) => {
            println!("  Cancelled! order_hash: {}", result.order_hash);
            println!("  remaining: {}", result.remaining);
        }
        Err(e) => println!("  Cancel failed: {}", e),
    }

    // ── 2. Cancel all orders in an orderbook ─────────────────────────────
    let orderbook_id = std::env::var("ORDERBOOK_ID")
        .unwrap_or_else(|_| "your_orderbook_id".to_string());

    let cancel_timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs() as i64;

    let cancel_all_body = CancelAllBody::signed(
        keypair.pubkey().to_string(),
        orderbook_id.clone(),
        cancel_timestamp,
        &keypair,
    );

    println!("\nCancelling all orders in orderbook: {}", orderbook_id);
    match client.orders().cancel_all(&cancel_all_body).await {
        Ok(result) => {
            println!("  Cancelled {} orders", result.count);
            println!("  message: {}", result.message);
            for hash in &result.cancelled_order_hashes {
                println!("    {}", hash);
            }
        }
        Err(e) => println!("  Cancel all failed: {}", e),
    }

    Ok(())
}
