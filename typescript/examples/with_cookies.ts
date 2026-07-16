// Per-call cookie forwarding for SSR / route handlers.
//
// Demonstrates the `*WithCookies` variants on `Positions`, `Notifications`,
// `Referrals`, `Orders`, `Metrics`, and `Markets`. These bypass the SDK's process-wide
// auth_token store and forward the supplied raw `Cookie` header for that single
// call only — so a server can relay whatever auth cookies the browser sent
// (`privy-token` and/or `lightcone-token`).
//
// In a real SSR / route handler the header would be built from the incoming
// request's cookie jar. Here we mimic that by:
//   1. Logging in once (the SDK captures the `lightcone-token` internally).
//   2. Reading the token off the client via `authToken()` and wrapping it in a
//      `Cookie` header.
//   3. Clearing the SDK's internal token to prove the `*WithCookies` path
//      doesn't depend on it.
//   4. Calling each `*WithCookies` method with the captured header.

import { restClient, getKeypair, login, runExample } from "./common";

async function main() {
  const client = restClient();
  const keypair = getKeypair();
  await login(client, keypair);

  const authToken = await client.authToken();
  if (!authToken) {
    throw new Error("authToken not set after login — SDK should have captured it");
  }
  await client.clearAuthToken();

  // Native consumers authenticate with a lightcone-token; build the `Cookie`
  // header the same way a browser would send it.
  const cookieHeader = `lightcone-token=${authToken}`;

  const positions = await client.positions().positionsWithCookies(cookieHeader);
  console.log("markets with positions:", positions.total_markets);

  const balances = await client
    .positions()
    .depositTokenBalancesWithCookies(cookieHeader);
  console.log("tracked deposit balances:", Object.keys(balances).length);

  const notifications = await client.notifications().fetchWithCookies(cookieHeader);
  console.log("notifications:", notifications.length);

  const status = await client.referrals().getStatusWithCookies(cookieHeader);
  console.log("referral codes:", status.referralCodes.length);

  const orders = await client
    .orders()
    .getUserOrdersWithCookies(50, undefined, cookieHeader);
  console.log("open orders:", orders.orders.length);

  const fills = await client
    .orders()
    .getUserOrderFillsWithCookies(undefined, 50, undefined, cookieHeader);
  console.log("order fills:", fills.orders.length);

  const userMetrics = await client.metrics().userWithCookies(cookieHeader);
  console.log(
    `user metrics: volume_usd=${userMetrics.total_volume_usd} outcomes_traded=${userMetrics.total_outcomes_traded}`,
  );

  const favoriteMarketPubkeys: string[] = [];
  let favoriteCursor: number | undefined;
  while (true) {
    const favoritePage = await client
      .markets()
      .favoriteMarketsWithCookies(cookieHeader, 1000, favoriteCursor);
    favoriteMarketPubkeys.push(...favoritePage.market_pubkeys);
    if (!favoritePage.has_more) break;
    if (favoritePage.next_cursor == null) {
      throw new Error("Favorite page is missing next_cursor");
    }
    favoriteCursor = favoritePage.next_cursor;
  }
  console.log("favorite markets:", favoriteMarketPubkeys.length);

  const selectedMarket = (await client.markets().get(undefined, 1)).markets[0];
  if (selectedMarket) {
    const wasFavorited = favoriteMarketPubkeys.includes(selectedMarket.pubkey);
    if (wasFavorited) {
      await client.markets().removeFavoriteMarketWithCookies(selectedMarket.pubkey, cookieHeader);
      await client.markets().addFavoriteMarketWithCookies(selectedMarket.pubkey, cookieHeader);
    } else {
      await client.markets().addFavoriteMarketWithCookies(selectedMarket.pubkey, cookieHeader);
      await client.markets().removeFavoriteMarketWithCookies(selectedMarket.pubkey, cookieHeader);
    }
    console.log("restored favorite state for", selectedMarket.pubkey);
  }
}

void runExample(main);
