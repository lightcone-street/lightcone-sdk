//! Fund-moving local/staging native SOL withdrawal example.
//!
//! The example initializes exact native and canonical WSOL state, plans a native
//! withdrawal without closing canonical WSOL, confirms with a slot, and
//! refreshes a complete snapshot covering that slot before publishing new state.

mod common;

use common::{get_keypair, get_keypair_from_env, login, other, rest_client, ExampleResult};
use futures_util::StreamExt;
use lightcone::{
    prelude::{
        PubkeyStr, SigningStrategy, WalletDepositBalancesApplyResult, WalletDepositBalancesState,
    },
    ws::{Kind, MessageOut, WsEvent},
};
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use std::{env, sync::Arc, time::Duration};

/// Native SOL transferred per run, expressed in lamports (0.001 SOL).
const WITHDRAW_AMOUNT_LAMPORTS: u64 = 1_000_000;

/// Run the fund-moving lifecycle against the configured non-production wallets.
#[tokio::main]
async fn main() -> ExampleResult {
    require_non_production()?;
    // SDK wallets form a stable funding cycle: Rust -> TypeScript -> Python -> Rust.
    // Reusing the existing paths avoids both a recipient-specific setting and
    // repeated faucet top-offs when this fund-moving example is run in CI.
    let recipient: Pubkey = get_keypair_from_env("LIGHTCONE_WALLET_PATH_TS", None)?.pubkey();
    let client = rest_client()?;
    let keypair = Arc::new(get_keypair()?);
    if recipient == keypair.pubkey() {
        return Err(other("Rust and TypeScript SDK wallet paths must identify peers").into());
    }
    let session = login(&client, keypair.as_ref(), false).await?;
    let wallet = session.user.trading_wallet(session.auth_method);

    let mut state = WalletDepositBalancesState::default();

    // Register the wallet stream before any action. Balance updates are
    // ignored until its first complete snapshot establishes the state baseline.
    let mut ws = client.ws_native();
    ws.connect().await?;
    ws.send(MessageOut::subscribe_wallet_deposit_balances(
        PubkeyStr::from(wallet),
    ))?;
    wait_for_wallet_state(&ws, &mut state, "initial snapshot", |_| true).await?;

    println!("wallet: {}", wallet);
    println!(
        "context slot: {}",
        state
            .context_slot
            .ok_or_else(|| other("wallet balance snapshot omitted context slot"))?
    );
    println!(
        "native SOL: {}",
        state
            .native_sol_balance
            .as_deref()
            .ok_or_else(|| other("wallet balance snapshot omitted native SOL"))?
    );
    println!("native + canonical WSOL: {}", state.combined_sol_balance()?);
    println!("tracked balances: {}", state.balances.len());

    let mut entries: Vec<_> = state.balances.values().collect();
    entries.sort_by(|a, b| a.symbol.cmp(&b.symbol));
    for balance in entries {
        println!(
            "  {:>8}  {:<42}  idle={}",
            balance.symbol, balance.mint, balance.idle
        );
    }

    client
        .set_signing_strategy(SigningStrategy::Native(keypair.clone()))
        .await;

    let plan = client
        .positions()
        .plan_native_sol_withdrawal(recipient, WITHDRAW_AMOUNT_LAMPORTS, &state, false)
        .await?;
    println!(
        "spendable SOL lamports: {}",
        plan.availability.spendable_lamports
    );
    println!(
        "reserved SOL lamports: {}",
        plan.availability.reserve_lamports
    );
    let confirmed = client
        .sign_and_submit_prepared_tx_confirmed_with_slot(plan.transaction)
        .await?;
    println!(
        "withdrew {WITHDRAW_AMOUNT_LAMPORTS} lamports to {recipient}: {} at slot {}",
        confirmed.signature, confirmed.slot
    );

    // Confirmation does not mutate cached state. First observe the wallet
    // stream at or beyond the processing slot, then replace it with a complete
    // slot-bounded REST snapshot before publishing post-transaction state.
    wait_for_wallet_state(&ws, &mut state, "post-withdraw wallet update", |state| {
        state
            .context_slot
            .is_some_and(|slot| slot >= confirmed.slot)
    })
    .await?;
    let snapshot = client
        .positions()
        .deposit_token_balances(Some(confirmed.slot))
        .await?;
    state.apply_rest_snapshot(PubkeyStr::from(wallet), &snapshot);
    println!(
        "post-withdraw native + canonical WSOL: {}",
        state.combined_sol_balance()?
    );

    // Graceful success-path teardown removes server interest before logout;
    // early errors still abort the native background task when `ws` is dropped.
    ws.send(MessageOut::unsubscribe_wallet_deposit_balances(
        PubkeyStr::from(wallet),
    ))?;
    ws.disconnect().await?;

    client.auth().logout().await?;
    Ok(())
}

/// Wait for an accepted state that satisfies one lifecycle condition.
///
/// The initial barrier accepts the first complete wallet baseline; the
/// post-transaction barrier requires an observation covering its confirmed slot.
async fn wait_for_wallet_state(
    ws: &lightcone::ws::native::WsClient,
    state: &mut WalletDepositBalancesState,
    description: &str,
    predicate: impl Fn(&WalletDepositBalancesState) -> bool,
) -> ExampleResult {
    let events = ws.events();
    tokio::pin!(events);
    tokio::time::timeout(Duration::from_secs(10), async {
        while let Some(event) = events.next().await {
            if let WsEvent::Message(Kind::WalletDepositBalances(update)) = event {
                if state.apply_event(&update) == WalletDepositBalancesApplyResult::Applied
                    && predicate(state)
                {
                    return Ok::<_, Box<dyn std::error::Error>>(());
                }
            }
        }
        Err(other(format!("wallet balance stream ended before {description}")).into())
    })
    .await
    .map_err(|_| other(format!("timed out waiting for {description}")))??;
    Ok(())
}

/// Fail closed before network or signing side effects outside built-in safe routes.
fn require_non_production() -> ExampleResult {
    // Missing configuration fails closed because this example moves funds.
    let environment = env::var("LIGHTCONE_ENV")
        .unwrap_or_else(|_| "prod".into())
        .to_lowercase();
    if !matches!(environment.as_str(), "local" | "staging") {
        return Err("SOL action examples are disabled in production".into());
    }

    // Overrides can repoint a safe environment label at production infrastructure.
    let override_name = ["SDK_API_URL", "SDK_WS_URL", "SDK_RPC_URL", "SDK_PROGRAM_ID"]
        .into_iter()
        .find(|name| env::var_os(name).is_some());
    if let Some(name) = override_name {
        return Err(format!(
            "SOL action examples require built-in local/staging configuration; unset {name}"
        )
        .into());
    }

    Ok(())
}
