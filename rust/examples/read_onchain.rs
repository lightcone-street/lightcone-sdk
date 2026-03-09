mod common;

use common::{
    deposit_mint, market, orderbook_mints, parse_pubkey, rest_client, rpc_client, wallet,
    ExampleResult,
};
use solana_signer::Signer;

#[tokio::main]
async fn main() -> ExampleResult {
    let client = rest_client()?;
    let rpc = rpc_client();
    let keypair = wallet()?;
    let market = market(&client).await?;
    let market_pubkey = parse_pubkey(&market.pubkey)?;
    let orderbook = market
        .orderbook_pairs
        .iter()
        .find(|pair| pair.active)
        .or_else(|| market.orderbook_pairs.first())
        .ok_or_else(|| common::other("selected market has no orderbooks"))?;
    let (base_mint, quote_mint) = orderbook_mints(orderbook)?;

    let exchange = rpc.get_exchange().await?;
    let onchain_market = rpc.get_market_by_pubkey(&market_pubkey).await?;
    let onchain_orderbook = rpc.get_orderbook(&base_mint, &quote_mint).await?;
    let nonce = rpc.get_current_nonce(&keypair.pubkey()).await?;
    let position = rpc.get_position(&keypair.pubkey(), &market_pubkey).await?;
    let deposit_mint = deposit_mint(&market)?;

    println!(
        "exchange: authority={} operator={} paused={}",
        exchange.authority, exchange.operator, exchange.paused
    );
    println!(
        "market: id={} outcomes={} status={:?}",
        onchain_market.market_id, onchain_market.num_outcomes, onchain_market.status
    );
    println!(
        "orderbook: lookup_table={} bump={}",
        onchain_orderbook.lookup_table, onchain_orderbook.bump
    );
    println!("user nonce: {}", nonce);
    println!("position exists: {}", position.is_some());
    println!(
        "pdas: exchange={} market={} position={} global_deposit={}",
        rpc.get_exchange_pda(),
        rpc.get_market_pda(onchain_market.market_id),
        rpc.get_position_pda(&keypair.pubkey(), &market_pubkey),
        rpc.get_global_deposit_token_pda(&deposit_mint)
    );
    Ok(())
}
