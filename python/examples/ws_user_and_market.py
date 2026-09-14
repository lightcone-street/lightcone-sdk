"""Read-only authenticated user, market, and Trading Wallet balance probe."""

import asyncio

from common import get_keypair, login, market, rest_client

from lightcone_sdk.domain.position import (
    WalletDepositBalancesApplyResult,
    WalletDepositBalanceSnapshot,
    WalletDepositBalancesState,
)
from lightcone_sdk.ws import MessageInType, WsEvent, WsEventType
from lightcone_sdk.ws.subscriptions import (
    MarketParams,
    SubscribeParams,
    UserParams,
    WalletDepositBalancesParams,
)


async def main() -> None:
    client = rest_client()
    keypair = get_keypair()
    session = await login(client, keypair)
    wallet = session.user.trading_wallet(session.auth_method)
    m = await market(client)
    ws = client.ws()
    state = WalletDepositBalancesState()
    stream_error: RuntimeError | None = None
    done = asyncio.Event()

    def on_event(event: WsEvent) -> None:
        nonlocal stream_error
        if event.type == WsEventType.MESSAGE and event.message:
            msg = event.message
            if msg.type == MessageInType.WALLET_DEPOSIT_BALANCES.value:
                update = msg.data
                if (
                    isinstance(update, WalletDepositBalanceSnapshot)
                    and state.apply_event(update)
                    is WalletDepositBalancesApplyResult.APPLIED
                ):
                    done.set()
            elif msg.type == MessageInType.ERROR.value:
                stream_error = RuntimeError(str(msg.data))
                done.set()
        elif event.type == WsEventType.ERROR:
            stream_error = RuntimeError(event.error or "WebSocket transport error")
            done.set()
        elif event.type == WsEventType.MAX_RECONNECT_REACHED:
            stream_error = RuntimeError("WebSocket reconnect attempts exhausted")
            done.set()

    remove_listener = ws.on(on_event)
    subscriptions: list[SubscribeParams] = [
        UserParams(wallet_address=wallet),
        MarketParams(market_pubkey=m.pubkey),
        WalletDepositBalancesParams(wallet_address=wallet),
    ]

    try:
        await ws.connect()
        for subscription in subscriptions:
            await ws.subscribe(subscription)
        await asyncio.wait_for(done.wait(), timeout=30)
        if stream_error is not None:
            raise stream_error
        if state.context_slot is None:
            raise RuntimeError("complete snapshot did not establish a slot")
        print(f"wallet={wallet} slot={state.context_slot} count={len(state.balances)}")
    finally:
        for subscription in subscriptions:
            await ws.unsubscribe(subscription)
        remove_listener()
        await ws.disconnect()
        await client.close()


asyncio.run(main())
