//! Subscription types, tracking, and matching.

use crate::domain::orderbook::aggregation::BookAggregation;
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
    /// Book snapshots, optionally aggregated (Hyperliquid-style).
    ///
    /// Each `(orderbook, aggregation)` pair is a distinct subscription — one
    /// connection may hold multiple aggregation views of the same orderbook.
    /// The wire spelling is camelCase `nSigFigs` and both keys are omitted
    /// when absent (the backend rejects unknown/snake_case params).
    #[serde(rename = "book_update")]
    Books {
        orderbook_ids: Vec<OrderBookId>,
        #[serde(default, rename = "nSigFigs", skip_serializing_if = "Option::is_none")]
        n_sig_figs: Option<u32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        mantissa: Option<u32>,
    },
    #[serde(rename = "trades")]
    Trades { orderbook_ids: Vec<OrderBookId> },
    #[serde(rename = "user")]
    User { wallet_address: PubkeyStr },
    #[serde(rename = "price_history")]
    PriceHistory {
        orderbook_id: OrderBookId,
        resolution: Resolution,
        #[serde(default)]
        include_ohlcv: bool,
    },
    #[serde(rename = "ticker")]
    Ticker { orderbook_ids: Vec<OrderBookId> },
    #[serde(rename = "market")]
    Market { market_pubkey: PubkeyStr },
    #[serde(rename = "deposit_price")]
    DepositPrice {
        deposit_asset: PubkeyStr,
        resolution: Resolution,
    },
    /// Live spot price for a single deposit asset. Snapshot on subscribe +
    /// per-asset price ticks. Distinct from `deposit_price` which carries
    /// OHLCV candles per resolution.
    #[serde(rename = "deposit_asset_price")]
    DepositAssetPrice { deposit_asset: PubkeyStr },
}

/// Parameters for unsubscribing from a WS channel.
///
/// Uses `#[serde(tag = "type")]` — the backend uses the same `SubscriptionParams`
/// type for both subscribe and unsubscribe, discriminated by the outer `method` field.
#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
#[serde(tag = "type")]
pub enum UnsubscribeParams {
    /// Unsubscribe is matched server-side on `(orderbook_ids, normalized
    /// aggregation)` — repeat the same `n_sig_figs`/`mantissa` that were
    /// subscribed or nothing is removed.
    #[serde(rename = "book_update")]
    Books {
        orderbook_ids: Vec<OrderBookId>,
        #[serde(default, rename = "nSigFigs", skip_serializing_if = "Option::is_none")]
        n_sig_figs: Option<u32>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        mantissa: Option<u32>,
    },
    #[serde(rename = "trades")]
    Trades { orderbook_ids: Vec<OrderBookId> },
    #[serde(rename = "user")]
    User { wallet_address: PubkeyStr },
    #[serde(rename = "price_history")]
    PriceHistory {
        orderbook_id: OrderBookId,
        resolution: Resolution,
    },
    #[serde(rename = "ticker")]
    Ticker { orderbook_ids: Vec<OrderBookId> },
    #[serde(rename = "market")]
    Market { market_pubkey: PubkeyStr },
    #[serde(rename = "deposit_price")]
    DepositPrice {
        deposit_asset: PubkeyStr,
        resolution: Resolution,
    },
    #[serde(rename = "deposit_asset_price")]
    DepositAssetPrice { deposit_asset: PubkeyStr },
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
            SubscribeParams::Books {
                orderbook_ids,
                n_sig_figs,
                mantissa,
            } => UnsubscribeParams::Books {
                orderbook_ids: orderbook_ids.clone(),
                n_sig_figs: *n_sig_figs,
                mantissa: *mantissa,
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
            SubscribeParams::DepositPrice {
                deposit_asset,
                resolution,
            } => UnsubscribeParams::DepositPrice {
                deposit_asset: deposit_asset.clone(),
                resolution: *resolution,
            },
            SubscribeParams::DepositAssetPrice { deposit_asset } => {
                UnsubscribeParams::DepositAssetPrice {
                    deposit_asset: deposit_asset.clone(),
                }
            }
        }
    }

    fn matches_unsubscribe(&self, unsub: &UnsubscribeParams) -> bool {
        match (self, unsub) {
            (
                SubscribeParams::Books {
                    orderbook_ids: sub_ids,
                    n_sig_figs: sub_n_sig_figs,
                    mantissa: sub_mantissa,
                },
                UnsubscribeParams::Books {
                    orderbook_ids: unsub_ids,
                    n_sig_figs: unsub_n_sig_figs,
                    mantissa: unsub_mantissa,
                },
            ) => {
                let sub_set: HashSet<_> = sub_ids.iter().collect();
                let unsub_set: HashSet<_> = unsub_ids.iter().collect();
                let sub_aggregation = BookAggregation::from_frame(*sub_n_sig_figs, *sub_mantissa);
                let unsub_aggregation =
                    BookAggregation::from_frame(*unsub_n_sig_figs, *unsub_mantissa);
                sub_set == unsub_set && sub_aggregation == unsub_aggregation
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
            (
                SubscribeParams::DepositPrice {
                    deposit_asset: sub_asset,
                    resolution: sub_res,
                },
                UnsubscribeParams::DepositPrice {
                    deposit_asset: unsub_asset,
                    resolution: unsub_res,
                },
            ) => sub_asset == unsub_asset && sub_res == unsub_res,
            (
                SubscribeParams::DepositAssetPrice {
                    deposit_asset: sub_asset,
                },
                UnsubscribeParams::DepositAssetPrice {
                    deposit_asset: unsub_asset,
                },
            ) => sub_asset == unsub_asset,
            _ => false,
        }
    }

    fn subscription_key(&self) -> String {
        match self {
            SubscribeParams::Books {
                orderbook_ids,
                n_sig_figs,
                mantissa,
            } => {
                let aggregation = BookAggregation::from_frame(*n_sig_figs, *mantissa);
                if aggregation.is_full() {
                    // Unchanged from the pre-aggregation key so existing
                    // consumers' tracked subscriptions stay stable.
                    format!("book:{}", ids_key(orderbook_ids))
                } else {
                    format!(
                        "book:{}:{}",
                        ids_key(orderbook_ids),
                        aggregation.key_suffix()
                    )
                }
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
            SubscribeParams::DepositPrice {
                deposit_asset,
                resolution,
            } => format!("deposit_price:{}:{}", deposit_asset, resolution),
            SubscribeParams::DepositAssetPrice { deposit_asset } => {
                format!("deposit_asset_price:{}", deposit_asset)
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
            n_sig_figs: None,
            mantissa: None,
        };
        let json = serde_json::to_string(&params).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["type"], "book_update");
        assert_eq!(parsed["orderbook_ids"][0], "abc");
        // Full precision must omit the aggregation keys entirely — the
        // backend rejects unknown fields and nulls.
        assert!(parsed.get("nSigFigs").is_none());
        assert!(parsed.get("n_sig_figs").is_none());
        assert!(parsed.get("mantissa").is_none());
    }

    #[test]
    fn test_subscribe_params_aggregation_serializes_camel_case() {
        let params = SubscribeParams::Books {
            orderbook_ids: vec![OrderBookId::new("abc")],
            n_sig_figs: Some(5),
            mantissa: Some(2),
        };
        let json = serde_json::to_string(&params).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["nSigFigs"], 5);
        assert_eq!(parsed["mantissa"], 2);
        assert!(parsed.get("n_sig_figs").is_none());
    }

    #[test]
    fn test_unsubscribe_params_uses_type_tag() {
        let params = UnsubscribeParams::Books {
            orderbook_ids: vec![OrderBookId::new("abc")],
            n_sig_figs: None,
            mantissa: None,
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
    fn test_deposit_price_serialization() {
        let params = SubscribeParams::DepositPrice {
            deposit_asset: PubkeyStr::new("mint123"),
            resolution: Resolution::Hour1,
        };
        let json = serde_json::to_string(&params).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed["type"], "deposit_price");
        assert_eq!(parsed["deposit_asset"], "mint123");
        assert_eq!(parsed["resolution"], "1h");
    }

    #[test]
    fn test_matches_unsubscribe_books_set_equality() {
        let sub = SubscribeParams::Books {
            orderbook_ids: vec![OrderBookId::new("a"), OrderBookId::new("b")],
            n_sig_figs: None,
            mantissa: None,
        };
        let unsub_same = UnsubscribeParams::Books {
            orderbook_ids: vec![OrderBookId::new("b"), OrderBookId::new("a")],
            n_sig_figs: None,
            mantissa: None,
        };
        let unsub_diff = UnsubscribeParams::Books {
            orderbook_ids: vec![OrderBookId::new("c")],
            n_sig_figs: None,
            mantissa: None,
        };

        assert!(sub.matches_unsubscribe(&unsub_same));
        assert!(!sub.matches_unsubscribe(&unsub_diff));
    }

    #[test]
    fn test_matches_unsubscribe_books_compares_normalized_aggregation() {
        let sub = SubscribeParams::Books {
            orderbook_ids: vec![OrderBookId::new("a")],
            n_sig_figs: Some(5),
            mantissa: None,
        };
        let unsub_normalized = UnsubscribeParams::Books {
            orderbook_ids: vec![OrderBookId::new("a")],
            n_sig_figs: Some(5),
            mantissa: Some(1),
        };
        let unsub_full = UnsubscribeParams::Books {
            orderbook_ids: vec![OrderBookId::new("a")],
            n_sig_figs: None,
            mantissa: None,
        };
        let unsub_other_grouping = UnsubscribeParams::Books {
            orderbook_ids: vec![OrderBookId::new("a")],
            n_sig_figs: Some(5),
            mantissa: Some(2),
        };

        // (5, none) and (5, 1) are the same subscription.
        assert!(sub.matches_unsubscribe(&unsub_normalized));
        // A grouped subscription is never matched by a full-precision or
        // differently grouped unsubscribe.
        assert!(!sub.matches_unsubscribe(&unsub_full));
        assert!(!sub.matches_unsubscribe(&unsub_other_grouping));
    }

    #[test]
    fn test_matches_unsubscribe_cross_type_no_match() {
        let sub = SubscribeParams::Books {
            orderbook_ids: vec![OrderBookId::new("a")],
            n_sig_figs: None,
            mantissa: None,
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
            n_sig_figs: None,
            mantissa: None,
        };
        // Full precision keeps the pre-aggregation key shape.
        assert_eq!(sub.subscription_key(), "book:a,b");
    }

    #[test]
    fn test_subscription_key_distinguishes_aggregations() {
        let grouped = SubscribeParams::Books {
            orderbook_ids: vec![OrderBookId::new("a")],
            n_sig_figs: Some(5),
            mantissa: Some(2),
        };
        assert_eq!(grouped.subscription_key(), "book:a:sig5m2");

        let normalized = SubscribeParams::Books {
            orderbook_ids: vec![OrderBookId::new("a")],
            n_sig_figs: Some(5),
            mantissa: None,
        };
        let explicit = SubscribeParams::Books {
            orderbook_ids: vec![OrderBookId::new("a")],
            n_sig_figs: Some(5),
            mantissa: Some(1),
        };
        assert_eq!(
            normalized.subscription_key(),
            explicit.subscription_key(),
            "(5, none) and (5, 1) must produce the same key"
        );
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
