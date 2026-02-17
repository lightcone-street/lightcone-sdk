//! Native auth — keypair-based signing.
//!
//! Only available with the `native-auth` feature.

use solana_keypair::Keypair;
use solana_signer::Signer;

use crate::auth::generate_signin_message;

/// Sign a login message with a local keypair.
///
/// Returns (message_base58, signature_base58, pubkey_base58).
pub fn sign_login_message(
    keypair: &Keypair,
    timestamp: u64,
) -> (String, String, String) {
    let message = generate_signin_message(timestamp);
    let signature = keypair.sign_message(&message);

    (
        bs58::encode(&message).into_string(),
        signature.to_string(),
        keypair.pubkey().to_string(),
    )
}
