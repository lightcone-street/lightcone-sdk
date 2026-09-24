//! Jurisdiction sub-client.

use super::wire::JurisdictionResponse;
use crate::client::LightconeClient;
use crate::error::SdkError;
use crate::http::RetryPolicy;
#[cfg(not(target_arch = "wasm32"))]
use crate::http::{RelayContext, Relayed};

/// Sub-client for `GET /api/geoblock`.
pub struct Jurisdiction<'a> {
    pub(crate) client: &'a LightconeClient,
}

impl<'a> Jurisdiction<'a> {
    fn url(&self) -> String {
        format!("{}/api/geoblock", self.client.http.base_url())
    }

    /// The jurisdiction classification of this caller.
    pub async fn geoblock(&self) -> Result<JurisdictionResponse, SdkError> {
        self.client
            .http
            .get(&self.url(), RetryPolicy::Idempotent)
            .await
    }

    /// The classification of the visitor described by `context`, for servers
    /// that relay requests on a visitor's behalf.
    #[cfg(not(target_arch = "wasm32"))]
    pub async fn geoblock_relayed(
        &self,
        context: &RelayContext,
    ) -> Result<Relayed<JurisdictionResponse>, SdkError> {
        self.client
            .http
            .get_relayed(&self.url(), RetryPolicy::Idempotent, context)
            .await
    }
}
