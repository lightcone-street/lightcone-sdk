//! Admin sub-client — metadata management.

use crate::client::LightconeClient;
use crate::domain::admin::{AdminEnvelope, UnifiedMetadataRequest, UnifiedMetadataResponse};
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
}
