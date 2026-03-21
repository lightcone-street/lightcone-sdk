mod common;

use common::{fresh_order_nonce, market_and_orderbook, rest_client, wallet, ExampleResult};
use lightcone::prelude::*;
use solana_signer::Signer;

#[tokio::main]
async fn main() -> ExampleResult {
    let client = rest_client()?;
    let keypair = wallet()?;
    common::login(&client, &keypair, false).await?;

    let (_market, orderbook) = market_and_orderbook(&client).await?;

    let request = client
        .orders()
        .limit_order()
        .await
        .maker(keypair.pubkey())
        .bid()
        .price("0.55")
        .size("1")
        .nonce(fresh_order_nonce(&client, &keypair.pubkey()).await?)
        .salt(lightcone::program::orders::generate_salt())
        .sign(&keypair, &orderbook)?;

    let response = client.orders().submit(&request).await?;
    println!(
        "submitted: {} filled={} remaining={} fills={}",
        response.order_hash,
        response.filled,
        response.remaining,
        response.fills.len()
    );
    Ok(())
}
