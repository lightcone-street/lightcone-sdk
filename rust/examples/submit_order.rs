mod common;

use common::{
    fresh_order_nonce, market_and_orderbook, orderbook_mints, parse_pubkey, rest_client,
    scaling_decimals, wallet, ExampleResult,
};
use lightcone::prelude::*;
use solana_signer::Signer;

#[tokio::main]
async fn main() -> ExampleResult {
    let client = rest_client()?;
    let keypair = wallet()?;
    common::login(&client, &keypair, false).await?;

    let (market, orderbook) = market_and_orderbook(&client).await?;
    let decimals = scaling_decimals(&client, &orderbook).await?;
    let (base_mint, quote_mint) = orderbook_mints(&orderbook)?;

    let request = LimitOrderEnvelope::new()
        .maker(keypair.pubkey())
        .market(parse_pubkey(&market.pubkey)?)
        .base_mint(base_mint)
        .quote_mint(quote_mint)
        .bid()
        .price("0.55")
        .size("1")
        .nonce(fresh_order_nonce(&client, &keypair.pubkey()).await?)
        .apply_scaling(&decimals)?
        .sign(&keypair, orderbook.orderbook_id.as_str())?;

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
