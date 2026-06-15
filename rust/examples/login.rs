mod common;

use common::{get_keypair, rest_client, ExampleResult};
use lightcone::auth::native::sign_login_message;

#[tokio::main]
async fn main() -> ExampleResult {
    let client = rest_client()?;
    let keypair = get_keypair()?;

    let nonce = client.auth().get_nonce().await?;
    let signed = sign_login_message(&keypair, &nonce);
    let session = client
        .auth()
        .login_with_message(
            &signed.message,
            &signed.signature_bs58,
            &signed.pubkey_bytes,
            None,
        )
        .await?;

    let wallet = session.user.trading_wallet(session.auth_method);
    println!("logged in: {} ({})", session.user.user_id, wallet);
    println!("identity: {}", session.user.identity.text());
    println!("display name: {}", session.user.display_name());
    println!(
        "cached auth state: {}",
        client.auth().is_authenticated().await
    );

    let me = client.auth().check_session().await?;
    println!("session wallet: {}", me.user.trading_wallet(me.auth_method));

    client.auth().logout().await?;
    println!("logged out");
    Ok(())
}
