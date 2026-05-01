//! Claim testnet SOL and whitelisted deposit tokens for a wallet.
//!
//! Only works against environments with the faucet enabled (local / staging).
//!
//! Usage:
//!
//! ```bash
//! API_URL=http://localhost:3001 cargo run -p lightcone --example faucet_claim --features native
//! ```

mod common;

use common::{get_keypair, rest_client, ExampleResult};
use lightcone::prelude::PubkeyStr;
use solana_signer::Signer;

#[tokio::main]
async fn main() -> ExampleResult {
    let client = rest_client()?;
    let keypair = get_keypair()?;
    let wallet: PubkeyStr = keypair.pubkey().into();

    let result = client.claim(&wallet).await?;
    println!("claim tx: {}", result.signature);
    println!("sol: {}", result.sol);
    for token in &result.tokens {
        println!("  - {}: {}", token.symbol, token.amount);
    }
    Ok(())
}
