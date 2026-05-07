//! Notification domain — user notifications for events.

pub mod client;

use crate::domain::market::MarketResolutionResponse;
use crate::shared::PubkeyStr;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "notification_type", content = "data")]
#[serde(rename_all = "snake_case")]
pub enum NotificationKind {
    MarketResolved(MarketResolvedData),
    OrderFilled(OrderFilledData),
    NewMarket(MarketData),
    RulesClarified(MarketData),
    Global,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MarketResolvedData {
    pub market_pubkey: PubkeyStr,
    #[serde(default)]
    pub market_slug: Option<String>,
    #[serde(default)]
    pub market_name: Option<String>,
    #[serde(default)]
    pub resolution: Option<MarketResolutionResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OrderFilledData {
    pub order_hash: String,
    pub market_pubkey: PubkeyStr,
    pub side: String,
    pub price: Decimal,
    pub filled: Decimal,
    pub remaining: Decimal,
    #[serde(default)]
    pub market_slug: Option<String>,
    #[serde(default)]
    pub market_name: Option<String>,
    #[serde(default)]
    pub outcome_name: Option<String>,
    #[serde(default)]
    pub outcome_icon_url_low: Option<String>,
    #[serde(default)]
    pub outcome_icon_url_medium: Option<String>,
    #[serde(default)]
    pub outcome_icon_url_high: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MarketData {
    pub market_pubkey: PubkeyStr,
    #[serde(default)]
    pub market_slug: Option<String>,
    #[serde(default)]
    pub market_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Notification {
    pub id: String,
    #[serde(flatten)]
    pub kind: NotificationKind,
    pub title: String,
    pub message: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    pub created_at: String,
}

impl Notification {
    pub fn is_global(&self) -> bool {
        matches!(self.kind, NotificationKind::Global)
    }

    pub fn market_slug(&self) -> Option<&str> {
        match &self.kind {
            NotificationKind::MarketResolved(d) => d.market_slug.as_deref(),
            NotificationKind::OrderFilled(d) => d.market_slug.as_deref(),
            NotificationKind::NewMarket(d) | NotificationKind::RulesClarified(d) => {
                d.market_slug.as_deref()
            }
            NotificationKind::Global => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::market::MarketResolutionKind;

    #[test]
    fn market_resolved_notification_deserializes_resolution() {
        let notification: Notification = serde_json::from_str(
            r#"{
                "id": "notif_1",
                "notification_type": "market_resolved",
                "data": {
                    "market_pubkey": "market_1",
                    "market_slug": "test-market",
                    "market_name": "Test Market",
                    "resolution": {
                        "kind": "scalar",
                        "payout_denominator": 10,
                        "payouts": [
                            { "outcome_index": 0, "payout_numerator": 7 },
                            { "outcome_index": 1, "payout_numerator": 3 }
                        ],
                        "single_winning_outcome": null
                    }
                },
                "title": "Market resolved",
                "message": "The market has resolved.",
                "created_at": "2026-05-06T13:00:00Z"
            }"#,
        )
        .unwrap();

        match notification.kind {
            NotificationKind::MarketResolved(data) => {
                assert_eq!(data.market_pubkey.as_str(), "market_1");
                assert_eq!(data.market_slug.as_deref(), Some("test-market"));
                let resolution = data.resolution.unwrap();
                assert_eq!(resolution.kind, MarketResolutionKind::Scalar);
                assert_eq!(resolution.payout_denominator, 10);
                assert_eq!(resolution.single_winning_outcome, None);
                assert_eq!(resolution.payouts.len(), 2);
            }
            other => panic!("expected market_resolved notification, got {other:?}"),
        }
    }
}
