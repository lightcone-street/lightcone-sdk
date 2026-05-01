//! Exercise every metrics endpoint end-to-end.
//!
//! Useful as a wire-type smoke test: if any field deserialization is wrong,
//! this example will fail.
//!
//! Usage:
//!
//! ```bash
//! API_URL=http://localhost:3001 cargo run -p lightcone --example metrics_all --features native
//! ```

mod common;

use common::{get_keypair, login, rest_client, ExampleResult};
use lightcone::prelude::*;
use solana_signer::Signer;

#[tokio::main]
async fn main() -> ExampleResult {
    let client = rest_client()?;
    let keypair = get_keypair()?;
    login(&client, &keypair, false).await?;

    // ── Platform ─────────────────────────────────────────────────────────
    let platform = client.metrics().platform().await?;
    println!(
        "platform: 24h=${}, 7d=${}, active_markets={}, active_orderbooks={}",
        platform.volume_24h_usd,
        platform.volume_7d_usd,
        platform.active_markets,
        platform.active_orderbooks
    );
    println!(
        "  deposit token volumes: {}",
        platform.deposit_token_volumes.len()
    );

    // ── Markets list ─────────────────────────────────────────────────────
    let markets = client
        .metrics()
        .markets(&MarketsMetricsQuery::default())
        .await?;
    println!(
        "markets: {} entries (total={})",
        markets.markets.len(),
        markets.total
    );
    for entry in markets.markets.iter().take(3) {
        println!(
            "  - {} — 24h=${} (share={}%)",
            entry.market_name.as_deref().unwrap_or("?"),
            entry.volume_24h_usd,
            entry.platform_volume_share_24h_pct
        );
    }

    // ── Market detail (if we have at least one) ─────────────────────────
    if let Some(top_market) = markets.markets.first() {
        let detail = client
            .metrics()
            .market(&top_market.market_pubkey, &MarketMetricsQuery::default())
            .await?;
        println!(
            "market detail {}: outcomes={}, orderbooks={}",
            detail.market_pubkey,
            detail.outcome_volumes.len(),
            detail.orderbook_volumes.len()
        );

        if let Some(first_ob) = detail.orderbook_volumes.first() {
            let ob_metrics = client
                .metrics()
                .orderbook(&first_ob.orderbook_id, &OrderbookMetricsQuery::default())
                .await?;
            println!(
                "orderbook {}: 24h_usd=${} 24h_base={}",
                ob_metrics.orderbook_id, ob_metrics.volume_24h_usd, ob_metrics.volume_24h_base
            );
        }
    }

    // ── Categories ───────────────────────────────────────────────────────
    let categories = client.metrics().categories().await?;
    println!("categories: {}", categories.categories.len());
    if let Some(first_cat) = categories.categories.first() {
        let detail = client
            .metrics()
            .category(&first_cat.category, &CategoryMetricsQuery::default())
            .await?;
        println!(
            "category '{}': 24h=${}, traders_24h={}",
            detail.category, detail.volume_24h_usd, detail.unique_traders_24h
        );
    }

    // ── Deposit tokens ──────────────────────────────────────────────────
    let deposit_tokens = client.metrics().deposit_tokens().await?;
    println!("deposit tokens: {}", deposit_tokens.deposit_tokens.len());

    // ── Leaderboard ──────────────────────────────────────────────────────
    let board = client.metrics().leaderboard(Some(5)).await?;
    println!(
        "leaderboard ({}): {} entries",
        board.period,
        board.entries.len()
    );
    for entry in &board.entries {
        println!(
            "  #{} {} — 24h=${}",
            entry.rank,
            entry
                .market_name
                .as_deref()
                .unwrap_or(entry.market_pubkey.as_str()),
            entry.volume_24h_usd
        );
    }

    // ── History ──────────────────────────────────────────────────────────
    let history = client
        .metrics()
        .history("platform", "platform", &MetricsHistoryQuery::default())
        .await?;
    println!(
        "history platform/platform @ {}: {} buckets",
        history.resolution,
        history.points.len()
    );

    // ── Per-user metrics ────────────────────────────────────────────────
    // JWT (uses the SDK's process-wide token captured by the login above)
    let user_metrics = client.metrics().user().await?;
    println!(
        "user (jwt) {}: outcomes_traded={} volume=${} referrals_used={}",
        user_metrics.wallet_address,
        user_metrics.total_outcomes_traded,
        user_metrics.total_volume_usd,
        user_metrics.total_referrals_used
    );

    // Public path-based variant — no auth required.
    let by_wallet = client
        .metrics()
        .user_by_wallet(&keypair.pubkey().to_string())
        .await?;
    println!(
        "user (by-wallet) {}: outcomes_traded={} volume=${}",
        by_wallet.wallet_address, by_wallet.total_outcomes_traded, by_wallet.total_volume_usd
    );

    Ok(())
}
