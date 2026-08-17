//! Fund-moving local/staging wallet-balance lifecycle example.
//!
//! The example authenticates, initializes state from the SDK-selected WebSocket
//! endpoint, wraps `0.1` SOL, waits for authoritative state, and then closes the
//! entire canonical WSOL account. The close also unwraps any pre-existing WSOL.
//! A failure after transaction submission is not a rollback signal: inspect an
//! authoritative balance before retrying because funds may already have moved.

mod common;

use common::{get_keypair, login, other, rest_client, ExampleResult};
use futures_util::StreamExt;
use lightcone::{
    prelude::{
        PubkeyStr, SigningStrategy, WalletDepositBalancesApplyResult, WalletDepositBalancesState,
    },
    shared::exact_scaled_integer,
    ws::{Kind, MessageOut, WsEvent},
};
use num_bigint::BigUint;
use std::{env, sync::Arc, time::Duration};

const WRAP_AMOUNT: &str = "0.1";

#[tokio::main]
async fn main() -> ExampleResult {
    require_non_production()?;
    let client = rest_client()?;
    let keypair = Arc::new(get_keypair()?);
    let session = login(&client, keypair.as_ref(), false).await?;
    let wallet = session.user.trading_wallet(session.auth_method);

    let mut state = WalletDepositBalancesState::default();

    // Register the wallet stream before any conversion. Component updates are
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

    // Confirmation does not mutate cached state. Wait for the authoritative
    // post-wrap observation before using that state to authorize full unwrap.
    let expected_wsol_lamports =
        canonical_wsol_lamports(&state).unwrap_or_default() + exact_scaled_integer(WRAP_AMOUNT, 9)?;
    let signature = client.positions().wrap_sol(WRAP_AMOUNT, &state).await?;
    println!("wrapped {WRAP_AMOUNT} SOL: {signature}");
    wait_for_wallet_state(&ws, &mut state, "post-wrap WSOL update", move |state| {
        canonical_wsol_lamports(state).as_ref() == Some(&expected_wsol_lamports)
    })
    .await?;
    println!(
        "post-wrap native + canonical WSOL: {}",
        state.combined_sol_balance()?
    );

    println!("closing the full canonical WSOL account; partial unwrap is not supported");
    let signature = client.positions().unwrap_wsol(&state).await?;
    println!("unwrapped full canonical WSOL account: {signature}");
    wait_for_wallet_state(&ws, &mut state, "post-unwrap WSOL removal", move |state| {
        !has_positive_wsol(state)
    })
    .await?;
    println!(
        "post-unwrap native + canonical WSOL: {}",
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
/// Conversion barriers compare exact canonical WSOL lamports. A native-only or
/// unrelated positive update cannot release them, and equal/lower slots remain valid.
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

fn has_positive_wsol(state: &WalletDepositBalancesState) -> bool {
    canonical_wsol_lamports(state).is_some_and(|idle| idle > BigUint::default())
}

fn canonical_wsol_lamports(state: &WalletDepositBalancesState) -> Option<BigUint> {
    let mint = PubkeyStr::from(spl_token_interface::native_mint::id().to_string());
    state
        .balances
        .get(&mint)
        .and_then(|balance| exact_scaled_integer(&balance.idle.to_string(), 9).ok())
}

fn require_non_production() -> ExampleResult {
    // Missing configuration fails closed because this example moves funds and
    // destructively closes the wallet's complete canonical WSOL account.
    match env::var("LIGHTCONE_ENV")
        .unwrap_or_else(|_| "prod".into())
        .to_lowercase()
        .as_str()
    {
        "local" | "staging" => Ok(()),
        _ => Err("SOL conversion examples are disabled in production".into()),
    }
}
