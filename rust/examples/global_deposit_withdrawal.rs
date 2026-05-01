mod common;

use common::{
    get_keypair, login, market_and_orderbook, num_outcomes, parse_pubkey, quote_deposit_mint,
    rest_client, ExampleResult,
};
use lightcone::program::{get_position_alt_pda, get_position_pda};
use solana_commitment_config::CommitmentConfig;
use solana_signer::Signer;
use solana_transaction::Transaction;

#[tokio::main]
async fn main() -> ExampleResult {
    let client = rest_client()?;
    let keypair = get_keypair()?;
    login(&client, &keypair, false).await?;

    let (market, orderbook) = market_and_orderbook(&client).await?;
    let market_pubkey = parse_pubkey(&market.pubkey)?;
    let deposit_mint = quote_deposit_mint(&orderbook)?;
    let num_outcomes = num_outcomes(&market)?;
    let amount = 1_000_000;
    let deposit_amount = amount * 2; // deposit extra so global has funds after market transfer

    let rpc_sub = client.rpc();
    let rpc = rpc_sub.inner()?;

    let (position_pda, _) =
        get_position_pda(&keypair.pubkey(), &market_pubkey, client.program_id());

    // Check if position already exists (init_position_tokens is one-time per market)
    let position_account = rpc.get_account(&position_pda).await;
    let needs_init = position_account.is_err();

    if needs_init {
        // Init + extend in a single transaction.
        // Use "processed" commitment for get_slot to minimize staleness — the
        // on-chain CreateLookupTable instruction rejects slots that are too old.
        let max_attempts = 5;
        for attempt in 1..=max_attempts {
            let recent_slot = rpc
                .get_slot_with_commitment(CommitmentConfig::processed())
                .await?;
            let (lookup_table, _) = get_position_alt_pda(&position_pda, recent_slot);

            let init_ix = client
                .positions()
                .init_position_tokens()
                .payer(keypair.pubkey())
                .user(keypair.pubkey())
                .market(market_pubkey)
                .deposit_mints(vec![deposit_mint.clone()])
                .recent_slot(recent_slot)
                .num_outcomes(num_outcomes)
                .build_ix()?;

            let extend_ix = client
                .positions()
                .extend_position_tokens()
                .payer(keypair.pubkey())
                .user(keypair.pubkey())
                .market(market_pubkey)
                .lookup_table(lookup_table)
                .deposit_mints(vec![deposit_mint.clone()])
                .num_outcomes(num_outcomes)
                .build_ix()?;

            let blockhash = rpc_sub.get_latest_blockhash().await?;
            let mut tx =
                Transaction::new_with_payer(&[init_ix, extend_ix], Some(&keypair.pubkey()));
            tx.try_sign(&[&keypair], blockhash)?;

            match rpc.send_and_confirm_transaction(&tx).await {
                Ok(sig) => {
                    println!("init_position_tokens: confirmed {sig}");
                    break;
                }
                Err(error) => {
                    let message = error.to_string();
                    let retryable = message.contains("is not a recent slot")
                        || message.contains("UninitializedAccount")
                        || message.contains("already in use");
                    if attempt < max_attempts && retryable {
                        println!(
                            "init_position_tokens: retrying ({attempt}/{max_attempts}): {}",
                            &message[..message.len().min(80)]
                        );
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                        continue;
                    }
                    return Err(error.into());
                }
            }
        }
    } else {
        println!("position already initialized, skipping init_position_tokens + extend");
    }

    let mut instructions: Vec<(&str, solana_instruction::Instruction)> = vec![];

    // 3. Deposit to global — fund the global pool with collateral
    instructions.push((
        "deposit_to_global",
        client
            .positions()
            .deposit_to_global()
            .user(keypair.pubkey())
            .mint(deposit_mint)
            .amount(deposit_amount)
            .build_ix()?,
    ));

    // 4. Global to market deposit — move capital into a specific market
    instructions.push((
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
    ));

    // 5. Withdraw from global — pull tokens back out of the global pool
    instructions.push((
        "withdraw_from_global",
        client
            .positions()
            .withdraw_from_global()
            .user(keypair.pubkey())
            .mint(deposit_mint)
            .amount(amount)
            .build_ix()?,
    ));

    // 6. Merge — burn the complete set of conditional tokens minted in step 4
    //    back to the deposit asset, returning the collateral to the user's
    //    token account. Closes out the market position so the full example is
    //    net-neutral on the wallet's balance, the global pool, and the market
    //    position across CI runs.
    instructions.push((
        "merge",
        client
            .positions()
            .merge()
            .user(keypair.pubkey())
            .market(&market)
            .mint(deposit_mint)
            .amount(amount)
            .build_ix()?,
    ));

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
