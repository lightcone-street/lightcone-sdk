//! Fetch open orders for an authenticated user.
//!
//! ```bash
//! cargo run --example user_orders --features native
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

    let wallet = keypair.pubkey().to_string();
    println!("User orders for: {}\n", wallet);

    let orders = client.orders().get_user_orders(&GetUserOrdersRequest {
        wallet_address: wallet.clone(),
        limit: Some(100),
        cursor: None,
    }).await?;
    println!(
        "Response:\n{}",
        serde_json::to_string_pretty(&orders)?
    );

    // Test: fetch orders for a wallet we don't own — should be rejected (403)
    let other_wallet = "11111111111111111111111111111111";
    println!("\nFetching orders for unauthorized wallet: {}", other_wallet);
    match client.orders().get_user_orders(&GetUserOrdersRequest {
        wallet_address: other_wallet.to_string(),
        limit: Some(100),
        cursor: None,
    }).await {
        Ok(resp) => println!("Unexpected success:\n{}", serde_json::to_string_pretty(&resp)?),
        Err(e) => println!("Expected error: {}", e),
    }

    Ok(())
}
