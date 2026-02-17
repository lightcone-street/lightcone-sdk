//! Admin sub-client — metadata management.

use crate::client::LightconeClient;
use crate::domain::admin::{AdminEnvelope, UnifiedMetadataRequest, UnifiedMetadataResponse};
use crate::error::SdkError;

/// Sub-client for admin operations.
pub struct Admin<'a> {
    pub(crate) client: &'a LightconeClient,
}

impl<'a> Admin<'a> {
    pub async fn upsert_metadata(
        &self,
        envelope: &AdminEnvelope<UnifiedMetadataRequest>,
    ) -> Result<UnifiedMetadataResponse, SdkError> {
        Ok(self.client.http.admin_upsert_metadata(envelope).await?)
    }
}
