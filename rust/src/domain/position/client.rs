//! Positions sub-client — portfolio & position queries.

use crate::client::LightconeClient;
use crate::domain::position::wire::{MarketPositionsResponse, PositionsResponse};
use crate::error::SdkError;
use crate::http::RetryPolicy;

pub struct Positions<'a> {
    pub(crate) client: &'a LightconeClient,
}

impl<'a> Positions<'a> {
    /// Get all positions for a user across all markets.
    pub async fn get(&self, user_pubkey: &str) -> Result<PositionsResponse, SdkError> {
        let url = format!(
            "{}/api/users/{}/positions",
            self.client.http.base_url(),
            user_pubkey
        );
        Ok(self
            .client
            .http
            .get(&url, RetryPolicy::Idempotent)
            .await?)
    }

    /// Get positions for a user in a specific market.
    pub async fn get_for_market(
        &self,
        user_pubkey: &str,
        market_pubkey: &str,
    ) -> Result<MarketPositionsResponse, SdkError> {
        let url = format!(
            "{}/api/users/{}/markets/{}/positions",
            self.client.http.base_url(),
            user_pubkey,
            market_pubkey
        );
        Ok(self
            .client
            .http
            .get(&url, RetryPolicy::Idempotent)
            .await?)
    }
}
