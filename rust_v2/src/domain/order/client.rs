//! Orders sub-client — submit, cancel, query.

use crate::client::LightconeClient;
use crate::error::SdkError;
use crate::http::RetryPolicy;

pub struct Orders<'a> {
    pub(crate) client: &'a LightconeClient,
}

impl<'a> Orders<'a> {
    pub async fn submit(
        &self,
        request: &impl serde::Serialize,
    ) -> Result<serde_json::Value, SdkError> {
        let url = format!("{}/api/orders/submit", self.client.http.base_url());
        Ok(self.client.http.post(&url, request, RetryPolicy::None).await?)
    }

    pub async fn cancel(
        &self,
        request: &impl serde::Serialize,
    ) -> Result<serde_json::Value, SdkError> {
        let url = format!("{}/api/orders/cancel", self.client.http.base_url());
        Ok(self.client.http.post(&url, request, RetryPolicy::None).await?)
    }

    pub async fn cancel_all(
        &self,
        request: &impl serde::Serialize,
    ) -> Result<serde_json::Value, SdkError> {
        let url = format!("{}/api/orders/cancel-all", self.client.http.base_url());
        Ok(self.client.http.post(&url, request, RetryPolicy::None).await?)
    }

    pub async fn get_user_orders(
        &self,
        request: &impl serde::Serialize,
    ) -> Result<serde_json::Value, SdkError> {
        let url = format!("{}/api/users/orders", self.client.http.base_url());
        Ok(self
            .client
            .http
            .post(&url, request, RetryPolicy::Idempotent)
            .await?)
    }
}
