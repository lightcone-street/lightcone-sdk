import { PublicKey } from "@solana/web3.js";
import { program } from "../src";
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

  const exchange = await rpc.getExchange();
  const onchainMarket = await rpc.getMarketByPubkey(marketPubkey);
  const onchainOrderbook = await rpc.getOrderbook(baseMint, quoteMint);
  const nonce = await rpc.getCurrentNonce(keypair.publicKey);
  const position = await rpc.getPosition(keypair.publicKey, marketPubkey);
  const dMint = depositMint(m);

  console.log(
    `exchange: authority=${exchange.authority.toBase58()} operator=${exchange.operator.toBase58()} paused=${exchange.paused}`
  );
  console.log(
    `market: id=${onchainMarket.marketId} outcomes=${onchainMarket.numOutcomes} status=${program.MarketStatus[onchainMarket.status]}`
  );
  console.log(
    `orderbook: lookup_table=${onchainOrderbook.lookupTable.toBase58()} bump=${onchainOrderbook.bump}`
  );
  console.log(`user nonce: ${nonce}`);
  console.log(`position exists: ${position !== null}`);
  console.log(
    `pdas: exchange=${rpc.getExchangePda().toBase58()} market=${rpc.getMarketPda(onchainMarket.marketId).toBase58()} position=${rpc.getPositionPda(keypair.publicKey, marketPubkey).toBase58()} global_deposit=${rpc.getGlobalDepositTokenPda(dMint).toBase58()}`
  );
}

main().catch(console.error);
