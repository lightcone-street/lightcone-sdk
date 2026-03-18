import { PublicKey } from "@solana/web3.js";
import { LimitOrderEnvelope } from "../src/program";
import {
  rpcClient,
  wallet,
  login,
  marketAndOrderbook,
  freshOrderNonce,
} from "./common";

async function main() {
  const client = rpcClient();
  const keypair = wallet();
  await login(client, keypair);

  const [m, orderbook] = await marketAndOrderbook(client);
  const nonce = await freshOrderNonce(client, keypair.publicKey);

  const request = LimitOrderEnvelope.new()
    .maker(keypair.publicKey)
    .market(new PublicKey(m.pubkey))
    .baseMint(new PublicKey(orderbook.base.pubkey))
    .quoteMint(new PublicKey(orderbook.quote.pubkey))
    .bid()
    .price("0.55")
    .size("1")
    .nonce(nonce)
    .sign(keypair, orderbook);

  const response = await client.orders().submit(request);
  console.log(
    `submitted: ${response.order_hash} filled=${response.filled} remaining=${response.remaining} fills=${response.fills.length}`
  );
}

main().catch(console.error);
