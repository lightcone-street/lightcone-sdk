import { PublicKey } from "@solana/web3.js";
import { program } from "../src";
import {
  runExample,
  rpcClient,
  getKeypair,
  marketAndOrderbook,
  quoteDepositMint,
} from "./common";

async function main() {
  const client = rpcClient();
  const keypair = getKeypair();

  const [m, orderbook] = await marketAndOrderbook(client);
  const marketPubkey = new PublicKey(m.pubkey);
  const baseMint = new PublicKey(orderbook.base.pubkey);
  const quoteMint = new PublicKey(orderbook.quote.pubkey);

  const exchange = await client.rpc().getExchange();
  const onchainMarket = await client.markets().getOnchain(marketPubkey);
  let onchainOrderbook;
  try {
    onchainOrderbook = await client.orderbooks().getOnchain(baseMint, quoteMint);
  } catch {
    console.log("orderbook: not found on-chain");
  }
  const nonce = await client.orders().currentNonce(keypair.publicKey);
  const position = await client.positions().getOnchain(keypair.publicKey, marketPubkey);
  const dMint = quoteDepositMint(orderbook);

  console.log(
    `exchange: authority=${exchange.authority.toBase58()} operator=${exchange.operator.toBase58()} manager=${exchange.manager.toBase58()} paused=${exchange.paused}`
  );
  console.log(
    `market: id=${onchainMarket.marketId} outcomes=${onchainMarket.numOutcomes} status=${program.MarketStatus[onchainMarket.status]}`
  );
  if (onchainOrderbook) {
    console.log(
      `orderbook: lookup_table=${onchainOrderbook.lookupTable.toBase58()} base_index=${onchainOrderbook.baseIndex} bump=${onchainOrderbook.bump}`
    );
  }
  console.log(`user nonce: ${nonce}`);
  console.log(`position exists: ${position !== null}`);

  const rpc = client.rpc();
  console.log(
    `pdas: exchange=${rpc.getExchangePda().toBase58()} market=${client.markets().pda(onchainMarket.marketId).toBase58()} position=${client.positions().pda(keypair.publicKey, marketPubkey).toBase58()} global_deposit=${rpc.getGlobalDepositTokenPda(dMint).toBase58()}`
  );
}

void runExample(main);
