import { PublicKey } from "@solana/web3.js";
import { LimitOrderEnvelope } from "../src/program/envelope";
import {
  restClient,
  rpcClient,
  wallet,
  login,
  marketAndOrderbook,
  scalingDecimals,
  freshOrderNonce,
} from "./common";

async function main() {
  const client = restClient();
  const rpc = rpcClient();
  const keypair = wallet();

  // 1. Login
  await login(client, keypair);
  console.log("logged in:", keypair.publicKey.toBase58());

  // 2. Fetch market + orderbook + decimals
  const [m, orderbook] = await marketAndOrderbook(client);
  const decimals = await scalingDecimals(client, orderbook);
  console.log("market:", m.slug);
  console.log("orderbook:", orderbook.orderbookId);

  // 3. Get a fresh nonce from on-chain
  const nonce = await freshOrderNonce(rpc, keypair.publicKey);
  console.log("nonce:", nonce);

  // 4. Build, scale, sign a limit order
  const request = LimitOrderEnvelope.new()
    .maker(keypair.publicKey)
    .market(new PublicKey(m.pubkey))
    .baseMint(new PublicKey(orderbook.base.pubkey))
    .quoteMint(new PublicKey(orderbook.quote.pubkey))
    .bid()
    .price("0.55")
    .size("1")
    .nonce(nonce)
    .applyScaling(decimals)
    .sign(keypair, orderbook.orderbookId);

  // 5. Submit
  const response = await client.orders().submit(request);
  console.log("order hash:", response.order_hash);
  console.log("filled:", response.filled);
  console.log("remaining:", response.remaining);
  console.log("fills:", response.fills.length);
}

main().catch(console.error);
