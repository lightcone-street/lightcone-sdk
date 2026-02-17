//! Subscription types and tracking.

use crate::shared::{OrderBookId, PubkeyStr, Resolution};
use serde::{Deserialize, Serialize};

/// Parameters for subscribing to a WS channel.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "channel")]
pub enum SubscribeParams {
    /// Subscribe to orderbook updates for one or more orderbook IDs.
    #[serde(rename = "book")]
    Books {
        orderbook_ids: Vec<OrderBookId>,
    },
    /// Subscribe to trade events for one or more orderbook IDs.
    #[serde(rename = "trades")]
    Trades {
        orderbook_ids: Vec<OrderBookId>,
    },
    /// Subscribe to user-specific events (orders, balances).
    #[serde(rename = "user")]
    User,
    /// Subscribe to price history for an orderbook at a given resolution.
    #[serde(rename = "price_history")]
    PriceHistory {
        orderbook_id: OrderBookId,
        resolution: Resolution,
    },
    /// Subscribe to ticker data for one or more orderbook IDs.
    #[serde(rename = "ticker")]
    Ticker {
        orderbook_ids: Vec<OrderBookId>,
    },
    /// Subscribe to market lifecycle events.
    #[serde(rename = "market")]
    Market {
        market_pubkey: PubkeyStr,
    },
}

/// Parameters for unsubscribing from a WS channel.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "channel")]
pub enum UnsubscribeParams {
    #[serde(rename = "book")]
    Books {
        orderbook_ids: Vec<OrderBookId>,
    },
    #[serde(rename = "trades")]
    Trades {
        orderbook_ids: Vec<OrderBookId>,
    },
    #[serde(rename = "user")]
    User,
    #[serde(rename = "price_history")]
    PriceHistory {
        orderbook_id: OrderBookId,
        resolution: Resolution,
    },
    #[serde(rename = "ticker")]
    Ticker {
        orderbook_ids: Vec<OrderBookId>,
    },
    #[serde(rename = "market")]
    Market {
        market_pubkey: PubkeyStr,
    },
}

/// Trait for types that represent an active subscription for tracking/matching.
pub trait Subscription {
    /// Unique key for deduplication.
    fn subscription_key(&self) -> String;

    /// Convert to the corresponding unsubscribe params.
    fn to_unsubscribe(&self) -> UnsubscribeParams;
}

impl Subscription for SubscribeParams {
    fn subscription_key(&self) -> String {
        match self {
            SubscribeParams::Books { orderbook_ids } => {
                format!("book:{}", ids_key(orderbook_ids))
            }
            SubscribeParams::Trades { orderbook_ids } => {
                format!("trades:{}", ids_key(orderbook_ids))
            }
            SubscribeParams::User => "user".to_string(),
            SubscribeParams::PriceHistory {
                orderbook_id,
                resolution,
            } => format!("price_history:{}:{}", orderbook_id, resolution),
            SubscribeParams::Ticker { orderbook_ids } => {
                format!("ticker:{}", ids_key(orderbook_ids))
            }
            SubscribeParams::Market { market_pubkey } => {
                format!("market:{}", market_pubkey)
            }
        }
    }

    fn to_unsubscribe(&self) -> UnsubscribeParams {
        match self {
            SubscribeParams::Books { orderbook_ids } => UnsubscribeParams::Books {
                orderbook_ids: orderbook_ids.clone(),
            },
            SubscribeParams::Trades { orderbook_ids } => UnsubscribeParams::Trades {
                orderbook_ids: orderbook_ids.clone(),
            },
            SubscribeParams::User => UnsubscribeParams::User,
            SubscribeParams::PriceHistory {
                orderbook_id,
                resolution,
            } => UnsubscribeParams::PriceHistory {
                orderbook_id: orderbook_id.clone(),
                resolution: *resolution,
            },
            SubscribeParams::Ticker { orderbook_ids } => UnsubscribeParams::Ticker {
                orderbook_ids: orderbook_ids.clone(),
            },
            SubscribeParams::Market { market_pubkey } => UnsubscribeParams::Market {
                market_pubkey: market_pubkey.clone(),
            },
        }
    }
}

fn ids_key(ids: &[OrderBookId]) -> String {
    let mut sorted: Vec<_> = ids.iter().map(|id| id.to_string()).collect();
    sorted.sort();
    sorted.join(",")
}
