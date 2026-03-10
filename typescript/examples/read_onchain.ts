import { PublicKey } from "@solana/web3.js";
import {
  restClient,
  rpcClient,
  wallet,
  marketAndOrderbook,
  depositMint,
} from "./common";

async function main() {
  const client = restClient();
  const rpc = rpcClient();
  const keypair = wallet();

  const [m, orderbook] = await marketAndOrderbook(client);
  const marketPubkey = new PublicKey(m.pubkey);
  const baseMint = new PublicKey(orderbook.base.pubkey);
  const quoteMint = new PublicKey(orderbook.quote.pubkey);

  // 1. Exchange state
  const exchange = await rpc.getExchange();
  console.log("exchange authority:", exchange.authority.toBase58());
  console.log("exchange operator:", exchange.operator.toBase58());
  console.log("exchange paused:", exchange.paused);

  // 2. Market state
  const onchainMarket = await rpc.getMarketByPubkey(marketPubkey);
  console.log("market id:", onchainMarket.marketId.toString());
  console.log("num outcomes:", onchainMarket.numOutcomes);
  console.log("market status:", onchainMarket.status);

  // 3. Orderbook
  const ob = await rpc.getOrderbook(baseMint, quoteMint);
  if (ob) {
    console.log("orderbook bump:", ob.bump);
  } else {
    console.log("orderbook not found on-chain");
  }

  // 4. User nonce
  const nonce = await rpc.getCurrentNonce(keypair.publicKey);
  console.log("user nonce:", nonce);

  // 5. Position
  const position = await rpc.getPosition(keypair.publicKey, marketPubkey);
  console.log("position:", position ? "exists" : "none");

  // 6. PDA derivations
  console.log("exchange PDA:", rpc.getExchangePda().toBase58());
  console.log("position PDA:", rpc.getPositionPda(keypair.publicKey, marketPubkey).toBase58());
  console.log("user nonce PDA:", rpc.getUserNoncePda(keypair.publicKey).toBase58());

  const dMint = depositMint(m);
  console.log("global deposit token PDA:", rpc.getGlobalDepositTokenPda(dMint).toBase58());
}

main().catch(console.error);
