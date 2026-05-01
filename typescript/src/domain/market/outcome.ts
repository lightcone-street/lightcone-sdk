import { resolveIconUrls } from "./icon";
import type { OutcomeResponse } from "./wire";

export interface Outcome {
  index: number;
  iconUrlLow: string;
  iconUrlMedium: string;
  iconUrlHigh: string;
  name: string;
}

export class OutcomeValidationError extends Error {
  readonly details: string[];

  constructor(name: string, details: string[]) {
    super(`Outcome validation errors (${name}): ${details.join("; ")}`);
    this.name = "OutcomeValidationError";
    this.details = details;
  }
}

export function outcomeFromWire(source: OutcomeResponse): Outcome {
  const iconUrls = resolveIconUrls(source.icon_url_low, source.icon_url_medium, source.icon_url_high);
  if (!iconUrls) {
    throw new OutcomeValidationError(source.name, ["Missing icon URL"]);
  }

  return {
    index: source.index,
    iconUrlLow: iconUrls.low,
    iconUrlMedium: iconUrls.medium,
    iconUrlHigh: iconUrls.high,
    name: source.name,
  };
}
