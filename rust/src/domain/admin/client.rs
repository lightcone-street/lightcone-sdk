//! Admin sub-client — metadata and referral management.

use crate::client::LightconeClient;
use crate::domain::admin::{
    AdminEnvelope, AllocateCodesRequest, AllocateCodesResponse, RevokeRequest, RevokeResponse,
    UnifiedMetadataRequest, UnifiedMetadataResponse, UnrevokeRequest, UnrevokeResponse,
    WhitelistRequest, WhitelistResponse,
};
use crate::error::SdkError;
use crate::http::RetryPolicy;

pub struct Admin<'a> {
    pub(crate) client: &'a LightconeClient,
}

impl<'a> Admin<'a> {
    pub async fn upsert_metadata(
        &self,
        envelope: &AdminEnvelope<UnifiedMetadataRequest>,
    ) -> Result<UnifiedMetadataResponse, SdkError> {
        let url = format!("{}/api/admin/metadata", self.client.http.base_url());
        Ok(self
            .client
            .http
            .post(&url, envelope, RetryPolicy::None)
            .await?)
    }

    pub async fn allocate_codes(
        &self,
        envelope: &AdminEnvelope<AllocateCodesRequest>,
    ) -> Result<AllocateCodesResponse, SdkError> {
        let url = format!(
            "{}/api/admin/referral/allocate",
            self.client.http.base_url()
        );
        Ok(self
            .client
            .http
            .post(&url, envelope, RetryPolicy::None)
            .await?)
    }

    pub async fn whitelist(
        &self,
        envelope: &AdminEnvelope<WhitelistRequest>,
    ) -> Result<WhitelistResponse, SdkError> {
        let url = format!(
            "{}/api/admin/referral/whitelist",
            self.client.http.base_url()
        );
        Ok(self
            .client
            .http
            .post(&url, envelope, RetryPolicy::None)
            .await?)
    }

    pub async fn revoke(
        &self,
        envelope: &AdminEnvelope<RevokeRequest>,
    ) -> Result<RevokeResponse, SdkError> {
        let url = format!(
            "{}/api/admin/referral/revoke",
            self.client.http.base_url()
        );
        Ok(self
            .client
            .http
            .post(&url, envelope, RetryPolicy::None)
            .await?)
    }

    pub async fn unrevoke(
        &self,
        envelope: &AdminEnvelope<UnrevokeRequest>,
    ) -> Result<UnrevokeResponse, SdkError> {
        let url = format!(
            "{}/api/admin/referral/unrevoke",
            self.client.http.base_url()
        );
        Ok(self
            .client
            .http
            .post(&url, envelope, RetryPolicy::None)
            .await?)
    }
}
