mod common;

use common::{
    fresh_order_nonce, get_keypair, market_and_orderbook, quote_deposit_mint, rest_client,
    wait_for_global_balance, ExampleResult,
};
use lightcone::prelude::*;
use solana_signer::Signer;
use solana_transaction::Transaction;
use std::sync::Arc;

// Quote needed for the bid below (price * size, scaled to the deposit asset's
// decimals). Must stay in sync with the same constant in `cancel_order.rs`,
// which withdraws this amount back out of the global pool after cancelling.
const ORDER_QUOTE_AMOUNT: u64 = 1_100_000; // 0.55 * 2 USDC, 6 decimals

#[tokio::main]
async fn main() -> ExampleResult {
    let client = rest_client()?;
    let keypair = Arc::new(get_keypair()?);
    let maker = keypair.pubkey();
    common::login(&client, keypair.as_ref(), false).await?;

    let (_market, orderbook) = market_and_orderbook(&client).await?;
    let mint = quote_deposit_mint(&orderbook)?;
    let rpc_sub = client.rpc();
    let rpc = rpc_sub.inner()?;

    // 1. Deposit collateral into the global pool.
    //
    // submit_order uses the client's default DepositSource (Global), so the
    // global pool must cover `price * size` in the deposit asset's base units
    // before the order can be placed. The companion `cancel_order` example
    // cancels this order and withdraws the same amount back to the user's
    // token account, keeping the deposit/submit/cancel/withdraw cycle
    // net-neutral across CI runs.
    let deposit_ix = client
        .positions()
        .deposit_to_global()
        .user(maker)
        .mint(mint)
        .amount(ORDER_QUOTE_AMOUNT)
        .build_ix()?;
    let blockhash = rpc_sub.get_latest_blockhash().await?;
    let mut deposit_tx = Transaction::new_with_payer(&[deposit_ix], Some(&maker));
    deposit_tx.try_sign(&[keypair.as_ref()], blockhash)?;
    let deposit_sig = rpc.send_and_confirm_transaction(&deposit_tx).await?;
    println!("deposit_to_global: confirmed {deposit_sig}");

    client
        .set_signing_strategy(SigningStrategy::Native(keypair.clone()))
        .await;

    wait_for_global_balance(&client, &mint, rust_decimal::Decimal::new(11, 1)).await?;

    // 2. Submit the limit order. Fetch and cache the on-chain nonce once —
    //    subsequent orders that omit `.nonce()` use this cached value.
    let nonce = fresh_order_nonce(&client, &maker).await?;
    client.set_order_nonce(nonce).await;

    let response = client
        .orders()
        .limit_order()
        .await
        .maker(maker)
        .bid()
        .price("0.55")
        .size("2")
        .salt(lightcone::program::orders::generate_salt())
        .submit(&client, &orderbook)
        .await?;
    println!(
        "submitted: {} filled={} remaining={} fills={}",
        response.order_hash,
        response.filled,
        response.remaining,
        response.fills.len()
    );

    Ok(())
}
