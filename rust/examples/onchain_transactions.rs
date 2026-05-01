mod common;

use common::{get_keypair, market_and_orderbook, quote_deposit_mint, rest_client, ExampleResult};
use solana_signer::Signer;
use solana_transaction::Transaction;

fn describe_tx(name: &str, tx: &Transaction) -> ExampleResult {
    println!(
        "{name}: {} instruction(s), {} bytes, signature={}",
        tx.message.instructions.len(),
        bincode::serialize(tx)?.len(),
        tx.signatures[0]
    );
    Ok(())
}

#[tokio::main]
async fn main() -> ExampleResult {
    let client = rest_client()?;
    let keypair = get_keypair()?;
    let (market, orderbook) = market_and_orderbook(&client).await?;
    let deposit_mint = quote_deposit_mint(&orderbook)?;
    let amount = 1_000_000;
    let blockhash = client.rpc().get_latest_blockhash().await?;

    let mut transactions = vec![
        (
            "deposit",
            client
                .positions()
                .deposit()
                .await
                .user(keypair.pubkey())
                .mint(deposit_mint)
                .amount(amount)
                .with_market_deposit_source(&market)
                .build_tx()
                .await?,
        ),
        (
            "merge",
            client
                .positions()
                .merge()
                .user(keypair.pubkey())
                .market(&market)
                .mint(deposit_mint)
                .amount(amount)
                .build_tx()?,
        ),
        (
            "increment_nonce",
            client.orders().increment_nonce_tx(&keypair.pubkey())?,
        ),
    ];

    let rpc_sub = client.rpc();
    let rpc = rpc_sub.inner()?;
    for (name, tx) in &mut transactions {
        tx.try_sign(&[&keypair], blockhash)?;
        describe_tx(name, tx)?;
        let sig = rpc.send_and_confirm_transaction(tx).await?;
        println!("{name}: confirmed {sig}");
    }

    Ok(())
}
