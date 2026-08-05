"""Cancel a single order by hash, cancel all orders in an orderbook, and withdraw the released collateral back from the global pool."""

import asyncio

from common import (
    client as make_client,
)
from common import (
    get_keypair,
    login,
    market_and_orderbook,
    quote_deposit_mint,
    unix_timestamp,
)

from lightcone_sdk.domain.order import CancelAllBody, CancelBody
from lightcone_sdk.program.orders import (
    generate_cancel_all_salt,
    sign_cancel_all,
    sign_cancel_order,
)
from lightcone_sdk.program.types import OrderSide
from lightcone_sdk.rpc import require_connection
from lightcone_sdk.shared.scaling import scale_price_size


async def main():
    client = make_client()
    keypair = get_keypair()
    await login(client, keypair)
    pubkey = str(keypair.pubkey())

    # 1. Find an open limit order
    snapshot = await client.orders().get_user_orders(50)
    limit_order = next((o for o in snapshot.orders if o.order_type == "limit"), None)

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
    _, orderbook = await market_and_orderbook(client)
    rules = await client.orderbooks().decimals(orderbook.orderbook_id)
    order_quote_amount = scale_price_size(
        rules.trading_rules.price_quantum, "1", int(OrderSide.BID), rules
    ).quote_atoms
    mint = quote_deposit_mint(orderbook)
    connection = require_connection(client)
    withdraw_ix = (
        client.positions()
        .withdraw_from_global()
        .user(keypair.pubkey())
        .mint(mint)
        .amount(order_quote_amount)
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
