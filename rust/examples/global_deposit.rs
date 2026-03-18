mod common;

use common::{
    deposit_mint, login, market, num_outcomes, parse_pubkey, rest_client, wallet, ExampleResult,
};
use lightcone::program::{
    DepositToGlobalParams, ExtendPositionTokensParams, GlobalToMarketDepositParams,
    InitPositionTokensParams,
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
    let mut init_tx = client.positions().init_position_tokens_ix(
        InitPositionTokensParams {
            payer: keypair.pubkey(),
            user: keypair.pubkey(),
            market: market_pubkey,
            deposit_mints: vec![deposit_mint],
            recent_slot,
        },
        num_outcomes,
    )?;
    let blockhash = rpc_sub.get_latest_blockhash().await?;
    init_tx.try_sign(&[&keypair], blockhash)?;
    let sig = rpc.send_and_confirm_transaction(&init_tx).await?;
    println!("init_position_tokens confirmed: {sig}");

    // 2. Deposit to global — fund the global pool with collateral
    let blockhash = rpc_sub.get_latest_blockhash().await?;
    let mut deposit_tx = client.positions().deposit_to_global_ix(DepositToGlobalParams {
        user: keypair.pubkey(),
        mint: deposit_mint,
        amount,
    })?;
    deposit_tx.try_sign(&[&keypair], blockhash)?;
    let sig = rpc.send_and_confirm_transaction(&deposit_tx).await?;
    println!("deposit_to_global confirmed: {sig}");

    // 3. Global to market deposit — move capital into a specific market
    let blockhash = rpc_sub.get_latest_blockhash().await?;
    let mut move_tx = client.positions().global_to_market_deposit_ix(
        GlobalToMarketDepositParams {
            user: keypair.pubkey(),
            market: market_pubkey,
            deposit_mint,
            amount,
        },
        num_outcomes,
    )?;
    move_tx.try_sign(&[&keypair], blockhash)?;
    let sig = rpc.send_and_confirm_transaction(&move_tx).await?;
    println!("global_to_market_deposit confirmed: {sig}");

    // 4. Extend position tokens — add a new deposit mint to an existing ALT
    //    (only needed when a new deposit mint is whitelisted)
    let position = client
        .positions()
        .get_onchain(&keypair.pubkey(), &market_pubkey)
        .await?
        .ok_or("position not found")?;

    let blockhash = rpc_sub.get_latest_blockhash().await?;
    let mut extend_tx = client.positions().extend_position_tokens_ix(
        ExtendPositionTokensParams {
            payer: keypair.pubkey(),
            user: keypair.pubkey(),
            market: market_pubkey,
            lookup_table: position.lookup_table,
            deposit_mints: vec![deposit_mint],
        },
        num_outcomes,
    )?;
    extend_tx.try_sign(&[&keypair], blockhash)?;
    let sig = rpc.send_and_confirm_transaction(&extend_tx).await?;
    println!("extend_position_tokens confirmed: {sig}");

    Ok(())
}
