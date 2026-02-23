//! Full authentication lifecycle: sign -> login -> check_session -> logout.
//!
//! Requires a valid Solana keypair.
//!
//! ```bash
//! cargo run --example login --features native
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

    // ── 1. Sign a login message ──────────────────────────────────────────
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let signed = sign_login_message(&keypair, timestamp);
    println!("Signed login message (timestamp: {})", timestamp);

    // ── 2. Login ─────────────────────────────────────────────────────────
    let user = client
        .auth()
        .login_with_message(
            &signed.message,
            &signed.signature_bs58,
            &signed.pubkey_bytes,
            None,
        )
        .await?;

    println!("\nLogged in:");
    println!("  user_id:  {}", user.id);
    println!("  wallet:   {}", user.wallet_address);
    println!("  linked:   {} ({})", user.linked_account.address, user.linked_account.account_type);

    // ── 3. Check session ─────────────────────────────────────────────────
    let is_auth = client.auth().is_authenticated().await;
    println!("\nAuthenticated (cached): {}", is_auth);

    let session_user = client.auth().check_session().await?;
    println!("Session valid — user_id: {}", session_user.id);

    // ── 4. Logout ────────────────────────────────────────────────────────
    client.auth().logout().await?;
    let is_auth_after = client.auth().is_authenticated().await;
    println!("\nAfter logout — authenticated: {}", is_auth_after);

    Ok(())
}
