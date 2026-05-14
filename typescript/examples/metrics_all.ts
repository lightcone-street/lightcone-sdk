// Exercise every metrics endpoint end-to-end.
//
// Useful as a wire-type smoke test: if any field parsing is wrong, this
// example will fail.
//
// Usage:
//   LIGHTCONE_ENV=local npx tsx examples/metrics_all.ts

import { getKeypair, login, restClient, runExample } from "./common";

async function main() {
  const client = restClient();
  const keypair = getKeypair();
  await login(client, keypair);

  const metrics = client.metrics();

  const platform = await metrics.platform();
  console.log(
    `platform: 24h=$${platform.volume_24h_usd}, ` +
      `7d=$${platform.volume_7d_usd}, ` +
      `open_interest=$${platform.open_interest_usd}, ` +
      `active_markets=${platform.active_markets}, ` +
      `active_orderbooks=${platform.active_orderbooks}`,
  );
  console.log("  deposit token volumes:", platform.deposit_token_volumes.length);

  const markets = await metrics.markets();
  console.log(`markets: ${markets.markets.length} entries (total=${markets.total})`);
  for (const entry of markets.markets.slice(0, 3)) {
    console.log(
      `  - ${entry.market_name ?? "?"} -- ` +
        `24h=$${entry.volume_24h_usd} ` +
        `(share=${entry.platform_volume_share_24h_pct}%)`,
    );
  }

  const topMarket = markets.markets[0];
  if (topMarket) {
    const detail = await metrics.market(topMarket.market_pubkey);
    console.log(
      `market detail ${detail.market_pubkey}: ` +
        `outcomes=${detail.outcome_volumes.length}, ` +
        `orderbooks=${detail.orderbook_volumes.length}`,
    );

    const firstOrderbook = detail.orderbook_volumes[0];
    if (firstOrderbook) {
      const orderbookMetrics = await metrics.orderbook(firstOrderbook.orderbook_id);
      console.log(
        `orderbook ${orderbookMetrics.orderbook_id}: ` +
          `24h_usd=$${orderbookMetrics.volume_24h_usd} ` +
          `24h_base=${orderbookMetrics.volume_24h_base}`,
      );
    }
  }

  const categories = await metrics.categories();
  console.log("categories:", categories.categories.length);
  const firstCategory = categories.categories[0];
  if (firstCategory) {
    const detail = await metrics.category(firstCategory.category);
    console.log(
      `category '${detail.category}': ` +
        `24h=$${detail.volume_24h_usd}, ` +
        `traders_24h=${detail.unique_traders_24h}`,
    );
  }

  const depositTokens = await metrics.depositTokens();
  console.log("deposit tokens:", depositTokens.deposit_tokens.length);

  const depositTokenHistory = await metrics.depositTokensVolumeHistory();
  console.log(
    `deposit token volume history @ ${depositTokenHistory.resolution}: ` +
      `${depositTokenHistory.points.length} days, ` +
      `total=$${depositTokenHistory.volume_total_usd}`,
  );

  const openInterestHistory = await metrics.openInterestHistory();
  console.log(
    `open interest history @ ${openInterestHistory.resolution}: ` +
      `${openInterestHistory.points.length} days, ` +
      `latest=$${openInterestHistory.latest_open_interest_usd}`,
  );

  const uniqueTradersHistory = await metrics.uniqueTradersHistory();
  console.log(
    `unique traders history @ ${uniqueTradersHistory.resolution}: ` +
      `${uniqueTradersHistory.points.length} days, ` +
      `latest=${uniqueTradersHistory.latest_unique_traders}`,
  );

  const board = await metrics.leaderboard(5);
  console.log(`leaderboard (${board.period}): ${board.entries.length} entries`);
  for (const entry of board.entries) {
    const name = entry.market_name ?? entry.market_pubkey;
    console.log(`  #${entry.rank} ${name} -- 24h=$${entry.volume_24h_usd}`);
  }

  const history = await metrics.history("platform", "platform");
  console.log(
    `history platform/platform @ ${history.resolution}: ` +
      `${history.points.length} buckets`,
  );

  const userMetrics = await metrics.user();
  console.log(
    `user (jwt) ${userMetrics.wallet_address}: ` +
      `outcomes_traded=${userMetrics.total_outcomes_traded} ` +
      `volume=$${userMetrics.total_volume_usd} ` +
      `referrals_used=${userMetrics.total_referrals_used}`,
  );

  const byWallet = await metrics.userByWallet(keypair.publicKey.toBase58());
  console.log(
    `user (by-wallet) ${byWallet.wallet_address}: ` +
      `outcomes_traded=${byWallet.total_outcomes_traded} ` +
      `volume=$${byWallet.total_volume_usd}`,
  );
}

void runExample(main);
