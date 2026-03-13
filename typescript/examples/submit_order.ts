import { PublicKey } from "@solana/web3.js";
import { LimitOrderEnvelope } from "../src/program";
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
  await login(client, keypair);

  const [m, orderbook] = await marketAndOrderbook(client);
  const decimals = await scalingDecimals(client, orderbook);
  const nonce = await freshOrderNonce(rpc, keypair.publicKey);

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

  const response = await client.orders().submit(request);
  console.log(
    `submitted: ${response.order_hash} filled=${response.filled} remaining=${response.remaining} fills=${response.fills.length}`
  );
}

main().catch(console.error);
