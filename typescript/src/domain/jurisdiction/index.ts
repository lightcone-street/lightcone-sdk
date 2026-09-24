export * from "./client";
export * from "./wire";

export interface JurisdictionCapabilities {
  mode: string;
  canAuthenticate: boolean;
  canMutateAccount: boolean;
  canSubmitOrders: boolean;
  canCancelOrders: boolean;
}

export interface JurisdictionResponse {
  country: string | null;
  geoblocked: boolean;
  tier: string | null;
  policyVersion: string;
  frontend: JurisdictionCapabilities;
  api: JurisdictionCapabilities;
  relayed: boolean;
}
