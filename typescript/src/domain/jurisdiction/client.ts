/** Jurisdiction sub-client for the current direct API caller. */
import type { ClientContext } from "../../context";
import { RetryPolicy } from "../../http";
import type { JurisdictionCapabilities, JurisdictionResponse } from "./index";
import type { JurisdictionCapabilitiesWire, JurisdictionResponseWire } from "./wire";

function capabilitiesFromWire(wire: JurisdictionCapabilitiesWire): JurisdictionCapabilities {
  return {
    mode: wire.mode,
    canAuthenticate: wire.can_authenticate,
    canMutateAccount: wire.can_mutate_account,
    canSubmitOrders: wire.can_submit_orders,
    canCancelOrders: wire.can_cancel_orders,
  };
}

export class Jurisdiction {
  constructor(private readonly client: ClientContext) {}

  /** GET /api/geoblock using this client's ordinary API transport. */
  async geoblock(): Promise<JurisdictionResponse> {
    const url = `${this.client.http.baseUrl()}/api/geoblock`;
    const wire = await this.client.http.get<JurisdictionResponseWire>(url, RetryPolicy.Idempotent);
    // The relay stamp is required by the backend contract. Do not silently
    // treat an older response without it as a direct classification.
    if (typeof wire.relayed !== "boolean") {
      throw new Error("Jurisdiction response is missing the relay stamp");
    }
    return {
      country: wire.country,
      geoblocked: wire.geoblocked,
      tier: wire.tier,
      policyVersion: wire.policy_version,
      frontend: capabilitiesFromWire(wire.frontend),
      api: capabilitiesFromWire(wire.api),
      relayed: wire.relayed,
    };
  }
}
