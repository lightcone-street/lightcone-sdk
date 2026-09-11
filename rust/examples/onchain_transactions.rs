mod common;

use common::{get_keypair, market_and_orderbook, quote_deposit_mint, rest_client, ExampleResult};
use lightcone::program::V1Transaction;
use solana_signer::Signer;

fn describe_tx(name: &str, tx: &V1Transaction) -> ExampleResult {
    println!(
        "{name}: {} instruction(s), {} bytes, signature={}",
        tx.message().instructions.len(),
        tx.to_wire_bytes()?.len(),
        tx.as_versioned().signatures[0]
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
    let context = client.transaction_context().await?;

    let transactions = vec![
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
                .build_tx(&context)
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
                .build_tx(&context)?,
        ),
        (
            "increment_nonce",
            client
                .orders()
                .increment_nonce_tx(&keypair.pubkey(), &context)?,
        ),
    ];

    for (name, tx) in &transactions {
        let tx = tx.sign(&[&keypair])?;
        describe_tx(name, &tx)?;
        let sig = client.submit_signed_transaction(&tx).await?;
        client
            .confirm_signature(&sig, Some(tx.context().last_valid_block_height))
            .await?;
        println!("{name}: confirmed {sig}");
    }

    Ok(())
}
