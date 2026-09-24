//! Wire types for `GET /api/geoblock`.

use serde::{Deserialize, Serialize};

/// What a caller may do from a given jurisdiction tier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JurisdictionCapabilities {
    pub mode: String,
    pub can_authenticate: bool,
    pub can_mutate_account: bool,
    pub can_submit_orders: bool,
    pub can_cancel_orders: bool,
}

/// The backend's jurisdiction answer for one request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JurisdictionResponse {
    /// The effective visitor country, or `None` when the backend runs with
    /// edge classification disabled (local development).
    pub country: Option<String>,
    pub geoblocked: bool,
    /// `tier_1`, `tier_2`, or `tier_3`; `None` when unrestricted.
    pub tier: Option<String>,
    pub policy_version: String,
    /// Capabilities of the official frontend.
    pub frontend: JurisdictionCapabilities,
    /// Capabilities of direct API callers.
    pub api: JurisdictionCapabilities,
    /// True when Cloudflare accepted a relayed visitor country for this request.
    pub relayed: bool,
}

#[cfg(test)]
mod tests {
    use super::JurisdictionResponse;

    #[test]
    fn missing_relay_stamp_is_not_silently_classified_as_direct() {
        let response = serde_json::json!({
            "country": "US",
            "geoblocked": false,
            "tier": "tier_3",
            "policy_version": "test",
            "frontend": {"mode": "full", "can_authenticate": true, "can_mutate_account": true, "can_submit_orders": true, "can_cancel_orders": true},
            "api": {"mode": "full", "can_authenticate": true, "can_mutate_account": true, "can_submit_orders": true, "can_cancel_orders": true}
        });
        assert!(serde_json::from_value::<JurisdictionResponse>(response).is_err());
    }
}
