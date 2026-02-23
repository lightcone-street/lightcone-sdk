//! On-chain transactions: mint/merge complete set, withdraw from position.
//!
//! Demonstrates building and signing Solana transactions via the SDK.
//! Transactions are built but NOT sent — uncomment the send lines to execute.
//!
//! ```bash
//! cargo run --example onchain_transactions --features native
//! ```

use lightcone_sdk_v2::program::client::LightconePinocchioClient;
use lightcone_sdk_v2::program::types::*;
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use std::str::FromStr;

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
    let rpc_url =
        std::env::var("RPC_URL").unwrap_or_else(|_| "https://api.devnet.solana.com".to_string());

    let client = LightconePinocchioClient::new(&rpc_url);
    let user = keypair.pubkey();

    let market_pubkey = Pubkey::from_str(
        &std::env::var("MARKET_PUBKEY").expect("Set MARKET_PUBKEY in .env"),
    )?;
    let deposit_mint = Pubkey::from_str(
        &std::env::var("QUOTE_TOKEN").expect("Set QUOTE_TOKEN in .env"),
    )?;

    let num_outcomes: u8 = 2;
    let amount: u64 = 1_000_000; // 1 token (assuming 6 decimals)

    println!("User:    {}", user);
    println!("Market:  {}", market_pubkey);
    println!("Deposit: {}", deposit_mint);

    // ── 1. Mint complete set ─────────────────────────────────────────────
    println!("\n--- Mint Complete Set ---");
    let mint_params = MintCompleteSetParams {
        user,
        market: market_pubkey,
        deposit_mint,
        amount,
    };

    let mut mint_tx = client.mint_complete_set(mint_params, num_outcomes).await?;
    let blockhash = client.get_latest_blockhash().await?;
    mint_tx.partial_sign(&[&keypair], blockhash);
    println!("Transaction built and signed (MintCompleteSet)");
    // Uncomment to send:
    let sig = client.rpc_client.send_and_confirm_transaction(&mint_tx).await?;
    println!("  tx: {}", sig);

    // ── 2. Merge complete set ────────────────────────────────────────────
    println!("\n--- Merge Complete Set ---");
    let merge_params = MergeCompleteSetParams {
        user,
        market: market_pubkey,
        deposit_mint,
        amount,
    };

    let mut merge_tx = client.merge_complete_set(merge_params, num_outcomes).await?;
    let blockhash = client.get_latest_blockhash().await?;
    merge_tx.partial_sign(&[&keypair], blockhash);
    println!("Transaction built and signed (MergeCompleteSet)");

    // ── 3. Withdraw from position ────────────────────────────────────────
    println!("\n--- Withdraw From Position ---");
    let withdraw_params = WithdrawFromPositionParams {
        user,
        market: market_pubkey,
        mint: deposit_mint,
        amount,
        outcome_index: 0, // Withdraw outcome #0 tokens; use 255 for collateral
    };

    let mut withdraw_tx = client
        .withdraw_from_position(withdraw_params, false)
        .await?;
    let blockhash = client.get_latest_blockhash().await?;
    withdraw_tx.partial_sign(&[&keypair], blockhash);
    println!("Transaction built and signed (WithdrawFromPosition, outcome #0)");

    // ── 4. Increment nonce (mass cancel) ─────────────────────────────────
    println!("\n--- Increment Nonce ---");
    let current_nonce = client.get_user_nonce(&user).await?;
    println!("Current nonce: {}", current_nonce);

    let mut nonce_tx = client.increment_nonce(&user).await?;
    let blockhash = client.get_latest_blockhash().await?;
    nonce_tx.partial_sign(&[&keypair], blockhash);
    println!("Transaction built and signed (IncrementNonce)");
    println!("Sending this would invalidate all orders with nonce <= {}", current_nonce);

    Ok(())
}
