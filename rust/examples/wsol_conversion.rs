//! Fund-moving canonical-WSOL conversion for local or staging wallets.
//!
//! The example wraps an exact small amount, keeps each confirmed projection
//! frozen until a complete covering snapshot arrives, then rebuilds and submits
//! unwrap-all without an interactive pause. Unwrap-all closes the Trading
//! Wallet's canonical WSOL account and returns every account lamport. Any
//! submission or confirmation error exits without retry because the transaction
//! may already have landed.

mod common;

use common::{get_keypair, login, other, rest_client, ExampleResult};
use futures_util::StreamExt;
use lightcone::{
    prelude::{
        PubkeyStr, SigningStrategy, SolBalanceComponents, SolComponentDelta,
        WalletDepositBalancesApplyResult, WalletDepositBalancesState,
    },
    ws::{Kind, MessageOut, WsEvent},
};
use solana_pubkey::Pubkey;
use solana_signer::Signer;
use std::{env, future::Future, sync::Arc, time::Duration};

/// Exact native SOL wrapped per run, in lamports (0.001 SOL).
const WRAP_AMOUNT_LAMPORTS: u64 = 1_000_000;

/// Endpoint and program overrides checked before this fund-moving example.
const ENDPOINT_OVERRIDE_NAMES: [&str; 4] =
    ["SDK_API_URL", "SDK_WS_URL", "SDK_RPC_URL", "SDK_PROGRAM_ID"];

/// Run one exact wrap followed by a destructive full canonical-account close.
#[tokio::main]
async fn main() -> ExampleResult {
    require_non_production()?;
    let client = rest_client()?;
    let keypair = Arc::new(get_keypair()?);
    let session = login(&client, keypair.as_ref(), false).await?;
    let wallet = session
        .user
        .trading_wallet(session.auth_method)
        .parse::<Pubkey>()?;
    if wallet != keypair.pubkey() {
        return Err(other(
            "the authenticated Trading Wallet must equal the configured native keypair",
        )
        .into());
    }
    client
        .set_signing_strategy(SigningStrategy::Native(keypair))
        .await;

    let mut state = WalletDepositBalancesState::default();
    let mut ws = client.ws_native();
    ws.connect().await?;
    ws.send(MessageOut::subscribe_wallet_deposit_balances(
        PubkeyStr::from(wallet.to_string()),
    ))?;
    wait_for_wallet_state(&ws, &mut state, "initial snapshot", |_| true).await?;

    println!("wallet: {wallet}");
    print_balance("initial", &state)?;

    // The preview is informational. Rebuild immediately before signing so live
    // account, rent, blockhash, and fee reads remain the execution authority.
    let wrap_preview = client
        .positions()
        .plan_wrap_sol(WRAP_AMOUNT_LAMPORTS, &state)
        .await?;
    println!("wrap amount: {WRAP_AMOUNT_LAMPORTS} lamports");
    println!(
        "preview wrap fee: {} lamports",
        wrap_preview.costs.fee_lamports
    );
    println!(
        "preview wrap upfront account rent: {} lamports",
        wrap_preview.costs.upfront_rent_lamports
    );
    let wrap_plan = client
        .positions()
        .plan_wrap_sol(WRAP_AMOUNT_LAMPORTS, &state)
        .await?;
    println!("final wrap fee: {} lamports", wrap_plan.costs.fee_lamports);
    println!(
        "final wrap upfront account rent: {} lamports",
        wrap_plan.costs.upfront_rent_lamports
    );
    let wrap_projection =
        projected_components(wrap_plan.availability.components, wrap_plan.expected_delta)?;
    let wrap_confirmation = submit_prepared_once(wrap_plan.transaction, |transaction| {
        client.sign_and_submit_prepared_tx_confirmed_with_slot(transaction)
    })
    .await?;
    println!(
        "wrapped at slot {}: {}",
        wrap_confirmation.slot, wrap_confirmation.signature
    );
    println!(
        "frozen wrap projection: native={} canonical={}",
        wrap_projection.native_lamports, wrap_projection.canonical_wsol_lamports
    );
    refresh_covering_state(
        &client,
        &ws,
        &mut state,
        wallet,
        wrap_confirmation.slot,
        "wrap",
    )
    .await?;
    print_balance("post-wrap covering snapshot", &state)?;

    // Build once against the covering state, display that exact plan, then submit
    // its prepared message without an interactive pause or stale preview.
    let unwrap_plan = client.positions().plan_unwrap_wsol_all(&state).await?;
    let full_account_return_lamports =
        unwrap_account_return_lamports(unwrap_plan.expected_delta, unwrap_plan.costs.fee_lamports)?;
    println!(
        "unwrap-all fee: {} lamports",
        unwrap_plan.costs.fee_lamports
    );
    println!(
        "WARNING: unwrap-all closes the canonical WSOL account and returns all {full_account_return_lamports} account lamports, including rent and any excess, to {wallet}."
    );
    println!(
        "WARNING: ordinary actions never close this account, but a later action may recreate it and require rent again."
    );

    // There is intentionally no prompt or delay between final rebuild and
    // submission. An error exits; this example never retries a destructive close.
    let unwrap_projection = projected_components(
        unwrap_plan.availability.components,
        unwrap_plan.expected_delta,
    )?;
    let unwrap_confirmation = submit_prepared_once(unwrap_plan.transaction, |transaction| {
        client.sign_and_submit_prepared_tx_confirmed_with_slot(transaction)
    })
    .await?;
    println!(
        "unwrapped all at slot {}: {}",
        unwrap_confirmation.slot, unwrap_confirmation.signature
    );
    println!(
        "frozen unwrap projection: native={} canonical={}",
        unwrap_projection.native_lamports, unwrap_projection.canonical_wsol_lamports
    );
    refresh_covering_state(
        &client,
        &ws,
        &mut state,
        wallet,
        unwrap_confirmation.slot,
        "unwrap-all",
    )
    .await?;
    print_balance("post-unwrap covering snapshot", &state)?;

    ws.send(MessageOut::unsubscribe_wallet_deposit_balances(
        PubkeyStr::from(wallet.to_string()),
    ))?;
    ws.disconnect().await?;
    client.auth().logout().await?;
    Ok(())
}

/// Print only complete snapshot-backed balance state at lifecycle boundaries.
fn print_balance(label: &str, state: &WalletDepositBalancesState) -> ExampleResult {
    println!(
        "{label} slot: {}",
        state
            .context_slot
            .ok_or_else(|| other("snapshot omitted context slot"))?
    );
    println!(
        "{label} native + canonical WSOL: {}",
        state.combined_sol_balance()?
    );
    Ok(())
}

/// Apply a signed component delta, erroring on `i128` overflow or a result outside `u64`.
fn projected_components(
    components: SolBalanceComponents,
    delta: SolComponentDelta,
) -> ExampleResult<SolBalanceComponents> {
    /// Apply one signed delta while preserving the component's `u64` boundary.
    fn apply(value: u64, delta: i128) -> ExampleResult<u64> {
        let projected = i128::from(value)
            .checked_add(delta)
            .ok_or_else(|| other("projected SOL component overflowed i128"))?;
        Ok(u64::try_from(projected)
            .map_err(|_| other("projected SOL component left the u64 range"))?)
    }

    Ok(SolBalanceComponents {
        native_lamports: apply(components.native_lamports, delta.native_lamports)?,
        canonical_wsol_lamports: apply(
            components.canonical_wsol_lamports,
            delta.canonical_wsol_lamports,
        )?,
    })
}

/// Recover the preview's full account return, erroring on signed or `u64` overflow.
fn unwrap_account_return_lamports(
    expected_delta: SolComponentDelta,
    fee_lamports: u64,
) -> ExampleResult<u64> {
    let account_lamports = expected_delta
        .native_lamports
        .checked_add(i128::from(fee_lamports))
        .ok_or_else(|| other("unwrap-all account return overflowed i128"))?;
    Ok(u64::try_from(account_lamports)
        .map_err(|_| other("unwrap-all account return left the u64 range"))?)
}

/// Submit one prepared message exactly once and propagate any uncertain failure.
async fn submit_prepared_once<T, C, E, Submit, Submission>(
    transaction: T,
    submit: Submit,
) -> Result<C, E>
where
    Submit: FnOnce(T) -> Submission,
    Submission: Future<Output = Result<C, E>>,
{
    submit(transaction).await
}

/// Replace stream observations with REST state, rejecting snapshots below the confirmed slot.
async fn refresh_covering_state(
    client: &lightcone::prelude::LightconeClient,
    ws: &lightcone::ws::native::WsClient,
    state: &mut WalletDepositBalancesState,
    wallet: Pubkey,
    confirmed_slot: u64,
    action: &str,
) -> ExampleResult {
    wait_for_wallet_state(ws, state, action, |state| {
        state
            .context_slot
            .is_some_and(|slot| slot >= confirmed_slot)
    })
    .await?;
    let snapshot = client
        .positions()
        .deposit_token_balances(Some(confirmed_slot))
        .await?;
    validate_covering_snapshot_slot(snapshot.context_slot, confirmed_slot)?;
    state.apply_rest_snapshot(PubkeyStr::from(wallet.to_string()), &snapshot);
    Ok(())
}

/// Wait for a lifecycle barrier, erroring if the stream ends or ten seconds elapse first.
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

/// Reject production and unsafe endpoint overrides before any fund-moving work.
fn require_non_production() -> ExampleResult {
    let environment = env::var("LIGHTCONE_ENV")
        .unwrap_or_else(|_| "prod".into())
        .to_lowercase();
    let configured_overrides: Vec<_> = ENDPOINT_OVERRIDE_NAMES
        .into_iter()
        .filter(|name| env::var_os(name).is_some())
        .collect();
    validate_example_configuration(
        &environment,
        env::var_os("CI").is_some(),
        &configured_overrides,
    )
}

/// Validate environment and endpoint safety, naming the first rejected override.
fn validate_example_configuration(
    environment: &str,
    ci: bool,
    configured_overrides: &[&str],
) -> ExampleResult {
    if !matches!(environment, "local" | "staging") {
        return Err("WSOL conversion example is disabled in production".into());
    }

    if let Some(name) = configured_overrides.iter().find(|name| {
        let allowed_local_rpc = environment == "local" && **name == "SDK_RPC_URL";
        let allowed_staging_ci_endpoint = environment == "staging"
            && ci
            && matches!(**name, "SDK_API_URL" | "SDK_WS_URL" | "SDK_RPC_URL");
        !(allowed_local_rpc || allowed_staging_ci_endpoint)
    }) {
        return Err(format!(
            "WSOL conversion requires built-in API, WebSocket, and program configuration; unset {name}"
        )
        .into());
    }
    Ok(())
}

/// Require a complete REST snapshot to cover the transaction's confirmed slot.
fn validate_covering_snapshot_slot(snapshot_slot: u64, confirmed_slot: u64) -> ExampleResult {
    if snapshot_slot < confirmed_slot {
        return Err(format!(
            "REST snapshot slot {snapshot_slot} does not cover confirmed slot {confirmed_slot}"
        )
        .into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn production_configuration_is_rejected() {
        let error = validate_example_configuration("prod", false, &[]).unwrap_err();
        assert!(error.to_string().contains("disabled in production"));
        let error = validate_example_configuration("prod", true, &["SDK_RPC_URL"]).unwrap_err();
        assert!(error.to_string().contains("disabled in production"));
    }

    #[test]
    fn every_endpoint_override_is_rejected() {
        for name in ENDPOINT_OVERRIDE_NAMES {
            let error = validate_example_configuration("staging", false, &[name]).unwrap_err();
            assert!(error.to_string().contains(name), "{error}");
        }
    }

    #[test]
    fn built_in_non_production_configuration_is_accepted() -> ExampleResult {
        validate_example_configuration("local", false, &[])?;
        validate_example_configuration("staging", false, &[])?;
        Ok(())
    }

    #[test]
    fn local_accepts_paid_rpc_but_not_other_overrides() -> ExampleResult {
        validate_example_configuration("local", false, &["SDK_RPC_URL"])?;
        for name in ["SDK_API_URL", "SDK_WS_URL", "SDK_PROGRAM_ID"] {
            let error = validate_example_configuration("local", false, &[name]).unwrap_err();
            assert!(error.to_string().contains(name), "{error}");
        }
        Ok(())
    }

    #[test]
    fn staging_ci_accepts_workflow_endpoints_but_not_program_override() -> ExampleResult {
        validate_example_configuration(
            "staging",
            true,
            &["SDK_API_URL", "SDK_WS_URL", "SDK_RPC_URL"],
        )?;
        let error =
            validate_example_configuration("staging", true, &["SDK_PROGRAM_ID"]).unwrap_err();
        assert!(error.to_string().contains("SDK_PROGRAM_ID"), "{error}");
        Ok(())
    }

    #[test]
    fn rest_snapshot_must_cover_confirmed_slot() -> ExampleResult {
        validate_covering_snapshot_slot(10, 10)?;
        validate_covering_snapshot_slot(11, 10)?;
        let error = validate_covering_snapshot_slot(9, 10).unwrap_err();
        assert!(error.to_string().contains("does not cover confirmed slot"));
        Ok(())
    }

    #[tokio::test]
    async fn uncertain_submission_is_not_retried() {
        let mut attempts = 0;
        let error = submit_prepared_once((), |_| {
            attempts += 1;
            std::future::ready(Err::<(), _>("uncertain confirmation"))
        })
        .await
        .unwrap_err();

        assert_eq!(error, "uncertain confirmation");
        assert_eq!(attempts, 1);
    }

    #[test]
    fn unwrap_warning_return_uses_preview_delta_and_fee() -> ExampleResult {
        let returned = unwrap_account_return_lamports(
            SolComponentDelta {
                native_lamports: 2_034_280,
                canonical_wsol_lamports: -1,
            },
            5_000,
        )?;
        assert_eq!(returned, 2_039_280);
        Ok(())
    }
}
