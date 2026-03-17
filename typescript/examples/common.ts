import * as fs from "fs";
import * as path from "path";
import { Keypair, PublicKey } from "@solana/web3.js";
import { config } from "dotenv";

import { LightconeClient, type Market, type OrderBookPair } from "../src";
import { signLoginMessage, type User } from "../src/auth";
import type { OrderbookDecimals } from "../src/shared";

config({ path: path.resolve(__dirname, "../.env") });

export function restClient(): LightconeClient {
  return LightconeClient.builder().build();
}

export function rpcClient(): LightconeClient {
  const url = process.env.SOLANA_RPC_URL ?? "https://api.devnet.solana.com";
  return LightconeClient.builder().rpcUrl(url).build();
}

export function wallet(): Keypair {
  const walletPath = process.env.LIGHTCONE_WALLET_PATH;
  if (!walletPath) {
    throw new Error("LIGHTCONE_WALLET_PATH not set");
  }

  const resolved = walletPath.startsWith("~")
    ? path.join(process.env.HOME ?? "", walletPath.slice(1))
    : walletPath;

  const raw = fs.readFileSync(resolved, "utf-8");
  const secretKey = Uint8Array.from(JSON.parse(raw));
  return Keypair.fromSecretKey(secretKey);
}

export async function login(
  client: LightconeClient,
  keypair: Keypair
): Promise<User> {
  const nonce = await client.auth().getNonce();
  const signed = signLoginMessage(keypair, nonce);
  return client.auth().loginWithMessage(
    signed.message,
    signed.signature_bs58,
    signed.pubkey_bytes
  );
}

export async function market(client: LightconeClient): Promise<Market> {
  const result = await client.markets().get(undefined, 1);
  const m = result.markets[0];
  if (!m) throw new Error("No markets found");
  return m;
}

export async function marketAndOrderbook(
  client: LightconeClient
): Promise<[Market, OrderBookPair]> {
  const m = await market(client);
  const ob =
    m.orderbookPairs.find((p) => p.active) ?? m.orderbookPairs[0];
  if (!ob) throw new Error("Market has no orderbooks");
  return [m, ob];
}

export async function scalingDecimals(
  client: LightconeClient,
  orderbook: OrderBookPair
): Promise<OrderbookDecimals> {
  const decimals = await client.orderbooks().decimals(orderbook.orderbookId);
  return {
    orderbookId: decimals.orderbook_id,
    baseDecimals: decimals.base_decimals,
    quoteDecimals: decimals.quote_decimals,
    priceDecimals: decimals.price_decimals,
    tickSize: BigInt(Math.max(orderbook.tickSize, 0)),
  };
}

export function depositMint(m: Market): PublicKey {
  const asset = m.depositAssets[0];
  if (!asset) throw new Error("Market has no deposit assets");
  return new PublicKey(asset.pubkey);
}

export function numOutcomes(m: Market): number {
  return m.outcomes.length;
}

export async function freshOrderNonce(
  client: LightconeClient,
  user: PublicKey
): Promise<number> {
  return client.orders().currentNonce(user);
}

export function unixTimestamp(): number {
  return Math.floor(Date.now() / 1000);
}

export function unixTimestampMs(): number {
  return Date.now();
}

export async function withTimeout<T>(
  promise: Promise<T>,
  timeoutMs: number,
  message: string
): Promise<T> {
  let timer: ReturnType<typeof setTimeout> | undefined;

  try {
    return await new Promise<T>((resolve, reject) => {
      timer = setTimeout(() => {
        reject(new Error(message));
      }, timeoutMs);

      promise.then(resolve, reject);
    });
  } finally {
    if (timer) {
      clearTimeout(timer);
    }
  }
}
