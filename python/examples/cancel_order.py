"""Cancel a single order by hash, cancel all orders in an orderbook, and withdraw the released collateral back from the global pool."""

import asyncio

from common import (
    client as make_client,
    deposit_mint,
    get_keypair,
    login,
    market,
    unix_timestamp,
)
from lightcone_sdk.domain.order import CancelBody, CancelAllBody
from lightcone_sdk.program.orders import (
    generate_cancel_all_salt,
    sign_cancel_order,
    sign_cancel_all,
)
from lightcone_sdk.rpc import require_connection

# Mirrors the constant in submit_order.py. When we cancel the order that
# example left open, we withdraw the same quote amount back from the global
# pool so the deposit/submit/cancel/withdraw cycle is net-neutral.
ORDER_QUOTE_AMOUNT = 1_100_000  # 0.55 * 2 USDC, 6 decimals


async def main():
    client = make_client()
    keypair = get_keypair()
    await login(client, keypair)
    pubkey = str(keypair.pubkey())

    # 1. Find an open limit order
    snapshot = await client.orders().get_user_orders(50)
    limit_order = next(
        (o for o in snapshot.orders if o.order_type == "limit"), None
    )

    if limit_order is None:
        print("No open limit orders to cancel.")
        await client.close()
        return

    order_hash = limit_order.order_hash
    orderbook_id = limit_order.orderbook_id

    # 2. Cancel a single order
    signature = sign_cancel_order(order_hash, keypair)
    cancelled = await client.orders().cancel(
        CancelBody(order_hash=order_hash, maker=pubkey, signature=signature)
    )
    print(f"cancelled: {cancelled.order_hash} remaining={cancelled.remaining}")

    # 3. Cancel all orders in an orderbook
    timestamp = unix_timestamp()
    salt = generate_cancel_all_salt()
    cancel_all_sig = sign_cancel_all(pubkey, orderbook_id, timestamp, salt, keypair)
    cleared = await client.orders().cancel_all(
        CancelAllBody(
            user_pubkey=pubkey,
            orderbook_id=orderbook_id,
            signature=cancel_all_sig,
            timestamp=timestamp,
            salt=salt,
        )
    )
    print(f"cancel-all removed {cleared.count} order(s) in {cleared.orderbook_id}")

    # 4. Cleanup: cancelling the order released its locked collateral back into
    #    the global pool. Withdraw that amount to the user's token account so
    #    the companion submit_order → cancel_order cycle is net-neutral on the
    #    wallet's balance and the global pool.
    m = await market(client)
    mint = deposit_mint(m)
    connection = require_connection(client)
    withdraw_ix = (
        client.positions().withdraw_from_global()
        .user(keypair.pubkey())
        .mint(mint)
        .amount(ORDER_QUOTE_AMOUNT)
        .build_ix()
    )
    blockhash = await client.rpc().get_latest_blockhash()
    withdraw_tx = await client.rpc().build_transaction([withdraw_ix])
    withdraw_tx.sign([keypair], blockhash)
    withdraw_result = await connection.send_raw_transaction(bytes(withdraw_tx))
    await connection.confirm_transaction(withdraw_result.value)
    print(f"withdraw_from_global: confirmed {withdraw_result.value}")

    await client.close()


asyncio.run(main())
