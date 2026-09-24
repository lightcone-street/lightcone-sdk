/** Jurisdiction sub-client for the current direct API caller. */
import type { ClientContext } from "../../context";
import { RetryPolicy } from "../../http";
import type { JurisdictionCapabilities, JurisdictionResponse } from "./index";

function record(value: unknown): Record<string, unknown> {
  if (typeof value !== "object" || value === null || Array.isArray(value)) {
    throw new Error("Invalid jurisdiction response");
  }
  return value as Record<string, unknown>;
}

function boolean(value: unknown): boolean {
  if (typeof value !== "boolean") throw new Error("Invalid jurisdiction response");
  return value;
}

function string(value: unknown): string {
  if (typeof value !== "string") throw new Error("Invalid jurisdiction response");
  return value;
}

function nullableString(value: unknown): string | null {
  if (value !== null && typeof value !== "string") throw new Error("Invalid jurisdiction response");
  return value;
}

function capabilitiesFromWire(value: unknown): JurisdictionCapabilities {
  const wire = record(value);
  return {
    mode: string(wire.mode),
    canAuthenticate: boolean(wire.can_authenticate),
    canMutateAccount: boolean(wire.can_mutate_account),
    canSubmitOrders: boolean(wire.can_submit_orders),
    canCancelOrders: boolean(wire.can_cancel_orders),
  };
}

export class Jurisdiction {
  constructor(private readonly client: ClientContext) {}

  /** GET /api/geoblock using this client's ordinary API transport. */
  async geoblock(): Promise<JurisdictionResponse> {
    const url = `${this.client.http.baseUrl()}/api/geoblock`;
    const wire = record(await this.client.http.get<unknown>(url, RetryPolicy.Idempotent));
    // The relay stamp is required by the backend contract. Do not silently
    // treat an older response without it as a direct classification.
    if (typeof wire.relayed !== "boolean") {
      throw new Error("Jurisdiction response is missing the relay stamp");
    }
    return {
      country: nullableString(wire.country),
      geoblocked: boolean(wire.geoblocked),
      tier: nullableString(wire.tier),
      policyVersion: string(wire.policy_version),
      frontend: capabilitiesFromWire(wire.frontend),
      api: capabilitiesFromWire(wire.api),
      relayed: wire.relayed,
    };
  }
}
