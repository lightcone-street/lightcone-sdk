//! Subscription types, tracking, and matching.

use crate::shared::{OrderBookId, PubkeyStr, Resolution};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Parameters for subscribing to a WS channel.
///
/// Wire format uses `#[serde(tag = "type")]` matching the backend's
/// `SubscriptionParams` enum — both subscribe and unsubscribe use `"type"`.
#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
#[serde(tag = "type")]
pub enum SubscribeParams {
    #[serde(rename = "book_update")]
    Books {
        orderbook_ids: Vec<OrderBookId>,
    },
    #[serde(rename = "trades")]
    Trades {
        orderbook_ids: Vec<OrderBookId>,
    },
    #[serde(rename = "user")]
    User {
        wallet_address: PubkeyStr,
    },
    #[serde(rename = "price_history")]
    PriceHistory {
        orderbook_id: OrderBookId,
        resolution: Resolution,
        #[serde(default)]
        include_ohlcv: bool,
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

/// Parameters for unsubscribing from a WS channel.
///
/// Uses `#[serde(tag = "type")]` — the backend uses the same `SubscriptionParams`
/// type for both subscribe and unsubscribe, discriminated by the outer `method` field.
#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
#[serde(tag = "type")]
pub enum UnsubscribeParams {
    #[serde(rename = "book_update")]
    Books {
        orderbook_ids: Vec<OrderBookId>,
    },
    #[serde(rename = "trades")]
    Trades {
        orderbook_ids: Vec<OrderBookId>,
    },
    #[serde(rename = "user")]
    User {
        wallet_address: PubkeyStr,
    },
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

/// Trait for subscription types that can be tracked and matched.
pub trait Subscription {
    fn to_subscribe_params(&self) -> SubscribeParams;
    fn to_unsubscribe_params(&self) -> UnsubscribeParams;
    fn matches_unsubscribe(&self, unsub: &UnsubscribeParams) -> bool;
    fn subscription_key(&self) -> String;
}

impl Subscription for SubscribeParams {
    fn to_subscribe_params(&self) -> SubscribeParams {
        self.clone()
    }

    fn to_unsubscribe_params(&self) -> UnsubscribeParams {
        match self {
            SubscribeParams::Books { orderbook_ids } => UnsubscribeParams::Books {
                orderbook_ids: orderbook_ids.clone(),
            },
            SubscribeParams::Trades { orderbook_ids } => UnsubscribeParams::Trades {
                orderbook_ids: orderbook_ids.clone(),
            },
            SubscribeParams::User { wallet_address } => UnsubscribeParams::User {
                wallet_address: wallet_address.clone(),
            },
            SubscribeParams::PriceHistory {
                orderbook_id,
                resolution,
                ..
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

    fn matches_unsubscribe(&self, unsub: &UnsubscribeParams) -> bool {
        match (self, unsub) {
            (
                SubscribeParams::Books {
                    orderbook_ids: sub_ids,
                },
                UnsubscribeParams::Books {
                    orderbook_ids: unsub_ids,
                },
            ) => {
                let sub_set: HashSet<_> = sub_ids.iter().collect();
                let unsub_set: HashSet<_> = unsub_ids.iter().collect();
                sub_set == unsub_set
            }
            (
                SubscribeParams::Trades {
                    orderbook_ids: sub_ids,
                },
                UnsubscribeParams::Trades {
                    orderbook_ids: unsub_ids,
                },
            ) => {
                let sub_set: HashSet<_> = sub_ids.iter().collect();
                let unsub_set: HashSet<_> = unsub_ids.iter().collect();
                sub_set == unsub_set
            }
            (
                SubscribeParams::User {
                    wallet_address: sub_addr,
                },
                UnsubscribeParams::User {
                    wallet_address: unsub_addr,
                },
            ) => sub_addr == unsub_addr,
            (
                SubscribeParams::PriceHistory {
                    orderbook_id: sub_id,
                    resolution: sub_res,
                    ..
                },
                UnsubscribeParams::PriceHistory {
                    orderbook_id: unsub_id,
                    resolution: unsub_res,
                },
            ) => sub_id == unsub_id && sub_res == unsub_res,
            (
                SubscribeParams::Ticker {
                    orderbook_ids: sub_ids,
                },
                UnsubscribeParams::Ticker {
                    orderbook_ids: unsub_ids,
                },
            ) => {
                let sub_set: HashSet<_> = sub_ids.iter().collect();
                let unsub_set: HashSet<_> = unsub_ids.iter().collect();
                sub_set == unsub_set
            }
            (
                SubscribeParams::Market {
                    market_pubkey: sub_pk,
                },
                UnsubscribeParams::Market {
                    market_pubkey: unsub_pk,
                },
            ) => sub_pk == unsub_pk,
            _ => false,
        }
    }

    fn subscription_key(&self) -> String {
        match self {
            SubscribeParams::Books { orderbook_ids } => {
                format!("book:{}", ids_key(orderbook_ids))
            }
            SubscribeParams::Trades { orderbook_ids } => {
                format!("trades:{}", ids_key(orderbook_ids))
            }
            SubscribeParams::User { wallet_address } => {
                format!("user:{}", wallet_address)
            }
            SubscribeParams::PriceHistory {
                orderbook_id,
                resolution,
                ..
            } => format!("price_history:{}:{}", orderbook_id, resolution),
            SubscribeParams::Ticker { orderbook_ids } => {
                format!("ticker:{}", ids_key(orderbook_ids))
            }
            SubscribeParams::Market { market_pubkey } => {
                format!("market:{}", market_pubkey)
            }
        }
    }
}

fn ids_key(ids: &[OrderBookId]) -> String {
    let mut sorted: Vec<_> = ids.iter().map(|id| id.to_string()).collect();
    sorted.sort();
    sorted.join(",")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_subscribe_params_serialization() {
        let params = SubscribeParams::Books {
            orderbook_ids: vec![OrderBookId::new("abc")],
        };
        let json = serde_json::to_string(&params).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["type"], "book_update");
        assert_eq!(parsed["orderbook_ids"][0], "abc");
    }

    #[test]
    fn test_unsubscribe_params_uses_type_tag() {
        let params = UnsubscribeParams::Books {
            orderbook_ids: vec![OrderBookId::new("abc")],
        };
        let json = serde_json::to_string(&params).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Must be "type", NOT "source" or "channel"
        assert_eq!(parsed["type"], "book_update");
        assert!(parsed.get("source").is_none());
        assert!(parsed.get("channel").is_none());
    }

    #[test]
    fn test_subscribe_user_with_wallet() {
        let params = SubscribeParams::User {
            wallet_address: PubkeyStr::new("wallet123"),
        };
        let json = serde_json::to_string(&params).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["type"], "user");
        assert_eq!(parsed["wallet_address"], "wallet123");
    }

    #[test]
    fn test_price_history_include_ohlcv_default() {
        let params = SubscribeParams::PriceHistory {
            orderbook_id: OrderBookId::new("abc"),
            resolution: Resolution::Minute1,
            include_ohlcv: false,
        };
        let json = serde_json::to_string(&params).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["type"], "price_history");
        assert_eq!(parsed["include_ohlcv"], false);
    }

    #[test]
    fn test_matches_unsubscribe_books_set_equality() {
        let sub = SubscribeParams::Books {
            orderbook_ids: vec![OrderBookId::new("a"), OrderBookId::new("b")],
        };
        let unsub_same = UnsubscribeParams::Books {
            orderbook_ids: vec![OrderBookId::new("b"), OrderBookId::new("a")],
        };
        let unsub_diff = UnsubscribeParams::Books {
            orderbook_ids: vec![OrderBookId::new("c")],
        };

        assert!(sub.matches_unsubscribe(&unsub_same));
        assert!(!sub.matches_unsubscribe(&unsub_diff));
    }

    #[test]
    fn test_matches_unsubscribe_cross_type_no_match() {
        let sub = SubscribeParams::Books {
            orderbook_ids: vec![OrderBookId::new("a")],
        };
        let unsub = UnsubscribeParams::Trades {
            orderbook_ids: vec![OrderBookId::new("a")],
        };

        assert!(!sub.matches_unsubscribe(&unsub));
    }

    #[test]
    fn test_subscription_key_deterministic() {
        let sub = SubscribeParams::Books {
            orderbook_ids: vec![OrderBookId::new("b"), OrderBookId::new("a")],
        };
        assert_eq!(sub.subscription_key(), "book:a,b");
    }

    #[test]
    fn test_to_unsubscribe_params_roundtrip() {
        let sub = SubscribeParams::User {
            wallet_address: PubkeyStr::new("wallet123"),
        };
        let unsub = sub.to_unsubscribe_params();
        assert!(sub.matches_unsubscribe(&unsub));
    }
}
