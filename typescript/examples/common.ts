import * as fs from "fs";
import * as os from "os";
import * as path from "path";
import { Keypair, PublicKey } from "@solana/web3.js";

import { LightconeClient, LightconeEnv, type Market, type OrderBookPair } from "../src";
import { signLoginMessage, type User } from "../src/auth";

const DEFAULT_WALLET_PATH = "~/.config/solana/id.json";

/**
 * Build a LightconeClient.
 * Defaults to production. Set `LIGHTCONE_ENV` to override
 * (options: local, staging, prod).
 */
export function restClient(): LightconeClient {
  const builder = LightconeClient.builder();
  const envStr = process.env.LIGHTCONE_ENV?.toLowerCase();
  if (envStr) {
    builder.env(envStr as LightconeEnv);
  }
  return builder.build();
}

export function rpcClient(): LightconeClient {
  return restClient();
}

/**
 * Load a keypair from disk.
 * Defaults to `~/.config/solana/id.json`. Set `LIGHTCONE_WALLET_PATH`
 * to override.
 */
export function getKeypair(): Keypair {
  const walletFile = process.env.LIGHTCONE_WALLET_PATH ?? DEFAULT_WALLET_PATH;
  const resolved = walletFile.startsWith("~")
    ? path.join(os.homedir(), walletFile.slice(1))
    : walletFile;

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
