export * from "./client";
export * from "./wire";

export interface ReferralCodeInfo {
  code: string;
  maxUses: number;
  useCount: number;
}

export interface ReferralStatus {
  isBeta: boolean;
  source?: string;
  referralCodes: ReferralCodeInfo[];
}

export interface RedeemResult {
  success: boolean;
  isBeta: boolean;
}
