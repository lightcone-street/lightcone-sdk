//! Orders sub-client — submit, cancel, query.

use crate::client::LightconeClient;
use crate::error::SdkError;

/// Sub-client for order operations.
pub struct Orders<'a> {
    pub(crate) client: &'a LightconeClient,
}

impl<'a> Orders<'a> {
    pub async fn submit(
        &self,
        request: &impl serde::Serialize,
    ) -> Result<serde_json::Value, SdkError> {
        Ok(self.client.http.submit_order(request).await?)
    }

    pub async fn cancel(
        &self,
        request: &impl serde::Serialize,
    ) -> Result<serde_json::Value, SdkError> {
        Ok(self.client.http.cancel_order(request).await?)
    }

    pub async fn cancel_all(
        &self,
        request: &impl serde::Serialize,
    ) -> Result<serde_json::Value, SdkError> {
        Ok(self.client.http.cancel_all_orders(request).await?)
    }

    pub async fn get_user_orders(
        &self,
        request: &impl serde::Serialize,
    ) -> Result<serde_json::Value, SdkError> {
        Ok(self.client.http.get_user_orders(request).await?)
    }
}
