//! Privy sub-client — sign transactions, sign orders, export wallets.

use crate::client::LightconeClient;
use crate::error::SdkError;
use crate::http::RetryPolicy;

use super::builder::PrivyOrderKind;
use super::{
    CancelTarget, ExportWalletRequest, ExportWalletResponse, PrivyOrderBuilder,
    PrivyOrderEnvelope, SignAndCancelAllRequest, SignAndCancelOrderRequest,
    SignAndSendOrderRequest, SignAndSendTxRequest, SignAndSendTxResponse,
};

/// Sub-client for Privy embedded wallet operations.
///
/// Embedded wallets are provisioned during login by passing
/// `use_embedded_wallet: true` to `login_with_message()`. This works on any
/// platform. All methods require an active authenticated session.
pub struct Privy<'a> {
    pub(crate) client: &'a LightconeClient,
}

impl<'a> Privy<'a> {
    // ── Builder factories ────────────────────────────────────────────────

    /// Create a Privy order builder for limit orders, pre-seeded with the
    /// client's deposit source.
    ///
    /// Chain `.wallet_id()`, `.maker()`, `.market()`, etc., then call `.submit()`.
    pub async fn limit_order(&self) -> PrivyOrderBuilder<'a> {
        let deposit_source = self.client.deposit_source().await;
        PrivyOrderBuilder::new(self.client, deposit_source, PrivyOrderKind::Limit)
    }

    /// Create a Privy order builder for trigger orders, pre-seeded with the
    /// client's deposit source.
    ///
    /// Chain `.wallet_id()`, `.maker()`, `.market()`, trigger fields, then `.submit()`.
    /// Requires `.trigger_price()` and `.trigger_type()` (or shorthands like `.take_profit()`).
    pub async fn trigger_order(&self) -> PrivyOrderBuilder<'a> {
        let deposit_source = self.client.deposit_source().await;
        PrivyOrderBuilder::new(self.client, deposit_source, PrivyOrderKind::Trigger)
    }

    // ── Direct methods ───────────────────────────────────────────────────

    /// Sign and send a Solana transaction via the user's Privy embedded wallet.
    pub async fn sign_and_send_tx(
        &self,
        wallet_id: &str,
        base64_tx: &str,
    ) -> Result<SignAndSendTxResponse, SdkError> {
        let url = format!(
            "{}/api/privy/sign_and_send_tx",
            self.client.http.base_url()
        );
        let req = SignAndSendTxRequest {
            wallet_id: wallet_id.to_string(),
            base64_tx: base64_tx.to_string(),
        };
        self.client.http.post(&url, &req, RetryPolicy::None).await.map_err(Into::into)
    }

    /// Sign an order hash via Privy and submit it to the exchange engine.
    ///
    /// The backend computes the order hash, signs via Privy, and submits
    /// the signed order internally -- no round-trip back to the client.
    pub async fn sign_and_send_order(
        &self,
        wallet_id: &str,
        order: PrivyOrderEnvelope,
    ) -> Result<serde_json::Value, SdkError> {
        let url = format!(
            "{}/api/privy/sign_and_send_order",
            self.client.http.base_url()
        );
        let req = SignAndSendOrderRequest {
            wallet_id: wallet_id.to_string(),
            order,
        };
        self.client.http.post(&url, &req, RetryPolicy::None).await.map_err(Into::into)
    }

    /// Cancel a limit order via Privy signing.
    ///
    /// The backend signs the cancel message using the embedded wallet
    /// and submits the cancellation to the exchange engine.
    pub async fn sign_and_cancel_order(
        &self,
        wallet_id: &str,
        order_hash: &str,
        maker: &str,
    ) -> Result<serde_json::Value, SdkError> {
        let url = format!(
            "{}/api/privy/sign_and_cancel_order",
            self.client.http.base_url()
        );
        let req = SignAndCancelOrderRequest {
            wallet_id: wallet_id.to_string(),
            maker: maker.to_string(),
            cancel: CancelTarget::Limit {
                order_hash: order_hash.to_string(),
            },
        };
        self.client.http.post(&url, &req, RetryPolicy::None).await.map_err(Into::into)
    }

    /// Cancel a trigger order via Privy signing.
    ///
    /// The backend signs the trigger order ID using the embedded wallet
    /// and submits the cancellation to the exchange engine.
    pub async fn sign_and_cancel_trigger_order(
        &self,
        wallet_id: &str,
        trigger_order_id: &str,
        maker: &str,
    ) -> Result<serde_json::Value, SdkError> {
        let url = format!(
            "{}/api/privy/sign_and_cancel_order",
            self.client.http.base_url()
        );
        let req = SignAndCancelOrderRequest {
            wallet_id: wallet_id.to_string(),
            maker: maker.to_string(),
            cancel: CancelTarget::Trigger {
                trigger_order_id: trigger_order_id.to_string(),
            },
        };
        self.client.http.post(&url, &req, RetryPolicy::None).await.map_err(Into::into)
    }

    /// Cancel all orders for a user via Privy signing.
    ///
    /// The backend signs the cancel-all message using the embedded wallet
    /// and submits the cancellation to the exchange engine.
    pub async fn sign_and_cancel_all_orders(
        &self,
        wallet_id: &str,
        user_pubkey: &str,
        orderbook_id: &str,
        timestamp: i64,
        salt: &str,
    ) -> Result<serde_json::Value, SdkError> {
        let url = format!(
            "{}/api/privy/sign_and_cancel_all_orders",
            self.client.http.base_url()
        );
        let req = SignAndCancelAllRequest {
            wallet_id: wallet_id.to_string(),
            user_pubkey: user_pubkey.to_string(),
            orderbook_id: orderbook_id.to_string(),
            timestamp,
            salt: salt.to_string(),
        };
        self.client.http.post(&url, &req, RetryPolicy::None).await.map_err(Into::into)
    }

    /// Export an embedded wallet's private key (HPKE encrypted).
    ///
    /// The response contains the encrypted private key that only the
    /// user's client can decrypt using their HPKE keypair.
    pub async fn export_wallet(
        &self,
        wallet_id: &str,
        decode_pubkey_base64: &str,
    ) -> Result<ExportWalletResponse, SdkError> {
        let url = format!(
            "{}/api/privy/wallet/export",
            self.client.http.base_url()
        );
        let req = ExportWalletRequest {
            wallet_id: wallet_id.to_string(),
            decode_pubkey_base64: decode_pubkey_base64.to_string(),
        };
        self.client.http.post(&url, &req, RetryPolicy::None).await.map_err(Into::into)
    }
}
