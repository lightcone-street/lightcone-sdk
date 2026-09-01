"""Plan and confirm a native SOL withdrawal without closing canonical WSOL.

This fund-moving example is intentionally restricted to local and staging. It
refreshes a complete wallet snapshot covering the confirmation slot before
publishing the post-action balance.
"""

import asyncio
import os

from common import get_keypair, login, rest_client

from lightcone_sdk.domain.position import WalletDepositBalancesState
from lightcone_sdk.shared.signing import SigningStrategy
from lightcone_sdk.ws import WsEventType
from lightcone_sdk.ws.subscriptions import WalletDepositBalancesParams

#: Native SOL transferred per run, in lamports (0.001 SOL).
WITHDRAW_AMOUNT_LAMPORTS = 1_000_000


async def main():
    """Run the fund-moving lifecycle against configured non-production wallets."""
    require_non_production()
    # SDK wallets form a stable funding cycle: Rust -> TypeScript -> Python -> Rust.
    # The existing peer path avoids a recipient-specific setting and repeated top-offs.
    recipient = get_keypair("LIGHTCONE_WALLET_PATH").pubkey()
    client = rest_client()
    keypair = get_keypair("LIGHTCONE_WALLET_PATH_PYTHON")
    if recipient == keypair.pubkey():
        raise RuntimeError("Python and Rust SDK wallet paths must identify peers")
    session = await login(client, keypair)
    wallet = session.user.trading_wallet(session.auth_method)

    state = WalletDepositBalancesState()
    state_changed = asyncio.Event()
    ws = client.ws()

    def on_event(event):
        # Install the reducer before subscribing so the complete baseline cannot
        # race the listener; pre-baseline balance events are safely ignored.
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
        plan = await client.positions().plan_native_sol_withdrawal(
            recipient,
            WITHDRAW_AMOUNT_LAMPORTS,
            state,
            False,
        )
        print(f"spendable SOL lamports: {plan.availability.spendable_lamports}")
        print(f"reserved SOL lamports: {plan.availability.reserve_lamports}")
        confirmed = await client.sign_and_submit_prepared_tx_confirmed_with_slot(
            plan.transaction
        )
        print(
            f"withdrew {WITHDRAW_AMOUNT_LAMPORTS} lamports to {recipient}: "
            f"{confirmed.signature} at slot {confirmed.slot}"
        )
        # Confirmation does not mutate cached state. Observe the wallet stream at
        # or beyond the processing slot, then replace it with a complete slot-bounded
        # REST snapshot before publishing post-transaction state.
        await wait_for_state(
            state_changed,
            lambda: state.context_slot is not None
            and state.context_slot >= confirmed.slot,
            "post-withdraw wallet update",
        )
        snapshot = await client.positions().deposit_token_balances(confirmed.slot)
        state.apply_rest_snapshot(wallet, snapshot)
        print(
            "post-withdraw native + canonical WSOL: " f"{state.combined_sol_balance()}"
        )

        await ws.unsubscribe(params)
    finally:
        # Disconnect is definitive teardown if an earlier error skips unsubscribe.
        remove_listener()
        await ws.disconnect()

    await client.auth().logout()
    await client.close()


async def wait_for_state(state_changed, predicate, description: str) -> None:
    """Wait boundedly for reducer state, avoiding lost event wake-ups.

    The initial barrier accepts the first complete wallet baseline; the
    post-transaction barrier requires an observation covering its confirmed slot.
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


def require_non_production() -> None:
    """Refuse production before login, subscription, or transaction side effects."""
    if os.environ.get("LIGHTCONE_ENV", "prod").lower() not in {"local", "staging"}:
        raise RuntimeError("SOL action examples are disabled in production")

    # Overrides can repoint a safe environment label at production infrastructure.
    override_name = next(
        (
            name
            for name in ("SDK_API_URL", "SDK_WS_URL", "SDK_RPC_URL", "SDK_PROGRAM_ID")
            if name in os.environ
        ),
        None,
    )
    if override_name is not None:
        raise RuntimeError(
            "SOL action examples require built-in local/staging configuration; "
            f"unset {override_name}"
        )


asyncio.run(main())
