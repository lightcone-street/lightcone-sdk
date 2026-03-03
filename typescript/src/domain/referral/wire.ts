export interface ReferralCodeWire {
  code: string;
  max_uses: number;
  use_count: number;
}

export interface ReferralStatusResponse {
  is_beta: boolean;
  source?: string;
  referral_codes: ReferralCodeWire[];
}

export interface RedeemRequest {
  code: string;
}

export interface RedeemResponse {
  success: boolean;
  is_beta: boolean;
}

export interface RedeemErrorResponse {
  error: string;
  code: string;
}
