"""Wrap exact SOL, then close and unwrap all canonical WSOL.

This destructive fund-moving example refuses production and unsafe endpoint
overrides. Local runs may retain a paid Solana RPC to avoid public-devnet rate
limits. It never retries submission automatically: after any uncertain outcome,
inspect the signature and authoritative wallet balances before another run.
"""

import asyncio
import os
from collections.abc import Awaitable, Callable

from common import get_keypair, login, rest_client
from solders.transaction import Transaction

from lightcone_sdk.client import ConfirmedTransaction
from lightcone_sdk.domain.position import (
    DepositTokenBalancesSnapshot,
    SolBalanceBreakdown,
    SolBalanceDelta,
    WalletDepositBalancesState,
)
from lightcone_sdk.shared.signing import SigningStrategy
from lightcone_sdk.ws import WsEventType
from lightcone_sdk.ws.subscriptions import WalletDepositBalancesParams

#: Exact native amount wrapped per manual run, in lamports (0.001 SOL).
WRAP_AMOUNT_LAMPORTS = 1_000_000


async def main() -> None:
    """Run exact wrap and unwrap-all with slot-bounded authoritative refreshes."""
    require_non_production()
    client = rest_client()
    keypair = get_keypair("LIGHTCONE_WALLET_PATH_PYTHON")
    session = await login(client, keypair)
    wallet = session.user.trading_wallet(session.auth_method)
    if wallet != str(keypair.pubkey()):
        raise RuntimeError(
            "native keypair must control the authenticated Trading Wallet"
        )

    state = WalletDepositBalancesState()
    state_changed = asyncio.Event()
    ws = client.ws()
    params = WalletDepositBalancesParams(wallet_address=wallet)
    subscribed = False

    def on_event(event) -> None:
        # Install before subscription so a complete baseline cannot race the
        # reducer. Stream updates are observation barriers only after submission;
        # a complete covering REST snapshot is the publication authority.
        if (
            event.type is WsEventType.MESSAGE
            and event.message is not None
            and event.message.type == "wallet_deposit_balances"
            and state.apply_event(event.message.data).value == "applied"
        ):
            state_changed.set()

    remove_listener = ws.on(on_event)
    try:
        await ws.connect()
        await ws.subscribe(params)
        subscribed = True
        await wait_for_state(
            state_changed,
            lambda: state.context_slot is not None,
            "initial wallet balance snapshot",
        )
        client.set_signing_strategy(SigningStrategy.native(keypair))

        # Preview is read-only and makes the exact fee, reserve, and projection
        # visible before the final message is rebuilt immediately for signing.
        wrap_preview = await client.positions().plan_wrap_sol(
            WRAP_AMOUNT_LAMPORTS, state
        )
        print(f"wallet: {wallet}")
        print(f"authoritative slot: {state.context_slot}")
        print(f"native + canonical WSOL: {state.combined_sol_balance()}")
        print(f"exact wrap amount: {WRAP_AMOUNT_LAMPORTS} lamports")
        print(f"preview wrap fee: {wrap_preview.costs.fee_lamports} lamports")
        print(
            "preview wrap upfront rent: "
            f"{wrap_preview.costs.upfront_rent_lamports} lamports"
        )

        wrap_plan = await client.positions().plan_wrap_sol(WRAP_AMOUNT_LAMPORTS, state)
        print(f"final wrap fee: {wrap_plan.costs.fee_lamports} lamports")
        print(
            f"final wrap upfront rent: {wrap_plan.costs.upfront_rent_lamports} lamports"
        )
        frozen_wrap = project_breakdown(
            wrap_plan.availability.breakdown, wrap_plan.expected_delta
        )
        wrap_confirmed = await submit_prepared_once(
            wrap_plan.transaction,
            client.sign_and_submit_prepared_tx_confirmed_with_slot,
        )
        print(
            f"wrap submitted: {wrap_confirmed.signature} at slot {wrap_confirmed.slot}"
        )
        print(f"frozen wrap projection: {frozen_wrap}")
        await refresh_covering(
            client.positions().deposit_token_balances,
            state,
            wallet,
            wrap_confirmed.slot,
        )
        print(f"covered post-wrap balance: {state.combined_sol_balance()}")

        # Build once against covering state, display that exact plan, then submit
        # its prepared message without an interactive pause or stale preview.
        unwrap_plan = await client.positions().plan_unwrap_wsol_all(state)
        full_account_return = (
            unwrap_plan.expected_delta.native_lamports + unwrap_plan.costs.fee_lamports
        )
        print(f"unwrap-all fee: {unwrap_plan.costs.fee_lamports} lamports")
        print(f"full canonical account return: {full_account_return} lamports")
        print(
            "WARNING: unwrap-all closes this wallet's canonical WSOL account and "
            "returns its complete lamport balance, including rent and any extra "
            "lamports. A later ordinary action may need to recreate the account."
        )

        # No pause or cached plan crosses this boundary. Preserve and submit that
        # exact prepared message.
        frozen_unwrap = project_breakdown(
            unwrap_plan.availability.breakdown, unwrap_plan.expected_delta
        )
        unwrap_confirmed = await submit_prepared_once(
            unwrap_plan.transaction,
            client.sign_and_submit_prepared_tx_confirmed_with_slot,
        )
        print(
            f"unwrap-all submitted: {unwrap_confirmed.signature} at slot "
            f"{unwrap_confirmed.slot}"
        )
        print(f"frozen unwrap projection: {frozen_unwrap}")
        await refresh_covering(
            client.positions().deposit_token_balances,
            state,
            wallet,
            unwrap_confirmed.slot,
        )
        print(f"covered post-unwrap balance: {state.combined_sol_balance()}")
    finally:
        if subscribed:
            await ws.unsubscribe(params)
        remove_listener()
        await ws.disconnect()
        await client.close()


def project_breakdown(
    breakdown: SolBalanceBreakdown, delta: SolBalanceDelta
) -> SolBalanceBreakdown:
    """Freeze a plan projection without mutation, rejecting negative balances."""
    native_lamports = breakdown.native_lamports + delta.native_lamports
    canonical_wsol_lamports = (
        breakdown.canonical_wsol_lamports + delta.canonical_wsol_lamports
    )
    if native_lamports < 0 or canonical_wsol_lamports < 0:
        raise ValueError("planner produced a negative frozen SOL projection")
    return SolBalanceBreakdown(native_lamports, canonical_wsol_lamports)


async def submit_prepared_once(
    transaction: Transaction,
    submit: Callable[[Transaction], Awaitable[ConfirmedTransaction]],
) -> ConfirmedTransaction:
    """Submit one prepared message exactly once and propagate uncertain failure."""
    return await submit(transaction)


async def refresh_covering(
    fetch_snapshot: Callable[[int], Awaitable[DepositTokenBalancesSnapshot]],
    state: WalletDepositBalancesState,
    wallet: str,
    confirmed_slot: int,
) -> None:
    """Install a complete covering REST snapshot without gating on the stream."""
    snapshot = await fetch_snapshot(confirmed_slot)
    validate_covering_snapshot_slot(snapshot.context_slot, confirmed_slot)
    state.apply_rest_snapshot(wallet, snapshot)


def validate_covering_snapshot_slot(snapshot_slot: int, confirmed_slot: int) -> None:
    """Reject a REST snapshot that cannot restore post-confirmation authority."""
    if snapshot_slot < confirmed_slot:
        raise RuntimeError("REST snapshot did not cover the confirmed transaction slot")


async def wait_for_state(state_changed, predicate, description: str) -> None:
    """Wait up to ten seconds; expiration raises ``TimeoutError``."""
    deadline = asyncio.get_running_loop().time() + 10
    while not predicate():
        state_changed.clear()
        if predicate():
            return
        remaining = deadline - asyncio.get_running_loop().time()
        if remaining <= 0:
            raise TimeoutError(f"timed out waiting for {description}")
        try:
            await asyncio.wait_for(state_changed.wait(), timeout=remaining)
        except TimeoutError as error:
            raise TimeoutError(f"timed out waiting for {description}") from error


def require_non_production() -> None:
    """Reject production and unsafe endpoint overrides before fund-moving work."""
    environment = os.environ.get("LIGHTCONE_ENV", "prod").lower()
    if environment not in {"local", "staging"}:
        raise RuntimeError("WSOL conversion examples are disabled in production")
    ci = "CI" in os.environ
    override_name = next(
        (
            name
            for name in ("SDK_API_URL", "SDK_WS_URL", "SDK_RPC_URL", "SDK_PROGRAM_ID")
            if name in os.environ
            and not (
                (environment == "local" and name == "SDK_RPC_URL")
                or (
                    environment == "staging"
                    and ci
                    and name in {"SDK_API_URL", "SDK_WS_URL", "SDK_RPC_URL"}
                )
            )
        ),
        None,
    )
    if override_name is not None:
        raise RuntimeError(
            "WSOL conversion examples require built-in API, WebSocket, and program "
            "configuration; "
            f"unset {override_name}"
        )


if __name__ == "__main__":
    asyncio.run(main())
