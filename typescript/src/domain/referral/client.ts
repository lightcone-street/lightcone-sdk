import { RetryPolicy, type LightconeHttp } from "../../http";
import type { RedeemResult, ReferralStatus } from "./index";
import type { RedeemRequest, RedeemResponse, ReferralStatusResponse } from "./wire";

interface ClientContext {
  http: LightconeHttp;
}

export class Referrals {
  constructor(private readonly client: ClientContext) {}

  async getStatus(): Promise<ReferralStatus> {
    const url = `${this.client.http.baseUrl()}/api/referral/status`;
    const response = await this.client.http.get<ReferralStatusResponse>(url, RetryPolicy.Idempotent);
    return referralStatusFromWire(response);
  }

  /**
   * Same as {@link getStatus}, but uses the supplied `authToken` for this
   * call instead of the SDK's process-wide cookie store. For server-side
   * cookie forwarding (SSR / route handlers).
   */
  async getStatusWithAuth(authToken: string): Promise<ReferralStatus> {
    const url = `${this.client.http.baseUrl()}/api/referral/status`;
    const response = await this.client.http.getWithAuth<ReferralStatusResponse>(
      url,
      RetryPolicy.Idempotent,
      authToken,
    );
    return referralStatusFromWire(response);
  }

  async redeem(code: string): Promise<RedeemResult> {
    const url = `${this.client.http.baseUrl()}/api/referral/redeem`;
    const body: RedeemRequest = { code };
    const response = await this.client.http.post<RedeemResponse, RedeemRequest>(
      url,
      body,
      RetryPolicy.None
    );

    return {
      success: response.success,
      isBeta: response.is_beta,
    };
  }
}

function referralStatusFromWire(response: ReferralStatusResponse): ReferralStatus {
  return {
    isBeta: response.is_beta,
    source: response.source,
    referralCodes: response.referral_codes.map((code) => ({
      code: code.code,
      maxUses: code.max_uses,
      useCount: code.use_count,
    })),
  };
}
