"""Wrap 0.1 SOL, observe authority, then fully close canonical WSOL.

This fund-moving example is intentionally restricted to local and staging. A
pre-existing canonical WSOL balance is also closed; partial unwrap is unsupported.
A failure after submission does not prove rollback; inspect authoritative balances
before retrying because funds may already have moved.
"""

import asyncio
import os

from common import get_keypair, login, rest_client

from lightcone_sdk.domain.position import WRAPPED_SOL_MINT, WalletDepositBalancesState
from lightcone_sdk.shared.scaling import exact_scaled_integer
from lightcone_sdk.shared.signing import SigningStrategy
from lightcone_sdk.ws import WsEventType
from lightcone_sdk.ws.subscriptions import WalletDepositBalancesParams

WRAP_AMOUNT = "0.1"


async def main():
    require_non_production()
    client = rest_client()
    keypair = get_keypair()
    session = await login(client, keypair)
    wallet = session.user.trading_wallet(session.auth_method)

    state = WalletDepositBalancesState()
    state_changed = asyncio.Event()
    ws = client.ws()

    def on_event(event):
        # Install the reducer before subscribing so the complete baseline cannot
        # race the listener; pre-baseline component events are safely ignored.
        if (
            event.type is WsEventType.MESSAGE
            and event.message is not None
            and event.message.type == "wallet_deposit_balances"
            and state.apply_event(event.message.data).value == "applied"
        ):
            state_changed.set()

    remove_listener = ws.on(on_event)
    await ws.connect()
    params = WalletDepositBalancesParams(wallet_address=wallet)
    try:
        await ws.subscribe(params)
        await wait_for_state(
            state_changed,
            lambda: state.context_slot is not None,
            "initial wallet balance snapshot",
        )

        print(f"wallet: {wallet}")
        print(f"context slot: {state.context_slot}")
        print(f"native SOL: {state.native_sol_balance}")
        print(f"native + canonical WSOL: {state.combined_sol_balance()}")
        print(f"tracked balances: {len(state.balances)}")

        entries = sorted(state.balances.values(), key=lambda b: b.symbol)
        for balance in entries:
            print(f"  {balance.symbol:>8}  {balance.mint:<42}  idle={balance.idle}")

        client.set_signing_strategy(SigningStrategy.native(keypair))
        # Confirmation does not mutate state. Wait for authoritative WS changes
        # before using the refreshed cache to authorize the next conversion.
        expected_wsol_lamports = canonical_wsol_lamports(state) + exact_scaled_integer(
            WRAP_AMOUNT, 9
        )
        state_changed.clear()
        wrap_signature = await client.positions().wrap_sol(WRAP_AMOUNT, state)
        print(f"wrapped {WRAP_AMOUNT} SOL: {wrap_signature}")
        await wait_for_state(
            state_changed,
            lambda: canonical_wsol_lamports(state) == expected_wsol_lamports,
            "post-wrap WSOL update",
        )
        print(f"post-wrap native + canonical WSOL: {state.combined_sol_balance()}")

        print(
            "closing the full canonical WSOL account; partial unwrap is not supported"
        )
        state_changed.clear()
        unwrap_signature = await client.positions().unwrap_wsol(state)
        print(f"unwrapped full canonical WSOL account: {unwrap_signature}")
        await wait_for_state(
            state_changed,
            lambda: canonical_wsol_lamports(state) == 0,
            "post-unwrap WSOL removal",
        )
        print(f"post-unwrap native + canonical WSOL: {state.combined_sol_balance()}")

        await ws.unsubscribe(params)
    finally:
        # Disconnect is definitive teardown if an earlier error skips unsubscribe.
        remove_listener()
        await ws.disconnect()

    await client.auth().logout()
    await client.close()


async def wait_for_state(state_changed, predicate, description: str) -> None:
    """Wait boundedly for reducer state, avoiding lost event wake-ups.

    Conversion predicates compare exact canonical WSOL lamports, so native-only
    or unrelated positive updates cannot release the barrier.
    """
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


def canonical_wsol_lamports(state: WalletDepositBalancesState) -> int:
    """Return exact cached canonical WSOL lamports, treating absence as zero."""
    balance = state.balances.get(WRAPPED_SOL_MINT)
    return exact_scaled_integer(balance.idle, 9) if balance is not None else 0


def require_non_production() -> None:
    """Refuse production before login, subscription, or transaction side effects."""
    if os.environ.get("LIGHTCONE_ENV", "prod").lower() not in {"local", "staging"}:
        raise RuntimeError("SOL conversion examples are disabled in production")


asyncio.run(main())
