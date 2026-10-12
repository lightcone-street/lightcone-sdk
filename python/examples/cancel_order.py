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
from lightcone_sdk.shared.scaling import scale_price_size
from lightcone_sdk.shared.signing import SigningStrategy


async def main():
    client = make_client()
    keypair = get_keypair()
    client.set_signing_strategy(SigningStrategy.native(keypair))
    await login(client, keypair)
    pubkey = str(keypair.pubkey())

    # 1. Find an open order
    page = await client.orders().get_user_orders(50)
    limit_order = next(iter(page.orders), None)

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
    quantities = cancelled.quantities
    print(
        f"cancel: {cancelled.order_hash} {cancelled.status.value} "
        f"revision={cancelled.revision} "
        f"newly_cancelled_base_atoms="
        f"{quantities.newly_cancelled_base if quantities else 'n/a'}"
    )

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
    if cleared.closure is not None:
        print(
            f"cancel-all cutoff committed at revision {cleared.closure.committed_revision} "
            f"(accepted_seq <= {cleared.closure.accepted_seq_cutoff}) in {cleared.orderbook_id}"
        )
    else:
        print(f"cancel-all: {cleared.message}")

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
    withdraw_ix = (
        client.positions()
        .withdraw_from_global()
        .user(keypair.pubkey())
        .mint(mint)
        .amount(order_quote_amount)
        .build_ix()
    )
    withdraw_tx = await client.rpc().build_transaction(
        [withdraw_ix], keypair.pubkey(), await client.transaction_context()
    )
    withdraw_result = await client.sign_and_submit_tx_confirmed_with_slot(withdraw_tx)
    print(f"withdraw_from_global: confirmed {withdraw_result.signature}")

    await client.close()


asyncio.run(main())
