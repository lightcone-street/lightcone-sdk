//! Integration tests for nonce-based wallet authentication.
//!
//! Exercises the full challenge-response flow against a running API:
//!   1. GET /api/auth/nonce returns a valid single-use nonce
//!   2. Login with no nonce → rejected
//!   3. Login with a bogus nonce → rejected
//!   4. Happy path: fetch nonce → sign → login → success
//!   5. Replay the same nonce → rejected
//!
//! All tests are `#[ignore]` because they require a running server.
//!
//! Run with:
//! ```bash
//! API_URL=http://localhost:8080 cargo test -p lightcone-sdk-v2 --features native \
//!     --test nonce_auth_integration -- --ignored --nocapture
//! ```

use lightcone_sdk_v2::auth::{self, LoginRequest};
use lightcone_sdk_v2::client::LightconeClient;

use solana_keypair::Keypair;
use solana_signer::Signer;

fn api_url() -> String {
    std::env::var("API_URL").unwrap_or_else(|_| "http://localhost:8080".into())
}

fn make_client() -> LightconeClient {
    LightconeClient::builder()
        .base_url(&api_url())
        .build()
        .expect("failed to build client")
}

async fn raw_login(req: &LoginRequest) -> reqwest::Response {
    let url = format!("{}/api/auth/login_or_register_with_message", api_url());
    reqwest::Client::new()
        .post(&url)
        .json(req)
        .send()
        .await
        .expect("request failed")
}

#[tokio::test]
#[ignore]
async fn get_nonce_returns_64_hex_chars() {
    let client = make_client();
    let nonce = client.auth().get_nonce().await.expect("get_nonce failed");

    assert_eq!(nonce.len(), 64, "nonce should be 64 hex chars, got {}", nonce.len());
    assert!(
        nonce.chars().all(|c| c.is_ascii_hexdigit()),
        "nonce should be hex, got: {nonce}"
    );
    println!("nonce = {nonce}");
}

#[tokio::test]
#[ignore]
async fn login_without_nonce_is_rejected() {
    let kp = Keypair::new();
    let message = "Welcome to Lightcone!";
    let sig = kp.sign_message(message.as_bytes());

    let resp = raw_login(&LoginRequest {
        message: message.to_string(),
        signature_bs58: sig.to_string(),
        pubkey_bytes: kp.pubkey().to_bytes().to_vec(),
        use_embedded_wallet: None,
    })
    .await;

    assert_eq!(resp.status().as_u16(), 401, "expected 401 for missing nonce");
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["code"], "INVALID_NONCE");
}

#[tokio::test]
#[ignore]
async fn login_with_bogus_nonce_is_rejected() {
    let kp = Keypair::new();
    let fake_nonce = "deadbeef".repeat(8);
    let message_bytes = auth::generate_signin_message(&fake_nonce);
    let message = String::from_utf8(message_bytes.clone()).unwrap();
    let sig = kp.sign_message(&message_bytes);

    let resp = raw_login(&LoginRequest {
        message,
        signature_bs58: sig.to_string(),
        pubkey_bytes: kp.pubkey().to_bytes().to_vec(),
        use_embedded_wallet: None,
    })
    .await;

    assert_eq!(resp.status().as_u16(), 401, "expected 401 for bogus nonce");
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["code"], "INVALID_NONCE");
}

#[tokio::test]
#[ignore]
async fn happy_path_login_with_valid_nonce() {
    let client = make_client();
    let kp = Keypair::new();

    let nonce = client.auth().get_nonce().await.expect("get_nonce failed");
    let signed = auth::native::sign_login_message(&kp, &nonce);

    let user = client
        .auth()
        .login_with_message(
            &signed.message,
            &signed.signature_bs58,
            &signed.pubkey_bytes,
            None,
        )
        .await
        .expect("login should succeed");

    assert_eq!(user.wallet_address, kp.pubkey().to_string());
    println!("logged in as user {} ({})", user.id, user.wallet_address);
}

#[tokio::test]
#[ignore]
async fn replay_same_nonce_is_rejected() {
    let client = make_client();
    let kp = Keypair::new();

    let nonce = client.auth().get_nonce().await.expect("get_nonce failed");
    let signed = auth::native::sign_login_message(&kp, &nonce);

    // First login consumes the nonce
    client
        .auth()
        .login_with_message(
            &signed.message,
            &signed.signature_bs58,
            &signed.pubkey_bytes,
            None,
        )
        .await
        .expect("first login should succeed");

    // Same nonce again — should be rejected
    let resp = raw_login(&LoginRequest {
        message: signed.message,
        signature_bs58: signed.signature_bs58,
        pubkey_bytes: signed.pubkey_bytes.to_vec(),
        use_embedded_wallet: None,
    })
    .await;

    assert_eq!(resp.status().as_u16(), 401, "replayed nonce should be rejected");
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["code"], "INVALID_NONCE");
}
