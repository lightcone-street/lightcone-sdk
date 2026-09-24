/** Exact response fields returned by GET /api/geoblock. */
export interface JurisdictionCapabilitiesWire {
  mode: string;
  can_authenticate: boolean;
  can_mutate_account: boolean;
  can_submit_orders: boolean;
  can_cancel_orders: boolean;
}

export interface JurisdictionResponseWire {
  country: string | null;
  geoblocked: boolean;
  tier: string | null;
  policy_version: string;
  frontend: JurisdictionCapabilitiesWire;
  api: JurisdictionCapabilitiesWire;
  relayed: boolean;
}
