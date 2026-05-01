import * as fs from "fs";
import * as os from "os";
import * as path from "path";
import { Connection, Keypair, PublicKey } from "@solana/web3.js";

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
    switch (envStr) {
      case LightconeEnv.Local:
      case LightconeEnv.Staging:
      case LightconeEnv.Prod:
        builder.env(envStr);
        break;
      default:
        throw new Error(
          `invalid LIGHTCONE_ENV '${envStr}'. Options: local, staging, prod`
        );
    }
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
  keypair: Keypair,
  useEmbeddedWallet = false
): Promise<User> {
  const nonce = await client.auth().getNonce();
  const signed = signLoginMessage(keypair, nonce);
  return client.auth().loginWithMessage(
    signed.message,
    signed.signature_bs58,
    signed.pubkey_bytes,
    useEmbeddedWallet
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

export function quoteDepositMint(orderbook: OrderBookPair): PublicKey {
  return new PublicKey(orderbook.quote.depositAsset);
}

export function numOutcomes(m: Market): number {
  return m.outcomes.length;
}

export async function waitForGlobalBalance(
  client: LightconeClient,
  mint: PublicKey,
  minimumAmount: number,
  timeoutMs = 30_000,
  intervalMs = 2_000,
): Promise<void> {
  const deadline = Date.now() + timeoutMs;
  const mintStr = mint.toBase58();
  let attempt = 0;
  console.log(`waiting for global balance: mint=${mintStr} required=${minimumAmount}`);
  while (Date.now() < deadline) {
    attempt++;
    const balances = await client.positions().depositTokenBalances();
    const entry = Object.values(balances).find((balance) => balance.mint === mintStr);
    const currentIdle = entry ? Number(entry.idle) : 0;
    const symbol = entry?.symbol ?? "unknown";
    if (currentIdle >= minimumAmount) {
      console.log(`global balance ready: ${symbol} idle=${currentIdle} (attempt ${attempt})`);
      return;
    }
    const remainingMs = deadline - Date.now();
    console.log(
      `global balance not ready: ${symbol} idle=${currentIdle}/${minimumAmount} ` +
      `(attempt ${attempt}, ${Math.round(remainingMs / 1000)}s remaining)`
    );
    await new Promise((resolve) => setTimeout(resolve, intervalMs));
  }
  throw new Error(
    `global balance for ${mintStr} did not reach ${minimumAmount} within ${timeoutMs}ms`,
  );
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

export function formatError(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  if (typeof error === "string") {
    return error;
  }
  try {
    return JSON.stringify(error);
  } catch {
    return String(error);
  }
}

export async function runExample(main: () => Promise<void>): Promise<void> {
  try {
    await main();
  } catch (error) {
    console.error(formatError(error));
    process.exit(1);
  }
}

export async function confirmTransactionOrThrow(
  connection: Connection,
  signature: string,
  context?: {
    blockhash: string;
    lastValidBlockHeight: number;
  }
): Promise<void> {
  const confirmation = context
    ? await connection.confirmTransaction({
        signature,
        blockhash: context.blockhash,
        lastValidBlockHeight: context.lastValidBlockHeight,
      })
    : await connection.confirmTransaction(signature);

  if (confirmation.value.err) {
    throw new Error(`Transaction failed: ${JSON.stringify(confirmation.value.err)}`);
  }
}
