//! Native auth — keypair-based signing.
//!
//! Only available with the `native-auth` feature.

use solana_keypair::Keypair;
use solana_signer::Signer;

use crate::auth::generate_signin_message;

/// Signed login material ready to pass to `client.auth().login_with_message()`.
pub struct SignedLogin {
    pub message: String,
    pub signature_bs58: String,
    pub pubkey_bytes: [u8; 32],
}

/// Sign a login message with a local keypair.
///
/// Returns a [`SignedLogin`] that can be passed directly to
/// `client.auth().login_with_message()`.
pub fn sign_login_message(keypair: &Keypair, timestamp: u64) -> SignedLogin {
    let message_bytes = generate_signin_message(timestamp);
    let signature = keypair.sign_message(&message_bytes);
    let message = String::from_utf8(message_bytes)
        .expect("generate_signin_message always produces valid UTF-8");

    SignedLogin {
        message,
        signature_bs58: signature.to_string(),
        pubkey_bytes: keypair.pubkey().to_bytes(),
    }
}
