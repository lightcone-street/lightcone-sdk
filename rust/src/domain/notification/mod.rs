//! Notification domain — user notifications for events.

pub mod client;

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
    pub winning_outcome: Option<i16>,
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
