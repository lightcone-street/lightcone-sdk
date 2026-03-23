mod common;

use common::{
    deposit_mint, market, num_outcomes, parse_pubkey, rest_client, wallet, ExampleResult,
};
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
    let market_pubkey = parse_pubkey(&market.pubkey)?;
    let deposit_mint = deposit_mint(&market)?;
    let num_outcomes = num_outcomes(&market)?;
    let amount = 1_000_000;
    let blockhash = client.rpc().get_latest_blockhash().await?;

    let mut transactions = vec![
        (
            "mint_complete_set",
            client
                .markets()
                .mint_complete_set()
                .user(keypair.pubkey())
                .market(market_pubkey)
                .mint(deposit_mint)
                .amount(amount)
                .num_outcomes(num_outcomes)
                .build_tx()?,
        ),
        (
            "merge_complete_set",
            client
                .markets()
                .merge_complete_set()
                .user(keypair.pubkey())
                .market(market_pubkey)
                .mint(deposit_mint)
                .amount(amount)
                .num_outcomes(num_outcomes)
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
