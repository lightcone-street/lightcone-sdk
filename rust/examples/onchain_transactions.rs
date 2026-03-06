mod common;

use common::{
    deposit_mint, market, num_outcomes, parse_pubkey, rest_client, rpc_client, token_2022, wallet,
    ExampleResult,
};
use lightcone::program::{
    MergeCompleteSetParams, MintCompleteSetParams, WithdrawFromPositionParams,
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
    let rpc = rpc_client();
    let keypair = wallet()?;
    let market = market(&client).await?;
    let market_pubkey = parse_pubkey(&market.pubkey)?;
    let deposit_mint = deposit_mint(&market)?;
    let num_outcomes = num_outcomes(&market)?;
    let amount = 1_000_000;
    let blockhash = rpc.get_latest_blockhash().await?;

    let mut transactions = vec![
        (
            "mint_complete_set",
            rpc.mint_complete_set(
                MintCompleteSetParams {
                    user: keypair.pubkey(),
                    market: market_pubkey,
                    deposit_mint,
                    amount,
                },
                num_outcomes,
            )
            .await?,
        ),
        (
            "merge_complete_set",
            rpc.merge_complete_set(
                MergeCompleteSetParams {
                    user: keypair.pubkey(),
                    market: market_pubkey,
                    deposit_mint,
                    amount,
                },
                num_outcomes,
            )
            .await?,
        ),
        (
            "withdraw_from_position",
            rpc.withdraw_from_position(
                WithdrawFromPositionParams {
                    user: keypair.pubkey(),
                    market: market_pubkey,
                    mint: deposit_mint,
                    amount,
                    outcome_index: 255,
                },
                token_2022(),
            )
            .await?,
        ),
        (
            "increment_nonce",
            rpc.increment_nonce(&keypair.pubkey()).await?,
        ),
    ];

    for (name, tx) in &mut transactions {
        tx.try_sign(&[&keypair], blockhash)?;
        describe_tx(name, tx)?;
    }

    Ok(())
}
