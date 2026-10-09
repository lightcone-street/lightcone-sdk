//! Wire types for committed custody funding (REST positions + WS `user`).
//!
//! The backend reads balances from the committed trading engine: every
//! custody token account a wallet can trade from is a *funding account*
//! (a global deposit account, or a conditional-token account of one market
//! position). Amount fields are decimal strings in the account mint's own
//! units unless the field name ends in `_atoms`.

use crate::shared::{serde_util, DecimalText, PubkeyStr};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Which custody account backs an order or holds a balance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FundingSource {
    /// The wallet's global deposit account for a deposit mint.
    Global,
    /// A conditional-token account of one market position.
    Conditional,
    /// A source this SDK version does not know.
    #[serde(other)]
    Unknown,
}

/// Evidence state of the last custody observation for a funding account.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ObservationState {
    /// The token account exists and `raw_observed` holds its balance.
    Present,
    /// The token account was observed not to exist.
    Absent,
    /// The account exists but is not a valid token account for this mint.
    Invalid,
    /// No coherent observation has been accepted yet.
    Unobserved,
    /// A state this SDK version does not know.
    #[serde(other)]
    Unknown,
}

/// One recorded funding account (REST positions, REST user orders, and the WS
/// `user` snapshot).
///
/// These are the committed quantities at the response's database snapshot,
/// not live spending capacity: the engine alone decides whether an order can
/// be funded.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FundingAccount {
    /// Custody token account.
    pub account: PubkeyStr,
    /// Token mint held by `account` (deposit mint or conditional mint).
    pub mint: PubkeyStr,
    pub source: FundingSource,
    /// Market of a conditional account. The backend sends `""` for global
    /// accounts, which reads as `None`.
    #[serde(default, deserialize_with = "serde_util::deserialize_nonempty_string")]
    pub market_pubkey: Option<PubkeyStr>,
    /// Deposit mint that collateralizes `mint`.
    pub deposit_mint: PubkeyStr,
    /// Last observed token balance in `mint` units; `None` when unobserved.
    #[serde(default)]
    pub raw_observed: Option<Decimal>,
    /// Amount reserved by open orders, in `mint` units (u128 aggregate).
    pub order_reserved: DecimalText,
    /// Amount reserved by in-flight executions, in `mint` units (u128 aggregate).
    pub execution_reserved: DecimalText,
    /// Recorded balance minus reservations, in raw signed atoms of `mint`.
    /// Can be negative and is not spendable capacity; `None` when unobserved.
    #[serde(default, with = "serde_util::opt_i128_text")]
    pub signed_remaining_atoms: Option<i128>,
    /// Recorded observation boundary (opaque).
    #[serde(default)]
    pub accepted_boundary: String,
    /// Solana slot of the last accepted observation.
    #[serde(default, with = "serde_util::opt_u64_text")]
    pub observed_slot: Option<u64>,
    /// Blockhash of the last accepted observation (empty when unobserved).
    #[serde(default)]
    pub observed_blockhash: String,
    pub observation_state: ObservationState,
}

/// Live funding-account update on the WS `user` channel (`event_type: "funding"`).
///
/// Unlike [`FundingAccount`] this carries the engine's transient readiness
/// view: `available` and `ready` describe spending capacity at publication
/// time and are forced to zero/`false` when the fact is not actionable.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FundingUpdate {
    #[serde(flatten)]
    pub commit: crate::domain::order::wire::CommitInfo,
    pub user_pubkey: PubkeyStr,
    pub account: PubkeyStr,
    pub mint: PubkeyStr,
    pub source: FundingSource,
    /// Market of a conditional account; `None` for global accounts.
    #[serde(default, deserialize_with = "serde_util::deserialize_nonempty_string")]
    pub market_pubkey: Option<PubkeyStr>,
    /// Last observed token balance in `mint` units.
    #[serde(default)]
    pub raw_observed: Option<Decimal>,
    /// Observed balance usable for trading, in `mint` units.
    #[serde(default)]
    pub usable_observed: Option<Decimal>,
    /// Amount reserved by open orders, in `mint` units.
    pub order_reserved: DecimalText,
    /// Amount reserved by in-flight executions, in `mint` units.
    pub execution_reserved: DecimalText,
    /// Usable balance minus reservations, in `mint` units; may be negative.
    #[serde(default)]
    pub signed_remaining: Option<DecimalText>,
    /// Spendable amount for new orders, in `mint` units.
    pub available: Decimal,
    /// Whether the engine will fund new orders from this account.
    pub ready: bool,
    #[serde(default)]
    pub accepted_boundary: Option<String>,
    /// Solana slot of the observation.
    #[serde(default, with = "serde_util::opt_u64_text")]
    pub slot: Option<u64>,
    #[serde(default)]
    pub blockhash: Option<String>,
}

/// Response for all four positions routes: `GET /api/users/positions`,
/// `/api/users/{user}/positions`, `/api/users/markets/{market}/positions` and
/// `/api/users/{user}/markets/{market}/positions`.
///
/// Market-scoped routes keep global accounts and the conditional accounts of
/// that market. Pages are filtered after a bounded scan, so a page can be
/// empty while `has_more` is true.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PositionsResponse {
    pub owner: PubkeyStr,
    /// Market filter of a market-scoped route; `None` for wallet-wide routes.
    #[serde(default)]
    pub market_pubkey: Option<PubkeyStr>,
    pub funding_accounts: Vec<FundingAccount>,
    /// Database revision of this page; later pages may be newer.
    #[serde(with = "serde_util::u64_text")]
    pub committed_revision: u64,
    #[serde(with = "serde_util::u64_text")]
    pub projection_generation: u64,
    pub has_more: bool,
    /// Funding-account cursor for the next page (authenticated routes only;
    /// the public `{user}` routes reject every query string).
    #[serde(default)]
    pub next_cursor: Option<String>,
}

/// Market-scoped positions share the wallet-wide response shape.
pub type MarketPositionsResponse = PositionsResponse;

#[cfg(test)]
mod tests {
    use super::*;

    /// Shape emitted by `trading_wire::funding_account`, including the
    /// backend's u128 maximum aggregate fixture.
    fn conditional_account() -> serde_json::Value {
        serde_json::json!({
            "account": "Acct1111111111111111111111111111111111111111",
            "mint": "Cond1111111111111111111111111111111111111111",
            "source": "conditional",
            "market_pubkey": "A9Bxkkc4nah517EjgjwSafwspGnmU9s1Ei5PTo5ZkJd9",
            "deposit_mint": "7SrxsoXjNR7Y8T3koJCt1yV4FrNUumoAUrJExDt6tQez",
            "raw_observed": "0.000100",
            "order_reserved": "340282366920938463463374607431768.211455",
            "execution_reserved": "0.000040",
            "signed_remaining_atoms": "-184467440737095516100",
            "accepted_boundary": "boundary",
            "observed_slot": "7",
            "observed_blockhash": "hash",
            "observation_state": "present"
        })
    }

    #[test]
    fn positions_page_decodes_backend_shape() {
        let global = serde_json::json!({
            "account": "Glob1111111111111111111111111111111111111111",
            "mint": "7SrxsoXjNR7Y8T3koJCt1yV4FrNUumoAUrJExDt6tQez",
            "source": "global",
            "market_pubkey": "",
            "deposit_mint": "7SrxsoXjNR7Y8T3koJCt1yV4FrNUumoAUrJExDt6tQez",
            "raw_observed": null,
            "order_reserved": "0",
            "execution_reserved": "0",
            "signed_remaining_atoms": null,
            "accepted_boundary": "",
            "observed_slot": null,
            "observed_blockhash": "",
            "observation_state": "unobserved"
        });
        let response: PositionsResponse = serde_json::from_value(serde_json::json!({
            "owner": "Wallet1111111111111111111111111111111111111",
            "market_pubkey": null,
            "funding_accounts": [conditional_account(), global],
            "committed_revision": "275",
            "projection_generation": "1",
            "has_more": true,
            "next_cursor": "Glob1111111111111111111111111111111111111111"
        }))
        .unwrap();

        assert_eq!(response.committed_revision, 275);
        assert_eq!(response.projection_generation, 1);
        assert!(response.has_more);
        let conditional = &response.funding_accounts[0];
        assert_eq!(conditional.source, FundingSource::Conditional);
        assert_eq!(conditional.raw_observed, Some(Decimal::new(100, 6)));
        assert_eq!(conditional.order_reserved.to_decimal(), None);
        assert_eq!(
            conditional.execution_reserved.to_decimal(),
            Some(Decimal::new(40, 6))
        );
        assert_eq!(
            conditional.signed_remaining_atoms,
            Some(-184_467_440_737_095_516_100)
        );
        assert_eq!(conditional.observed_slot, Some(7));
        assert_eq!(conditional.observation_state, ObservationState::Present);

        let global = &response.funding_accounts[1];
        assert_eq!(global.source, FundingSource::Global);
        assert_eq!(global.market_pubkey, None);
        assert_eq!(global.raw_observed, None);
        assert_eq!(global.signed_remaining_atoms, None);
        assert_eq!(global.observation_state, ObservationState::Unobserved);
    }

    #[test]
    fn market_scoped_page_keeps_the_market_filter() {
        let response: MarketPositionsResponse = serde_json::from_value(serde_json::json!({
            "owner": "Wallet1111111111111111111111111111111111111",
            "market_pubkey": "A9Bxkkc4nah517EjgjwSafwspGnmU9s1Ei5PTo5ZkJd9",
            "funding_accounts": [],
            "committed_revision": "9",
            "projection_generation": "2",
            "has_more": true,
            "next_cursor": null
        }))
        .unwrap();
        assert_eq!(
            response.market_pubkey.as_ref().map(PubkeyStr::as_str),
            Some("A9Bxkkc4nah517EjgjwSafwspGnmU9s1Ei5PTo5ZkJd9")
        );
        assert!(response.funding_accounts.is_empty() && response.has_more);
    }

    #[test]
    fn unknown_observation_state_does_not_fail_the_page() {
        let mut account = conditional_account();
        account["observation_state"] = serde_json::json!("quarantined");
        let account: FundingAccount = serde_json::from_value(account).unwrap();
        assert_eq!(account.observation_state, ObservationState::Unknown);
    }
}
