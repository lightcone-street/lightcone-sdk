//! Privy sub-client — sign transactions, sign orders, export wallets.

use crate::client::LightconeClient;
use crate::error::SdkError;
use crate::http::RetryPolicy;

use super::{
    ExportWalletRequest, ExportWalletResponse, OrderForSigning, SignAndSendOrderRequest,
    SignAndSendTxRequest, SignAndSendTxResponse,
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
        order: OrderForSigning,
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
