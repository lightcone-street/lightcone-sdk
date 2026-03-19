//! Signing strategy types for client-level signing configuration.
//!
//! The signing strategy determines how orders, cancels, and transactions
//! are signed. Set it on the client at construction time or update at runtime.

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

#[cfg(feature = "native-auth")]
use solana_keypair::Keypair;

/// Trait for external wallet signers (browser wallet adapters).
///
/// Implement this trait to integrate a browser wallet adapter with the SDK.
/// The SDK calls these methods internally when the signing strategy is `WalletAdapter`.
///
/// # Example
///
/// ```rust,ignore
/// struct MyAdapterSigner;
///
/// impl ExternalSigner for MyAdapterSigner {
///     fn sign_message(&self, message: &[u8])
///         -> Pin<Box<dyn Future<Output = Result<Vec<u8>, String>> + '_>>
///     {
///         Box::pin(async move {
///             let result = ADAPTER().sign_message(message).await.map_err(|e| format!("{e:?}"))?;
///             Ok(result.signature_bytes())
///         })
///     }
///
///     fn sign_transaction(&self, tx_bytes: &[u8])
///         -> Pin<Box<dyn Future<Output = Result<Vec<u8>, String>> + '_>>
///     {
///         Box::pin(async move {
///             let result = ADAPTER().sign_transaction(tx_bytes, None).await.map_err(|e| format!("{e:?}"))?;
///             Ok(result)
///         })
///     }
/// }
/// ```
pub trait ExternalSigner: Send + Sync {
    /// Sign a message and return the raw signature bytes.
    fn sign_message<'a>(
        &'a self,
        message: &'a [u8],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, String>> + 'a>>;

    /// Sign a serialized unsigned transaction and return the signed transaction bytes.
    fn sign_transaction<'a>(
        &'a self,
        tx_bytes: &'a [u8],
    ) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, String>> + 'a>>;
}

/// Signing strategy for the client.
///
/// Determines how orders, cancels, and transactions are signed.
/// Set via builder methods or `client.set_signing_strategy()` at runtime.
pub enum SigningStrategy {
    /// Native keypair signing (CLI, bots).
    /// Signs locally using the provided keypair.
    #[cfg(feature = "native-auth")]
    Native(Arc<Keypair>),

    /// External wallet adapter (browser).
    /// Delegates signing to the provided `ExternalSigner` implementation.
    WalletAdapter(Arc<dyn ExternalSigner>),

    /// Privy embedded wallet (backend-managed signing).
    /// The backend signs on behalf of the user using the Privy wallet.
    Privy { wallet_id: String },
}

/// Check if an external signer error indicates the user cancelled/rejected
/// the wallet popup. Returns `SdkError::UserCancelled` if so, otherwise
/// wraps in `SdkError::Signing`.
pub(crate) fn classify_signer_error(error: String) -> crate::error::SdkError {
    let lower = error.to_lowercase();
    let is_cancellation = lower.contains("reject")
        || lower.contains("cancel")
        || lower.contains("denied")
        || lower.contains("user refused")
        || lower.contains("declined")
        // wallet-adapter wraps JS rejection as InternalError with this message
        || lower.contains("reflect.get called on non-object");

    if is_cancellation {
        crate::error::SdkError::UserCancelled
    } else {
        crate::error::SdkError::Signing(error)
    }
}

impl Clone for SigningStrategy {
    fn clone(&self) -> Self {
        match self {
            #[cfg(feature = "native-auth")]
            Self::Native(keypair) => Self::Native(keypair.clone()),
            Self::WalletAdapter(signer) => Self::WalletAdapter(signer.clone()),
            Self::Privy { wallet_id } => Self::Privy {
                wallet_id: wallet_id.clone(),
            },
        }
    }
}
