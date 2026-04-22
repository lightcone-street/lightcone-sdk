//! Exercise the admin referral config + codes endpoints.
//!
//! Requires `LIGHTCONE_ADMIN_WALLET_PATH` pointing to an admin-authorized keypair
//! (falls back to `LIGHTCONE_WALLET_PATH` / `~/.config/solana/id.json`).
//!
//! Usage:
//!
//! ```bash
//! API_URL=http://localhost:3001 \
//!   LIGHTCONE_ADMIN_WALLET_PATH=/path/to/admin.json \
//!   cargo run -p lightcone --example admin_referral_codes --features native
//! ```

mod common;

use common::{rest_client, ExampleResult};
use lightcone::prelude::*;
use solana_keypair::{read_keypair_file, Keypair};
use solana_signer::Signer;
use std::env;

fn load_admin_keypair() -> ExampleResult<Keypair> {
    let raw = env::var("LIGHTCONE_ADMIN_WALLET_PATH")
        .or_else(|_| env::var("LIGHTCONE_WALLET_PATH"))
        .unwrap_or_else(|_| "~/.config/solana/id.json".to_string());
    let path = if let Some(rest) = raw.strip_prefix("~/") {
        let home = env::var("HOME").map_err(|_| "HOME not set")?;
        std::path::PathBuf::from(home).join(rest)
    } else {
        raw.into()
    };
    read_keypair_file(path)
}

async fn admin_login(client: &LightconeClient, keypair: &Keypair) -> ExampleResult {
    let nonce = client.admin().get_admin_nonce().await?;
    let signature = keypair.sign_message(nonce.message.as_bytes());
    client
        .admin()
        .admin_login(
            &nonce.message,
            &signature.to_string(),
            &keypair.pubkey().to_bytes(),
        )
        .await?;
    Ok(())
}

#[tokio::main]
async fn main() -> ExampleResult {
    let client = rest_client()?;
    let keypair = load_admin_keypair()?;

    admin_login(&client, &keypair).await?;
    println!("admin logged in as {}", keypair.pubkey());

    // Referral config
    let config = client.admin().get_referral_config().await?;
    println!(
        "referral config: default_code_count={} (updated_at={})",
        config.default_code_count, config.updated_at
    );

    // List the first 10 referral codes
    let listing = client
        .admin()
        .list_referral_codes(&ListCodesRequest {
            limit: 10,
            offset: 0,
            ..ListCodesRequest::default()
        })
        .await?;
    println!("codes ({} total):", listing.count);
    for entry in &listing.codes {
        println!(
            "  - {} owner={} uses={}/{} vanity={}",
            entry.code, entry.owner_user_id, entry.use_count, entry.max_uses, entry.is_vanity
        );
    }

    client.admin().admin_logout().await?;
    Ok(())
}
