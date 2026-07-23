import { asPubkeyStr } from "../../shared";
import { MAX_OUTCOMES, MIN_OUTCOMES } from "../../program/constants";
import type { OrderBookPair } from "../orderbook";
import { orderBookPairFromWire } from "../orderbook/convert";
import { resolveIconUrls } from "./icon";
import { outcomeFromWire } from "./outcome";
import { statusFromWire, type Market, MarketValidationError, Status } from "./index";
import {
  globalDepositAssetFromWire,
  sortByDisplayPriority,
  validatedTokensFromWire,
  type DepositAsset,
  type DepositAssetPair,
} from "./tokens";
import type { MarketResponse } from "./wire";

export { globalDepositAssetFromWire, resolveIconUrls };

export function marketFromWire(source: MarketResponse): Market {
  const errors: string[] = [];
  const numOutcomes = source.num_outcomes ?? source.deposit_assets[0]?.num_outcomes ?? 0;
  if (!Number.isInteger(numOutcomes) || numOutcomes < MIN_OUTCOMES || numOutcomes > MAX_OUTCOMES) {
    errors.push(`Invalid outcome count: ${numOutcomes}`);
  }
  const inconsistentOutcomeCounts = source.deposit_assets
    .map((asset) => asset.num_outcomes)
    .filter((count) => count !== numOutcomes);
  if (inconsistentOutcomeCounts.length > 0) {
    errors.push(`Deposit asset outcome counts do not match market: ${inconsistentOutcomeCounts.join(", ")}`);
  }

  const outcomes = source.outcomes.flatMap((outcome) => {
    try {
      return [outcomeFromWire(outcome)];
    } catch (error) {
      errors.push(error instanceof Error ? error.message : String(error));
      return [];
    }
  });

  const depositAssets = [] as Market["depositAssets"];
  const conditionalTokens = [] as Market["conditionalTokens"];
  const tokenMetadata: Market["tokenMetadata"] = {};

  for (const depositAsset of source.deposit_assets) {
    try {
      const validated = validatedTokensFromWire(depositAsset);
      depositAssets.push(validated.token);
      conditionalTokens.push(...validated.conditionals);
      Object.assign(tokenMetadata, validated.metadata);
    } catch (error) {
      errors.push(error instanceof Error ? error.message : String(error));
    }
  }

  const orderbookPairs = source.orderbooks.flatMap((orderbook) => {
    try {
      return [orderBookPairFromWire(orderbook, conditionalTokens)];
    } catch (error) {
      errors.push(error instanceof Error ? error.message : String(error));
      return [];
    }
  });

  const status = statusFromWire(source.market_status);
  const definition = typeof source.definition === "string" ? source.definition : undefined;
  if (!source.slug) errors.push("Missing slug");
  if (!source.market_name) errors.push("Missing market name");
  if (!status) errors.push(`Invalid status: ${source.market_status}`);
  if (!definition) errors.push("Missing definition");

  const iconUrls = resolveIconUrls(source.icon_url_low, source.icon_url_medium, source.icon_url_high);
  if (!iconUrls) errors.push("Missing icon URL");

  // Banners are optional; cross-fallback still applies when any variant is set.
  const bannerUrls = resolveIconUrls(source.banner_image_url_low, source.banner_image_url_medium, source.banner_image_url_high);

  const depositAssetPairs = sortByDisplayPriority(
    deriveDepositAssetPairs(depositAssets, orderbookPairs),
  );

  if (depositAssetPairs.length === 0) {
    errors.push("Missing deposit asset pairs");
  }

  if (errors.length > 0) {
    throw new MarketValidationError(source.market_pubkey, errors);
  }

  return {
    id: source.market_id,
    pubkey: asPubkeyStr(source.market_pubkey),
    name: source.market_name ?? "",
    bannerImageUrlLow: bannerUrls?.low,
    bannerImageUrlMedium: bannerUrls?.medium,
    bannerImageUrlHigh: bannerUrls?.high,
    iconUrlLow: iconUrls?.low ?? "",
    iconUrlMedium: iconUrls?.medium ?? "",
    iconUrlHigh: iconUrls?.high ?? "",
    featuredRank: source.featured_rank,
    slug: source.slug ?? "",
    status: status ?? Status.Pending,
    createdAt: new Date(source.created_at),
    activatedAt: source.activated_at ? new Date(source.activated_at) : undefined,
    settledAt: source.settled_at ? new Date(source.settled_at) : undefined,
    resolutionBy: source.resolution_by ?? undefined,
    resolution: source.resolution,
    description: source.description,
    definition: definition as string,
    category: source.category,
    subcategory: source.subcategory,
    tags: source.tags ?? [],
    numOutcomes,
    depositAssets,
    depositAssetPairs,
    conditionalTokens,
    outcomes,
    orderbookPairs,
    orderbookIds: orderbookPairs.map((pair) => pair.orderbookId),
    tokenMetadata,
  };
}

/**
 * Derive unique base/quote deposit-asset pairs across the market's orderbook
 * pairs. Deduplicated by `(basePubkey, quotePubkey)`; orderbook pairs whose
 * base or quote deposit asset is not present in `depositAssets` are skipped.
 */
export function deriveDepositAssetPairs(
  depositAssets: DepositAsset[],
  orderbookPairs: OrderBookPair[],
): DepositAssetPair[] {
  const seen = new Map<string, DepositAssetPair>();

  for (const pair of orderbookPairs) {
    const base = depositAssets.find(
      (asset) => asset.depositAsset === pair.base.depositAsset,
    );
    const quote = depositAssets.find(
      (asset) => asset.depositAsset === pair.quote.depositAsset,
    );

    if (!base || !quote) continue;

    const key = `${base.depositAsset}|${quote.depositAsset}`;
    if (!seen.has(key)) {
      seen.set(key, {
        id: `${base.depositAsset}-${quote.depositAsset}`,
        base,
        quote,
      });
    }
  }

  return Array.from(seen.values());
}

export function tryMarketFromWire(source: MarketResponse): { market?: Market; error?: string } {
  try {
    return { market: marketFromWire(source) };
  } catch (error) {
    return { error: error instanceof Error ? error.message : String(error) };
  }
}
