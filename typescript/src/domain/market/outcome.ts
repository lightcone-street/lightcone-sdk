import type { OutcomeResponse } from "./wire";

export interface Outcome {
  index: number;
  iconUrl: string;
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
  if (!source.icon_url) {
    errors.push("Missing thumbnail URL");
  }

  if (errors.length > 0) {
    throw new OutcomeValidationError(source.name, errors);
  }

  return {
    index: source.index,
    iconUrl: source.icon_url ?? "",
    name: source.name,
  };
}
