mod common;

use common::{
    deposit_mint, login, market, num_outcomes, parse_pubkey, rest_client, wallet, ExampleResult,
};
use lightcone::program::{get_position_alt_pda, get_position_pda};
use solana_signer::Signer;
use solana_transaction::Transaction;

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

    let (position_pda, _) =
        get_position_pda(&keypair.pubkey(), &market_pubkey, client.program_id());
    let (lookup_table, _) = get_position_alt_pda(&position_pda, recent_slot);

    let instructions: Vec<(&str, solana_instruction::Instruction)> = vec![
        // 1. Init position tokens
        (
            "init_position_tokens",
            client
                .positions()
                .init_position_tokens()
                .payer(keypair.pubkey())
                .user(keypair.pubkey())
                .market(market_pubkey)
                .deposit_mints(vec![deposit_mint])
                .recent_slot(recent_slot)
                .num_outcomes(num_outcomes)
                .build_ix()?,
        ),
        // 2. Deposit to global — fund the global pool with collateral
        (
            "deposit_to_global",
            client
                .positions()
                .deposit_to_global()
                .user(keypair.pubkey())
                .mint(deposit_mint)
                .amount(amount)
                .build_ix()?,
        ),
        // 3. Global to market deposit — move capital into a specific market
        (
            "global_to_market_deposit",
            client
                .positions()
                .global_to_market_deposit()
                .user(keypair.pubkey())
                .market(market_pubkey)
                .mint(deposit_mint)
                .amount(amount)
                .num_outcomes(num_outcomes)
                .build_ix()?,
        ),
        // 4. Extend position tokens — add a new deposit mint to an existing ALT
        (
            "extend_position_tokens",
            client
                .positions()
                .extend_position_tokens()
                .payer(keypair.pubkey())
                .user(keypair.pubkey())
                .market(market_pubkey)
                .lookup_table(lookup_table)
                .deposit_mints(vec![deposit_mint])
                .num_outcomes(num_outcomes)
                .build_ix()?,
        ),
        // 5. Withdraw from global — pull tokens back out of the global pool
        (
            "withdraw_from_global",
            client
                .positions()
                .withdraw_from_global()
                .user(keypair.pubkey())
                .mint(deposit_mint)
                .amount(amount)
                .build_ix()?,
        ),
    ];

    for (name, ix) in &instructions {
        let blockhash = rpc_sub.get_latest_blockhash().await?;
        let mut tx = Transaction::new_with_payer(&[ix.clone()], Some(&keypair.pubkey()));
        tx.try_sign(&[&keypair], blockhash)?;
        let sig = rpc.send_and_confirm_transaction(&tx).await?;
        println!("{name}: confirmed {sig}");
    }

    // ── Unified deposit/withdraw/merge builders ─────────────────────────
    //
    // Deposit and withdraw builders dispatch based on the client's deposit
    // source setting (or a per-call override). Merge is market-only.

    // Deposit — explicitly override to Global
    let global_deposit_ix = client
        .positions()
        .deposit()
        .await
        .user(keypair.pubkey())
        .mint(deposit_mint)
        .amount(amount)
        .with_global_deposit_source()
        .build_ix()
        .await?;
    println!(
        "builder global deposit ix: {} accounts",
        global_deposit_ix.accounts.len()
    );

    // Deposit — explicitly override to Market (mints conditional tokens)
    let market_deposit_ix = client
        .positions()
        .deposit()
        .await
        .user(keypair.pubkey())
        .mint(deposit_mint)
        .amount(amount)
        .with_market_deposit_source(&market)
        .build_ix()
        .await?;
    println!(
        "builder market deposit ix: {} accounts",
        market_deposit_ix.accounts.len()
    );

    // Withdraw — Global mode (global pool → wallet)
    let global_withdraw_ix = client
        .positions()
        .withdraw()
        .await
        .user(keypair.pubkey())
        .mint(deposit_mint)
        .amount(amount)
        .with_global_deposit_source()
        .build_ix()
        .await?;
    println!(
        "builder global withdraw ix: {} accounts",
        global_withdraw_ix.accounts.len()
    );

    // Withdraw — Market mode (position ATA → user's wallet)
    let market_withdraw_ix = client
        .positions()
        .withdraw()
        .await
        .user(keypair.pubkey())
        .mint(deposit_mint)
        .amount(amount)
        .with_market_deposit_source(&market)
        .outcome_index(0)
        .token_2022(true)
        .build_ix()
        .await?;
    println!(
        "builder market withdraw ix: {} accounts",
        market_withdraw_ix.accounts.len()
    );

    // Merge — burns complete set of conditional tokens, releases collateral
    let merge_ix = client
        .positions()
        .merge()
        .user(keypair.pubkey())
        .market(&market)
        .mint(deposit_mint)
        .amount(amount)
        .build_ix()?;
    println!("builder merge ix: {} accounts", merge_ix.accounts.len());

    Ok(())
}
