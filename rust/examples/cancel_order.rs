mod common;

use common::{
    deposit_mint, get_keypair, login, market, rest_client, unix_timestamp, ExampleResult,
};
use lightcone::prelude::*;
use solana_signer::Signer;
use solana_transaction::Transaction;

// Mirrors the constant in `submit_order.rs`. When we cancel the order that
// example left open, we withdraw the same quote amount back from the global
// pool so the deposit/submit/cancel/withdraw cycle is net-neutral.
const ORDER_QUOTE_AMOUNT: u64 = 1_100_000; // 0.55 * 2 USDC, 6 decimals

#[tokio::main]
async fn main() -> ExampleResult {
    let client = rest_client()?;
    let keypair = get_keypair()?;
    login(&client, &keypair, false).await?;

    let snapshot = client.orders().get_user_orders(Some(50), None).await?;

    let Some((order_hash, orderbook_id)) = snapshot.orders.iter().find_map(|order| match order {
        UserSnapshotOrder::Limit { common, .. } => {
            Some((common.order_hash.clone(), common.orderbook_id.clone()))
        }
        UserSnapshotOrder::Trigger { .. } => None,
    }) else {
        println!("No open limit orders to cancel.");
        return Ok(());
    };

    let cancel = CancelBody::signed(order_hash, keypair.pubkey().into(), &keypair);
    let salt = generate_cancel_all_salt();
    let cancel_all = CancelAllBody::signed(
        keypair.pubkey().into(),
        orderbook_id,
        unix_timestamp()?,
        salt,
        &keypair,
    );

    let cancelled = client.orders().cancel(&cancel).await?;
    let cleared = client.orders().cancel_all(&cancel_all).await?;

    println!(
        "cancelled: {} remaining={}",
        cancelled.order_hash, cancelled.remaining
    );
    println!(
        "cancel-all removed {} order(s) in {}",
        cleared.count, cleared.orderbook_id
    );

    // Cleanup: cancelling the order released its locked collateral back into
    // the global pool. Withdraw that amount to the user's token account so the
    // companion `submit_order` → `cancel_order` cycle is net-neutral on the
    // wallet's balance and the global pool.
    let market = market(&client).await?;
    let mint = deposit_mint(&market)?;
    let rpc_sub = client.rpc();
    let rpc = rpc_sub.inner()?;
    let withdraw_ix = client
        .positions()
        .withdraw_from_global()
        .user(keypair.pubkey())
        .mint(mint)
        .amount(ORDER_QUOTE_AMOUNT)
        .build_ix()?;
    let blockhash = rpc_sub.get_latest_blockhash().await?;
    let mut withdraw_tx = Transaction::new_with_payer(&[withdraw_ix], Some(&keypair.pubkey()));
    withdraw_tx.try_sign(&[&keypair], blockhash)?;
    let withdraw_sig = rpc.send_and_confirm_transaction(&withdraw_tx).await?;
    println!("withdraw_from_global: confirmed {withdraw_sig}");

    Ok(())
}
