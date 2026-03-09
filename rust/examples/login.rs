mod common;

use common::{rest_client, wallet, ExampleResult};
use lightcone::auth::native::sign_login_message;

#[tokio::main]
async fn main() -> ExampleResult {
    let client = rest_client()?;
    let keypair = wallet()?;

    let nonce = client.auth().get_nonce().await?;
    let signed = sign_login_message(&keypair, &nonce);
    let user = client
        .auth()
        .login_with_message(
            &signed.message,
            &signed.signature_bs58,
            &signed.pubkey_bytes,
            None,
        )
        .await?;

    println!("logged in: {} ({})", user.id, user.wallet_address);
    println!(
        "cached auth state: {}",
        client.auth().is_authenticated().await
    );
    println!(
        "session wallet: {}",
        client.auth().check_session().await?.wallet_address
    );

    client.auth().logout().await?;
    println!("logged out");
    Ok(())
}
