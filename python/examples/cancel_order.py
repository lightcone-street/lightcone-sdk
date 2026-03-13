"""Cancel a single order by hash and cancel all orders in an orderbook."""

import asyncio

from common import rest_client, wallet, login, unix_timestamp
from lightcone_sdk.domain.order import CancelBody, CancelAllBody
from lightcone_sdk.program.orders import (
    generate_cancel_all_salt,
    sign_cancel_order,
    sign_cancel_all,
)


async def main():
    client = rest_client()
    keypair = wallet()
    await login(client, keypair)
    pubkey = str(keypair.pubkey())

    # 1. Find an open limit order
    snapshot = await client.orders().get_user_orders(pubkey, 50)
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

    await client.close()


asyncio.run(main())
