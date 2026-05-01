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
  const errors: string[] = [];
  if (!source.icon_url_low) {
    errors.push("Missing thumbnail URL (low)");
  }
  if (!source.icon_url_medium) {
    errors.push("Missing thumbnail URL (medium)");
  }
  if (!source.icon_url_high) {
    errors.push("Missing thumbnail URL (high)");
  }

  if (errors.length > 0) {
    throw new OutcomeValidationError(source.name, errors);
  }

  return {
    index: source.index,
    iconUrlLow: source.icon_url_low ?? "",
    iconUrlMedium: source.icon_url_medium ?? "",
    iconUrlHigh: source.icon_url_high ?? "",
    name: source.name,
  };
}
