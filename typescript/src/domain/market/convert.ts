import { asPubkeyStr } from "../../shared";
import { orderBookPairFromWire } from "../orderbook/convert";
import { outcomeFromWire } from "./outcome";
import { statusFromWire, type Market, MarketValidationError, Status } from "./index";
import { globalDepositAssetFromWire, validatedTokensFromWire } from "./tokens";
import type { MarketResponse } from "./wire";

export { globalDepositAssetFromWire };

export function marketFromWire(source: MarketResponse): Market {
  const errors: string[] = [];

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
  if (!source.slug) errors.push("Missing slug");
  if (!source.market_name) errors.push("Missing market name");
  if (!status) errors.push(`Invalid status: ${source.market_status}`);
  if (!source.description) errors.push("Missing description");
  if (!source.definition) errors.push("Missing definition");
  if (!source.icon_url) errors.push("Missing icon URL");
  if (!source.banner_image_url) errors.push("Missing banner image URL");

  if (errors.length > 0) {
    throw new MarketValidationError(source.market_pubkey, errors);
  }

  return {
    id: source.market_id,
    pubkey: asPubkeyStr(source.market_pubkey),
    name: source.market_name ?? "",
    bannerImageUrl: source.banner_image_url ?? "",
    iconUrl: source.icon_url ?? "",
    featuredRank: source.featured_rank,
    volume: "0",
    slug: source.slug ?? "",
    status: status ?? Status.Pending,
    createdAt: new Date(source.created_at),
    activatedAt: source.activated_at ? new Date(source.activated_at) : undefined,
    settledAt: source.settled_at ? new Date(source.settled_at) : undefined,
    winningOutcome: source.winning_outcome,
    description: source.description ?? "",
    definition: source.definition ?? "",
    category: source.category,
    tags: source.tags ?? [],
    depositAssets,
    conditionalTokens,
    outcomes,
    orderbookPairs,
    orderbookIds: orderbookPairs.map((pair) => pair.orderbookId),
    tokenMetadata,
  };
}

export function tryMarketFromWire(source: MarketResponse): { market?: Market; error?: string } {
  try {
    return { market: marketFromWire(source) };
  } catch (error) {
    return { error: error instanceof Error ? error.message : String(error) };
  }
}
