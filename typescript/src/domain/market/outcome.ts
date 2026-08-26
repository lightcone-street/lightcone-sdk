import { resolveIconUrls } from "./icon";
import type { OutcomeResponse } from "./wire";

/** One market result whose artwork qualities are all populated or all absent. */
export interface Outcome {
  index: number;
  /** Optional low-quality artwork, cross-filled when another quality exists. */
  iconUrlLow?: string;
  /** Optional medium-quality artwork, cross-filled when another quality exists. */
  iconUrlMedium?: string;
  /** Optional high-quality artwork, cross-filled when another quality exists. */
  iconUrlHigh?: string;
  name: string;
  nameLong?: string;
}

/** Retained conversion error surface; absent outcome artwork no longer throws it. */
export class OutcomeValidationError extends Error {
  readonly details: string[];

  constructor(name: string, details: string[]) {
    super(`Outcome validation errors (${name}): ${details.join("; ")}`);
    this.name = "OutcomeValidationError";
    this.details = details;
  }
}

/** Converts outcome metadata while preserving non-blank URLs and optional artwork. */
export function outcomeFromWire(source: OutcomeResponse): Outcome {
  const iconUrls = resolveIconUrls(
    nonBlank(source.icon_url_low),
    nonBlank(source.icon_url_medium),
    nonBlank(source.icon_url_high),
  );

  return {
    index: source.index,
    iconUrlLow: iconUrls?.low,
    iconUrlMedium: iconUrls?.medium,
    iconUrlHigh: iconUrls?.high,
    name: source.name,
    nameLong: source.name_long,
  };
}

/** Treats blank outcome artwork metadata as absent without changing non-blank URLs. */
function nonBlank(value: string | undefined | null): string | undefined {
  return value?.trim() ? value : undefined;
}
