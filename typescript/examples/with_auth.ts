// Per-call auth-token forwarding for SSR / route handlers.
//
// Demonstrates the `*WithAuth` variants on `Positions`, `Notifications`,
// `Referrals`, `Orders`, and `Metrics`. These bypass the SDK's process-wide
// auth_token store and pass the supplied token as a `Cookie: auth_token=…`
// header for that single call only.
//
// In a real SSR / route handler the token would be extracted from the
// incoming request's cookie jar. Here we mimic that by:
//   1. Logging in once (the SDK captures the token internally).
//   2. Reading the token off the client via `authToken()`.
//   3. Clearing the SDK's internal token to prove the `*WithAuth` path
//      doesn't depend on it.
//   4. Calling each `*WithAuth` method with the captured token.

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

  const positions = await client.positions().positionsWithAuth(authToken);
  console.log("markets with positions:", positions.total_markets);

  const balances = await client
    .positions()
    .depositTokenBalancesWithAuth(authToken);
  console.log("tracked deposit balances:", Object.keys(balances).length);

  const notifications = await client.notifications().fetchWithAuth(authToken);
  console.log("notifications:", notifications.length);

  const status = await client.referrals().getStatusWithAuth(authToken);
  console.log("referral codes:", status.referralCodes.length);

  const orders = await client
    .orders()
    .getUserOrdersWithAuth(50, undefined, authToken);
  console.log("open orders:", orders.orders.length);

  const fills = await client
    .orders()
    .getUserOrderFillsWithAuth(undefined, 50, undefined, authToken);
  console.log("order fills:", fills.orders.length);

  const userMetrics = await client.metrics().userWithAuth(authToken);
  console.log(
    `user metrics: volume_usd=${userMetrics.total_volume_usd} outcomes_traded=${userMetrics.total_outcomes_traded}`,
  );
}

void runExample(main);
