//! Exercise the admin log endpoints (list events, get event by id, metrics, metric history).
//!
//! Requires `LIGHTCONE_ADMIN_WALLET_PATH` pointing to an admin-authorized keypair
//! (falls back to `LIGHTCONE_WALLET_PATH` / `~/.config/solana/id.json`).
//!
//! Usage:
//!
//! ```bash
//! API_URL=http://localhost:3001 \
//!   LIGHTCONE_ADMIN_WALLET_PATH=/path/to/admin.json \
//!   cargo run -p lightcone --example admin_logs --features native
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

    // List recent events (limit 5).
    let events_query = AdminLogEventsQuery {
        limit: Some(5),
        ..AdminLogEventsQuery::default()
    };
    let events = client.admin().list_log_events(&events_query).await?;
    println!(
        "log events: {} (next_cursor={:?})",
        events.events.len(),
        events.next_cursor
    );
    for event in &events.events {
        println!(
            "  [{}] {} {} — {}",
            event.severity, event.component, event.operation, event.message
        );
    }

    // Get a specific event by public_id.
    if let Some(first) = events.events.first() {
        let detail = client.admin().get_log_event(&first.public_id).await?;
        println!(
            "event {} detail: category={}",
            detail.public_id, detail.category
        );
    }

    // Metrics breakdowns (defaults to server-picked windows/scopes).
    let metrics = client
        .admin()
        .log_metrics(&AdminLogMetricsQuery::default())
        .await?;
    println!("metrics breakdowns: {}", metrics.breakdowns.len());

    // Metric history for the "service" scope at 1h resolution.
    let history_query = AdminLogMetricHistoryQuery::new("service");
    let history = client.admin().log_metric_history(&history_query).await?;
    println!(
        "metric history scope='{}' resolution={}: {} points",
        history.scope,
        history.resolution,
        history.points.len()
    );

    client.admin().admin_logout().await?;
    Ok(())
}
