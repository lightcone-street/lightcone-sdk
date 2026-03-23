import {
  rpcClient,
  wallet,
  login,
  marketAndOrderbook,
  freshOrderNonce,
} from "./common";
import { generateSalt } from "../src/program/orders";

async function main() {
  const client = rpcClient();
  const keypair = wallet();
  await login(client, keypair);

  const [_market, orderbook] = await marketAndOrderbook(client);

  const request = client
    .orders()
    .limitOrder()
    .maker(keypair.publicKey)
    .bid()
    .price("0.55")
    .size("1")
    .nonce(await freshOrderNonce(client, keypair.publicKey))
    .salt(generateSalt())
    .sign(keypair, orderbook);

  const response = await client.orders().submit(request);
  console.log(
    `submitted: ${response.order_hash} filled=${response.filled} remaining=${response.remaining} fills=${response.fills.length}`
  );
}

main().catch(console.error);
