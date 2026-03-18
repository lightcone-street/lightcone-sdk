mod common;

use common::{
    deposit_mint, login, market, num_outcomes, parse_pubkey, rest_client, wallet, ExampleResult,
};
use lightcone::{
    domain::position::wire::PositionsResponse,
    program::{DepositToGlobalParams, GlobalToMarketDepositParams},
};
use solana_pubkey::Pubkey;
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

fn global_balance(response: &PositionsResponse, mint: &Pubkey) -> Option<&str> {
    let mint = mint.to_string();
    response
        .global_deposits
        .iter()
        .find(|deposit| deposit.deposit_mint == mint)
        .map(|deposit| deposit.balance.as_str())
}

#[tokio::main]
async fn main() -> ExampleResult {
    let client = rest_client()?;
    let keypair = wallet()?;
    let user = login(&client, &keypair, false).await?;
    let market = market(&client).await?;
    let market_pubkey = parse_pubkey(&market.pubkey)?;
    let deposit_mint = deposit_mint(&market)?;
    let num_outcomes = num_outcomes(&market)?;
    let amount = 1_000_000;

    let whitelist = client.rpc().get_global_deposit_token(&deposit_mint).await?;
    let before = client.positions().get(user.wallet_address.as_str()).await?;

    println!("wallet: {}", user.wallet_address);
    println!("market: {} ({})", market.slug, market.pubkey);
    println!("deposit mint: {}", deposit_mint);
    println!(
        "global deposit token: active={} index={}",
        whitelist.active, whitelist.index
    );
    println!(
        "pdas: whitelist={} user_global_deposit={}",
        client.rpc().get_global_deposit_token_pda(&deposit_mint),
        client
            .rpc()
            .get_user_global_deposit_pda(&keypair.pubkey(), &deposit_mint)
    );
    println!(
        "global balance before: {}",
        global_balance(&before, &deposit_mint).unwrap_or("0")
    );

    let blockhash = client.rpc().get_latest_blockhash().await?;
    let mut deposit_tx = client.positions().deposit_to_global_ix(DepositToGlobalParams {
        user: keypair.pubkey(),
        mint: deposit_mint,
        amount,
    })?;
    let mut global_to_market_tx = client.positions().global_to_market_deposit_ix(
        GlobalToMarketDepositParams {
            user: keypair.pubkey(),
            market: market_pubkey,
            deposit_mint,
            amount,
        },
        num_outcomes,
    )?;

    deposit_tx.try_sign(&[&keypair], blockhash)?;
    global_to_market_tx.try_sign(&[&keypair], blockhash)?;

    describe_tx("deposit_to_global", &deposit_tx)?;
    describe_tx("global_to_market_deposit", &global_to_market_tx)?;

    let rpc = client.rpc().inner()?;
    let deposit_sig = rpc.send_and_confirm_transaction(&deposit_tx).await?;
    println!("deposit_to_global: confirmed {deposit_sig}");
    let global_to_market_sig = rpc.send_and_confirm_transaction(&global_to_market_tx).await?;
    println!("global_to_market_deposit: confirmed {global_to_market_sig}");

    let after = client.positions().get(user.wallet_address.as_str()).await?;
    let per_market = client
        .positions()
        .get_for_market(user.wallet_address.as_str(), market.pubkey.as_str())
        .await?;

    println!(
        "global balance after: {}",
        global_balance(&after, &deposit_mint).unwrap_or("0")
    );
    println!(
        "positions in {} after deposit: {}",
        market.slug,
        per_market.positions.len()
    );
    Ok(())
}
