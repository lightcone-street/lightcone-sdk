mod common;

use common::ExampleResult;
use lightcone::prelude::*;
use std::time::Instant;

// Unreachable address — simulates a dead primary RPC.
const DEAD_PRIMARY: &str = "http://localhost:1";

const BACKUP_RPC: &str =
    "https://devnet.helius-rpc.com/?api-key=55558885-9601-4d35-a25a-55af783fce2b";

#[tokio::main]
async fn main() -> ExampleResult {
    // ════════════════════════════════════════════════════════════════════
    // Part A: JSON-RPC path (raw_post — works on all platforms)
    // ════════════════════════════════════════════════════════════════════

    let client = LightconeClient::builder()
        .rpc_url(DEAD_PRIMARY)
        .backup_rpc_url(BACKUP_RPC)
        .build()?;

    println!("═══ JSON-RPC path (raw_post) ═══\n");
    println!("primary : {DEAD_PRIMARY}");
    println!("backup  : {BACKUP_RPC}");
    println!("active  : {:?}\n", client.active_rpc().await);

    // 1. First call — triggers failover.
    // Primary refuses connections → fast retry (100 ms) → still dead →
    // flip to backup → success.
    println!("--- call 1: get_latest_blockhash (triggers failover) ---");
    let start = Instant::now();
    let blockhash = client.get_latest_blockhash().await?;
    println!("blockhash : {blockhash}");
    println!("elapsed   : {:.2?}", start.elapsed());
    println!("active    : {:?}\n", client.active_rpc().await);

    // 2. Second call — goes straight to backup, no retry delay.
    println!("--- call 2: get_latest_blockhash (already on backup) ---");
    let start = Instant::now();
    let blockhash = client.get_latest_blockhash().await?;
    println!("blockhash : {blockhash}");
    println!("elapsed   : {:.2?}", start.elapsed());
    println!("active    : {:?}\n", client.active_rpc().await);

    // ════════════════════════════════════════════════════════════════════
    // Part B: Native SolanaRpcClient path (solana-rpc feature)
    // ════════════════════════════════════════════════════════════════════
    //
    // Fresh client so failover state starts at Primary — this verifies
    // that the native SolanaRpcClient path triggers its own failover.

    let native_client = LightconeClient::builder()
        .rpc_url(DEAD_PRIMARY)
        .backup_rpc_url(BACKUP_RPC)
        .build()?;

    println!("═══ Native SolanaRpcClient path ═══\n");
    println!("active  : {:?}\n", native_client.active_rpc().await);

    // 3. Native failover — primary dead → retry → flip → backup succeeds.
    println!("--- call 3: rpc().get_latest_blockhash() (triggers native failover) ---");
    let start = Instant::now();
    let blockhash = native_client.rpc().get_latest_blockhash().await?;
    println!("blockhash : {blockhash}");
    println!("elapsed   : {:.2?}", start.elapsed());
    println!("active    : {:?}\n", native_client.active_rpc().await);

    // 4. Subsequent native call — goes straight to backup.
    println!("--- call 4: rpc().get_latest_blockhash() (already on backup) ---");
    let start = Instant::now();
    let blockhash = native_client.rpc().get_latest_blockhash().await?;
    println!("blockhash : {blockhash}");
    println!("elapsed   : {:.2?}", start.elapsed());
    println!("active    : {:?}\n", native_client.active_rpc().await);

    // ════════════════════════════════════════════════════════════════════
    // Part C: Both endpoints dead — clean error, no panic
    // ════════════════════════════════════════════════════════════════════

    println!("═══ Both endpoints dead ═══\n");

    let dead_client = LightconeClient::builder()
        .rpc_url("http://localhost:1")
        .backup_rpc_url("http://localhost:2")
        .build()?;

    println!("--- call 5: JSON-RPC path, both dead ---");
    let start = Instant::now();
    match dead_client.get_latest_blockhash().await {
        Ok(_) => println!("unexpected success"),
        Err(error) => println!("error (expected): {error}"),
    }
    println!("elapsed   : {:.2?}", start.elapsed());
    println!("active    : {:?}\n", dead_client.active_rpc().await);

    let dead_native = LightconeClient::builder()
        .rpc_url("http://localhost:1")
        .backup_rpc_url("http://localhost:2")
        .build()?;

    println!("--- call 6: native path, both dead ---");
    let start = Instant::now();
    match dead_native.rpc().get_latest_blockhash().await {
        Ok(_) => println!("unexpected success"),
        Err(error) => println!("error (expected): {error}"),
    }
    println!("elapsed   : {:.2?}", start.elapsed());
    println!("active    : {:?}", dead_native.active_rpc().await);

    Ok(())
}
