//! Deserialization of order mutation and query responses.

use super::client::{
    CancelAllSuccess, CancelStatus, CancelSuccess, SubmitOrderResponse, SubmitOrderStatus,
    UserOrdersResponse,
};
use crate::domain::position::wire::FundingSource;
use crate::error::SdkError;
use crate::shared::{ApiResponse, DecimalText, RejectionCode};
use rust_decimal::Decimal;
use serde_json::{json, Value};

const HASH: &str = "4f1a4b1ab1c0c0ffee0000000000000000000000000000000000000000000001";

/// Unwrap a success envelope the way the HTTP client does.
fn body<T: serde::de::DeserializeOwned>(envelope: Value) -> Result<T, SdkError> {
    match serde_json::from_value::<ApiResponse<T>>(envelope)? {
        ApiResponse::Success { body } => Ok(body),
        ApiResponse::Rejected { details } => Err(SdkError::ApiRejected(details)),
    }
}

#[test]
fn submit_response_without_committed_view_is_accepted_pending() {
    let response: SubmitOrderResponse = body(json!({
        "status": "success",
        "body": {
            "order_hash": HASH,
            "status": "accepted_pending",
            "state": null,
            "initial_cohort": null,
            "fills": [],
            "fills_complete": false,
            "fills_next_cursor": null
        }
    }))
    .unwrap();
    assert_eq!(response.status, SubmitOrderStatus::AcceptedPending);
    assert!(response.state.is_none() && response.initial_cohort.is_none());
    assert_eq!(response.filled_base(), None);
}

#[test]
fn submit_statuses_decode() {
    for (wire, status) in [
        ("filled", SubmitOrderStatus::Filled),
        ("accepted", SubmitOrderStatus::Accepted),
        ("accepted_pending", SubmitOrderStatus::AcceptedPending),
    ] {
        let decoded: SubmitOrderStatus = serde_json::from_value(json!(wire)).unwrap();
        assert_eq!(decoded, status);
    }
    // Retired or future statuses must not hide the accepted order hash.
    assert_eq!(
        serde_json::from_value::<SubmitOrderStatus>(json!("partial_fill")).unwrap(),
        SubmitOrderStatus::Unknown
    );
    let response: SubmitOrderResponse = body(json!({
        "status": "success",
        "body": {
            "order_hash": HASH,
            "status": "accepted_partial",
            "state": null,
            "initial_cohort": null,
            "fills": [],
            "fills_complete": true,
            "fills_next_cursor": null
        }
    }))
    .unwrap();
    assert_eq!(response.order_hash, HASH);
    assert_eq!(response.status, SubmitOrderStatus::Unknown);
}

#[test]
fn submit_business_rejection_surfaces_the_code() {
    let error = body::<SubmitOrderResponse>(json!({
        "status": "error",
        "error_details": {
            "reason": "Requested funding cannot cover the complete order",
            "rejection_code": "INSUFFICIENT_BALANCE",
            "error_log_id": "LCERR_1"
        }
    }))
    .unwrap_err();
    let SdkError::ApiRejected(details) = error else {
        panic!("expected rejection");
    };
    assert_eq!(
        details.rejection_code,
        Some(RejectionCode::InsufficientBalance)
    );
}

#[test]
fn cancel_response_decodes_raw_base_atoms() {
    let response: CancelSuccess = body(json!({
        "status": "success",
        "body": {
            "status": "cancelled",
            "order_hash": HASH,
            "quantities": {
                "newly_cancelled_base": "18446744073709551615",
                "confirmed_base": "0",
                "pending_base": "40",
                "remaining_open_base": "0"
            },
            "quantity_unit": "base_atoms",
            "revision": "812",
            "closed_reason": ""
        }
    }))
    .unwrap();
    assert_eq!(response.status, CancelStatus::Cancelled);
    let quantities = response.quantities.unwrap();
    assert_eq!(quantities.newly_cancelled_base, u64::MAX);
    assert_eq!(quantities.pending_base, 40);
    assert_eq!(response.quantity_unit, "base_atoms");
    assert_eq!(response.revision, 812);
    assert_eq!(response.closed_reason, None);

    let already: CancelSuccess = body(json!({
        "status": "success",
        "body": {
            "status": "already_closed",
            "order_hash": HASH,
            "quantities": null,
            "quantity_unit": "base_atoms",
            "revision": "813",
            "closed_reason": "expired"
        }
    }))
    .unwrap();
    assert_eq!(already.status, CancelStatus::AlreadyClosed);
    assert_eq!(already.quantities, None);
    assert_eq!(already.closed_reason.as_deref(), Some("expired"));

    assert_eq!(
        serde_json::from_value::<CancelStatus>(json!("superseded")).unwrap(),
        CancelStatus::Unknown
    );
}

#[test]
fn cancel_unknown_hash_is_order_not_found() {
    let error = body::<CancelSuccess>(json!({
        "status": "error",
        "error_details": {
            "reason": "Order has not been accepted",
            "rejection_code": "ORDER_NOT_FOUND"
        }
    }))
    .unwrap_err();
    let SdkError::ApiRejected(details) = error else {
        panic!("expected rejection");
    };
    assert_eq!(details.rejection_code, Some(RejectionCode::OrderNotFound));
}

#[test]
fn cancel_all_response_decodes_closure() {
    let response: CancelAllSuccess = body(json!({
        "status": "success",
        "body": {
            "status": "success",
            "user_pubkey": "Wallet1111111111111111111111111111111111111",
            "orderbook_id": "j749bQAbDsBAiyDs2Tj868heQj1b5KVp98ZrjiZd56a",
            "closure": {
                "operation_id": "8c1d1c5e-2b8f-4bb1-9f53-3c3f5f0a1b2c",
                "committed_revision": "900",
                "scope": "WalletBook:Wallet1111111111111111111111111111111111111:j749bQAbDsBAiyDs2Tj868heQj1b5KVp98ZrjiZd56a",
                "accepted_seq_cutoff": "44",
                "cleanup_pending": true
            },
            "message": "Accepted-order cutoff committed"
        }
    }))
    .unwrap();
    let closure = response.closure.unwrap();
    assert_eq!(closure.committed_revision, 900);
    assert_eq!(closure.accepted_seq_cutoff, Some(44));
    assert!(closure.cleanup_pending);
    assert!(closure.scope.starts_with("WalletBook:"));

    let wallet_wide: CancelAllSuccess = body(json!({
        "status": "success",
        "body": {
            "status": "success",
            "user_pubkey": "Wallet1111111111111111111111111111111111111",
            "orderbook_id": "",
            "closure": {
                "operation_id": "8c1d1c5e-2b8f-4bb1-9f53-3c3f5f0a1b2d",
                "committed_revision": "901",
                "scope": "Wallet:Wallet1111111111111111111111111111111111111",
                "accepted_seq_cutoff": null,
                "cleanup_pending": false
            },
            "message": "Accepted-order cutoff committed"
        }
    }))
    .unwrap();
    assert_eq!(wallet_wide.orderbook_id.as_str(), "");
    assert_eq!(wallet_wide.closure.unwrap().accepted_seq_cutoff, None);
}

#[test]
fn user_orders_response_decodes_orders_and_funding_page() {
    let response: UserOrdersResponse = body(json!({
        "status": "success",
        "body": {
            "user_pubkey": "Wallet1111111111111111111111111111111111111",
            "orders": [{
                "order_hash": HASH,
                "market_pubkey": "A9Bxkkc4nah517EjgjwSafwspGnmU9s1Ei5PTo5ZkJd9",
                "orderbook_id": "j749bQAbDsBAiyDs2Tj868heQj1b5KVp98ZrjiZd56a",
                "side": "ask",
                "amount_in": "4.00000000",
                "amount_out": "2.200000",
                "price": "0.5500",
                "created_at": 1790685521784_i64,
                "expiration": 0,
                "base_mint": "BaseMint111111111111111111111111111111111111",
                "quote_mint": "QuoteMint11111111111111111111111111111111111",
                "outcome_index": -1,
                "state": {
                    "original_base": "4.00000000",
                    "confirmed_base": "0.00000000",
                    "pending_base": "0.00000000",
                    "open_base": "4.00000000",
                    "cancelled_base": "0.00000000",
                    "closed_reason": null,
                    "committed_revision": "812",
                    "accepted_seq": "45"
                },
                "tif": "GTC",
                "source": "conditional",
                "order_type": "limit"
            }],
            "funding_accounts": [{
                "account": "Acct1111111111111111111111111111111111111111",
                "mint": "Cond1111111111111111111111111111111111111111",
                "source": "conditional",
                "market_pubkey": "A9Bxkkc4nah517EjgjwSafwspGnmU9s1Ei5PTo5ZkJd9",
                "deposit_mint": "7SrxsoXjNR7Y8T3koJCt1yV4FrNUumoAUrJExDt6tQez",
                "raw_observed": "4.00000000",
                "order_reserved": "340282366920938463463374607431768.211455",
                "execution_reserved": "0.00000000",
                "signed_remaining_atoms": "0",
                "accepted_boundary": "boundary",
                "observed_slot": "123",
                "observed_blockhash": "hash",
                "observation_state": "present"
            }],
            "next_cursor": null,
            "has_more": false,
            "next_funding_cursor": null,
            "funding_has_more": false,
            "committed_revision": "812",
            "projection_generation": "1"
        }
    }))
    .unwrap();
    assert_eq!(response.committed_revision, 812);
    let order = response.orders[0].clone().into_limit_order().unwrap();
    assert_eq!(order.remaining_size, Decimal::from(4));
    assert_eq!(order.accepted_seq, 45);
    assert_eq!(order.funding_source, FundingSource::Conditional);
    let account = &response.funding_accounts[0];
    assert_eq!(
        account.order_reserved,
        "340282366920938463463374607431768.211455"
            .parse::<DecimalText>()
            .unwrap()
    );
    assert_eq!(account.observed_slot, Some(123));
}

#[test]
fn empty_user_orders_page_may_still_have_more() {
    let response: UserOrdersResponse = body(json!({
        "status": "success",
        "body": {
            "user_pubkey": "Wallet1111111111111111111111111111111111111",
            "orders": [],
            "funding_accounts": [],
            "next_cursor": "45:4f1a",
            "has_more": true,
            "next_funding_cursor": null,
            "funding_has_more": false,
            "committed_revision": "812",
            "projection_generation": "1"
        }
    }))
    .unwrap();
    assert!(response.orders.is_empty() && response.has_more);
    assert_eq!(response.next_cursor.as_deref(), Some("45:4f1a"));
}
