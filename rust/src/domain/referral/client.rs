//! Referral sub-client — beta status and code redemption.

use crate::client::LightconeClient;
use crate::domain::referral::wire::{RedeemRequest, RedeemResponse, ReferralStatusResponse};
use crate::domain::referral::{RedeemResult, ReferralCodeInfo, ReferralStatus};
use crate::error::SdkError;
use crate::http::RetryPolicy;

pub struct Referrals<'a> {
    pub(crate) client: &'a LightconeClient,
}

impl<'a> Referrals<'a> {
    pub async fn get_status(&self) -> Result<ReferralStatus, SdkError> {
        let url = format!("{}/api/referral/status", self.client.http.base_url());
        let resp: ReferralStatusResponse =
            self.client.http.get(&url, RetryPolicy::Idempotent).await?;

        Ok(referral_status_from_wire(resp))
    }

    /// Same as [`Self::get_status`], but uses the supplied `auth_token` for
    /// this call instead of the SDK's process-wide token store. For
    /// server-side cookie forwarding (SSR / server functions).
    pub async fn get_status_with_auth_override(
        &self,
        auth_token: &str,
    ) -> Result<ReferralStatus, SdkError> {
        let url = format!("{}/api/referral/status", self.client.http.base_url());
        let resp: ReferralStatusResponse = self
            .client
            .http
            .get_with_auth(&url, RetryPolicy::Idempotent, auth_token)
            .await?;
        Ok(referral_status_from_wire(resp))
    }

    pub async fn redeem(&self, code: &str) -> Result<RedeemResult, SdkError> {
        let url = format!("{}/api/referral/redeem", self.client.http.base_url());
        let body = RedeemRequest {
            code: code.to_string(),
        };
        let resp: RedeemResponse = self
            .client
            .http
            .post(&url, &body, RetryPolicy::None)
            .await?;

        Ok(RedeemResult {
            success: resp.success,
            is_beta: resp.is_beta,
        })
    }
}

fn referral_status_from_wire(resp: ReferralStatusResponse) -> ReferralStatus {
    ReferralStatus {
        is_beta: resp.is_beta,
        source: resp.source,
        referral_codes: resp
            .referral_codes
            .into_iter()
            .map(|c| ReferralCodeInfo {
                code: c.code,
                max_uses: c.max_uses,
                use_count: c.use_count,
            })
            .collect(),
    }
}
