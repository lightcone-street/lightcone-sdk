mod common;

use common::{deposit_mint, market, rest_client, wallet, ExampleResult};
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
    let keypair = wallet()?;
    let market = market(&client).await?;
    let deposit_mint = deposit_mint(&market)?;
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
