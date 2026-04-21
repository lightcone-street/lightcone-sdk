import type { PubkeyStr } from "../../shared";
import { asPubkeyStr } from "../../shared";
import type {
  ConditionalTokenResponse,
  DepositAssetResponse,
  GlobalDepositAssetResponse,
} from "./wire";

export interface Token {
  id: number;
  pubkey: PubkeyStr;
  name: string;
  symbol: string;
  description?: string;
  decimals: number;
  iconUrl: string;
}

export interface ConditionalToken extends Token {
  outcomeIndex: number;
  outcome: string;
  depositAsset: PubkeyStr;
  depositSymbol: string;
}

export interface DepositAsset extends Token {
  marketPda: PubkeyStr;
  depositAsset: PubkeyStr;
  numOutcomes: number;
}

/**
 * A base/quote pairing of two `DepositAsset`s.
 *
 * Populated on `Market.depositAssetPairs` during wire→domain conversion —
 * one entry per unique base/quote combination across the market's orderbook
 * pairs.
 */
export interface DepositAssetPair {
  /** Stable identifier of the form `"{basePubkey}-{quotePubkey}"`. */
  id: string;
  base: DepositAsset;
  quote: DepositAsset;
}

/**
 * A globally whitelisted deposit asset (platform-scoped, not market-bound).
 *
 * Distinct from `DepositAsset`, which is bound to a specific market.
 */
export interface GlobalDepositAsset extends Token {
  depositAsset: PubkeyStr;
  whitelistIndex: number;
  active: boolean;
}

export interface TokenMetadata {
  pubkey: PubkeyStr;
  symbol: string;
  decimals: number;
  iconUrl: string;
  name: string;
}

export interface ValidatedTokens {
  token: DepositAsset;
  conditionals: ConditionalToken[];
  metadata: Record<string, TokenMetadata>;
}

export class TokenValidationError extends Error {
  readonly details: string[];

  constructor(mint: string, details: string[]) {
    super(`Token validation errors (${mint}): ${details.join("; ")}`);
    this.name = "TokenValidationError";
    this.details = details;
  }
}

const USDC_MAINNET = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
const USDT_MAINNET = "Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB";
const USDC_DEVNET_LC = "7SrxsoXjNR7Y8T3koJCt1yV4FrNUumoAUrJExDt6tQez";

export function isUsdStablecoin(pubkey: string): boolean {
  return pubkey === USDC_MAINNET || pubkey === USDT_MAINNET || pubkey === USDC_DEVNET_LC;
}

export function currencySymbol(pubkey: string): "" | "$" {
  return isUsdStablecoin(pubkey) ? "$" : "";
}

export function validatedTokensFromWire(source: DepositAssetResponse): ValidatedTokens {
  const errors: string[] = [];
  const metadata: Record<string, TokenMetadata> = {};

  const depositPubkey = asPubkeyStr(source.deposit_asset);
  const iconUrl = source.icon_url;
  const name = source.display_name;
  const symbol = source.symbol;
  const decimals = source.decimals;

  if (!iconUrl) errors.push("Missing icon URL");
  if (!name) errors.push("Missing display name");
  if (!symbol) errors.push("Missing symbol");
  if (decimals === undefined) errors.push("Missing decimals");

  if (errors.length > 0) {
    throw new TokenValidationError(source.deposit_asset, errors);
  }

  metadata[source.deposit_asset] = {
    pubkey: depositPubkey,
    symbol: symbol ?? "",
    decimals: decimals ?? 0,
    iconUrl: iconUrl ?? "",
    name: name ?? "",
  };

  const conditionals = source.conditional_mints.map((conditional) =>
    conditionalFromWire(conditional, source.deposit_asset, symbol ?? "", iconUrl ?? "")
  );

  for (const conditional of conditionals) {
    metadata[conditional.pubkey] = {
      pubkey: conditional.pubkey,
      symbol: conditional.symbol,
      decimals: conditional.decimals,
      iconUrl: conditional.iconUrl,
      name: conditional.name,
    };
  }

  return {
    token: {
      id: source.id,
      marketPda: asPubkeyStr(source.market_pubkey),
      depositAsset: depositPubkey,
      numOutcomes: source.num_outcomes,
      pubkey: depositPubkey,
      name: name ?? "",
      symbol: symbol ?? "",
      description: source.description,
      decimals: decimals ?? 0,
      iconUrl: iconUrl ?? "",
    },
    conditionals,
    metadata,
  };
}

export function globalDepositAssetFromWire(
  source: GlobalDepositAssetResponse
): GlobalDepositAsset {
  const errors: string[] = [];

  const name = source.display_name;
  const symbol = source.symbol;
  const iconUrl = source.icon_url;
  const decimals = source.decimals;

  if (!name) errors.push("Missing display name");
  if (!symbol) errors.push("Missing symbol");
  if (!iconUrl) errors.push("Missing icon URL");
  if (decimals === undefined || decimals === null) errors.push("Missing decimals");

  if (errors.length > 0) {
    throw new TokenValidationError(source.mint, errors);
  }

  const depositAsset = asPubkeyStr(source.mint);
  return {
    id: source.id,
    pubkey: depositAsset,
    depositAsset,
    name: name ?? "",
    symbol: symbol ?? "",
    description: source.description,
    decimals: decimals ?? 0,
    iconUrl: iconUrl ?? "",
    whitelistIndex: source.whitelist_index,
    active: source.active,
  };
}

function conditionalFromWire(
  source: ConditionalTokenResponse,
  depositAsset: string,
  depositSymbol: string,
  iconUrl: string
): ConditionalToken {
  const errors: string[] = [];
  if (source.decimals === undefined) errors.push("Missing decimals");
  if (!source.short_symbol) errors.push("Missing short_symbol");
  if (!source.outcome) errors.push("Missing outcome");

  if (errors.length > 0) {
    throw new TokenValidationError(source.token_address, errors);
  }

  return {
    id: source.id,
    outcomeIndex: source.outcome_index,
    outcome: source.outcome ?? "",
    depositAsset: asPubkeyStr(depositAsset),
    depositSymbol,
    pubkey: asPubkeyStr(source.token_address),
    name: source.outcome ?? "",
    symbol: source.short_symbol ?? "",
    description: source.description,
    decimals: source.decimals ?? 0,
    iconUrl,
  };
}
