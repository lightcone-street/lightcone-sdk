mod common;

use common::{
    deposit_mint, login, market, num_outcomes, parse_pubkey, rest_client, wallet, ExampleResult,
};
use lightcone::program::{
    get_position_alt_pda, get_position_pda, DepositToGlobalParams, ExtendPositionTokensParams,
    GlobalToMarketDepositParams, InitPositionTokensParams,
};
use solana_signer::Signer;

#[tokio::main]
async fn main() -> ExampleResult {
    let client = rest_client()?;
    let keypair = wallet()?;
    login(&client, &keypair, false).await?;

    let market = market(&client).await?;
    let market_pubkey = parse_pubkey(&market.pubkey)?;
    let deposit_mint = deposit_mint(&market)?;
    let num_outcomes = num_outcomes(&market)?;
    let amount = 1_000_000;

    let rpc_sub = client.rpc();
    let rpc = rpc_sub.inner()?;

    // 1. Init position tokens — one-time setup per market (creates position + ALT)
    let recent_slot = rpc.get_slot().await?;

    // 4 uses the ALT address derived from position PDA + the slot used during init
    let (position_pda, _) = get_position_pda(&keypair.pubkey(), &market_pubkey, client.program_id());
    let (lookup_table, _) = get_position_alt_pda(&position_pda, recent_slot);

    let mut transactions = vec![
        (
            "init_position_tokens",
            client.positions().init_position_tokens_ix(
                InitPositionTokensParams {
                    payer: keypair.pubkey(),
                    user: keypair.pubkey(),
                    market: market_pubkey,
                    deposit_mints: vec![deposit_mint],
                    recent_slot,
                },
                num_outcomes,
            )?,
        ),
        // 2. Deposit to global — fund the global pool with collateral
        (
            "deposit_to_global",
            client.positions().deposit_to_global_ix(DepositToGlobalParams {
                user: keypair.pubkey(),
                mint: deposit_mint,
                amount,
            })?,
        ),
        // 3. Global to market deposit — move capital into a specific market
        (
            "global_to_market_deposit",
            client.positions().global_to_market_deposit_ix(
                GlobalToMarketDepositParams {
                    user: keypair.pubkey(),
                    market: market_pubkey,
                    deposit_mint,
                    amount,
                },
                num_outcomes,
            )?,
        ),
        // 4. Extend position tokens — add a new deposit mint to an existing ALT
        (
            "extend_position_tokens",
            client.positions().extend_position_tokens_ix(
                ExtendPositionTokensParams {
                    payer: keypair.pubkey(),
                    user: keypair.pubkey(),
                    market: market_pubkey,
                    lookup_table,
                    deposit_mints: vec![deposit_mint],
                },
                num_outcomes,
            )?,
        ),
    ];

    for (name, tx) in &mut transactions {
        let blockhash = rpc_sub.get_latest_blockhash().await?;
        tx.try_sign(&[&keypair], blockhash)?;
        let sig = rpc.send_and_confirm_transaction(tx).await?;
        println!("{name}: confirmed {sig}");
    }

    Ok(())
}
