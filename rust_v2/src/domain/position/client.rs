//! Positions sub-client — portfolio & position queries.

use crate::client::LightconeClient;
use crate::domain::position::wire::PositionsResponse;
use crate::error::SdkError;

/// Sub-client for position operations.
pub struct Positions<'a> {
    pub(crate) client: &'a LightconeClient,
}

impl<'a> Positions<'a> {
    /// Get all positions for a user.
    pub async fn get(&self, user_pubkey: &str) -> Result<PositionsResponse, SdkError> {
        Ok(self.client.http.get_user_positions(user_pubkey).await?)
    }

    /// Get positions for a user in a specific market.
    pub async fn get_for_market(
        &self,
        user_pubkey: &str,
        market_pubkey: &str,
    ) -> Result<PositionsResponse, SdkError> {
        Ok(self
            .client
            .http
            .get_user_market_positions(user_pubkey, market_pubkey)
            .await?)
    }
}
