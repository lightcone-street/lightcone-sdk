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
  iconUrlLow: string;
  iconUrlMedium: string;
  iconUrlHigh: string;
}

/**
 * Display priority for sorting: lower values come first. BTC/WBTC tie at 0,
 * ETH/WETH tie at 1, SOL at 2; everything else falls to the alphabetical tail.
 */
export function tokenDisplayPriority(token: Pick<Token, "symbol">): number {
  switch (token.symbol) {
    case "BTC":
    case "WBTC":
      return 0;
    case "ETH":
    case "WETH":
      return 1;
    case "SOL":
      return 2;
    default:
      return 255;
  }
}

/**
 * Shape accepted by {@link sortByDisplayPriority}. Either a token with a
 * top-level `symbol`, or a composite (e.g. `DepositAssetPair`, `OrderBookPair`)
 * that carries its display token on `base`.
 */
export type DisplaySortable =
  | Pick<Token, "symbol">
  | { readonly base: Pick<Token, "symbol"> };

function displaySymbol(item: DisplaySortable): string {
  return "symbol" in item ? item.symbol : item.base.symbol;
}

/**
 * Returns a new array ordered for display: priority groups first
 * (BTC/WBTC → ETH/WETH → SOL), then all remaining items alphabetically by the
 * display token's symbol.
 *
 * Accepts both pure tokens and composite types whose display token lives on
 * `base` (e.g. `DepositAssetPair`, `OrderBookPair`).
 */
export function sortByDisplayPriority<T extends DisplaySortable>(items: readonly T[]): T[] {
  const copy = [...items];
  copy.sort((left, right) => {
    const leftSymbol = displaySymbol(left);
    const rightSymbol = displaySymbol(right);
    const priorityDelta =
      tokenDisplayPriority({ symbol: leftSymbol }) -
      tokenDisplayPriority({ symbol: rightSymbol });
    if (priorityDelta !== 0) return priorityDelta;
    return leftSymbol.localeCompare(rightSymbol);
  });
  return copy;
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
  iconUrlLow: string;
  iconUrlMedium: string;
  iconUrlHigh: string;
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
  const iconUrlLow = source.icon_url_low;
  const iconUrlMedium = source.icon_url_medium;
  const iconUrlHigh = source.icon_url_high;
  const name = source.display_name;
  const symbol = source.symbol;
  const decimals = source.decimals;

  if (!iconUrlLow) errors.push("Missing icon URL (low)");
  if (!iconUrlMedium) errors.push("Missing icon URL (medium)");
  if (!iconUrlHigh) errors.push("Missing icon URL (high)");
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
    iconUrlLow: iconUrlLow ?? "",
    iconUrlMedium: iconUrlMedium ?? "",
    iconUrlHigh: iconUrlHigh ?? "",
    name: name ?? "",
  };

  const conditionals = source.conditional_mints.map((conditional) =>
    conditionalFromWire(conditional, source.deposit_asset, symbol ?? "", iconUrlLow ?? "", iconUrlMedium ?? "", iconUrlHigh ?? "")
  );

  for (const conditional of conditionals) {
    metadata[conditional.pubkey] = {
      pubkey: conditional.pubkey,
      symbol: conditional.symbol,
      decimals: conditional.decimals,
      iconUrlLow: conditional.iconUrlLow,
      iconUrlMedium: conditional.iconUrlMedium,
      iconUrlHigh: conditional.iconUrlHigh,
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
      iconUrlLow: iconUrlLow ?? "",
      iconUrlMedium: iconUrlMedium ?? "",
      iconUrlHigh: iconUrlHigh ?? "",
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
  const iconUrlLow = source.icon_url_low;
  const iconUrlMedium = source.icon_url_medium;
  const iconUrlHigh = source.icon_url_high;
  const decimals = source.decimals;

  if (!name) errors.push("Missing display name");
  if (!symbol) errors.push("Missing symbol");
  if (!iconUrlLow) errors.push("Missing icon URL (low)");
  if (!iconUrlMedium) errors.push("Missing icon URL (medium)");
  if (!iconUrlHigh) errors.push("Missing icon URL (high)");
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
    iconUrlLow: iconUrlLow ?? "",
    iconUrlMedium: iconUrlMedium ?? "",
    iconUrlHigh: iconUrlHigh ?? "",
    whitelistIndex: source.whitelist_index,
    active: source.active,
  };
}

function conditionalFromWire(
  source: ConditionalTokenResponse,
  depositAsset: string,
  depositSymbol: string,
  iconUrlLow: string,
  iconUrlMedium: string,
  iconUrlHigh: string
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
    iconUrlLow,
    iconUrlMedium,
    iconUrlHigh,
  };
}
