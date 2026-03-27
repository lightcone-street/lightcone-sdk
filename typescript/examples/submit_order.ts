import { PublicKey } from "@solana/web3.js";
import { generateSalt } from "../src/program";
import {
  rpcClient,
  wallet,
  login,
  marketAndOrderbook,
  freshOrderNonce,
} from "./common";

async function main() {
  const keypair = wallet();
  const client = rpcClient();
  client.setSigningStrategy({ type: "native", keypair });
  await login(client, keypair);

  const [_market, orderbook] = await marketAndOrderbook(client);

  // Fetch and cache the on-chain nonce once. Subsequent orders that omit
  // `.nonce()` will automatically use this cached value.
  const nonce = await freshOrderNonce(client, keypair.publicKey);
  client.setOrderNonce(nonce);

  // submit() auto-populates nonce from cache when `.nonce()` is not called.
  const response = await client
    .orders()
    .limitOrder()
    .maker(keypair.publicKey)
    .bid()
    .price("0.55")
    .size("2")
    .salt(generateSalt())
    .submit(client, orderbook);

  console.log(
    `submitted: ${response.order_hash} filled=${response.filled} remaining=${response.remaining} fills=${response.fills.length}`
  );
}

main().catch((error) => { console.error(error); process.exit(1); });
