"""Deposit the quote amount into the global pool, then place a limit order; cancel_order cleans it up."""

import asyncio

from common import (
    client as make_client,
    deposit_mint,
    get_keypair,
    login,
    market_and_orderbook,
)
from lightcone_sdk.program.orders import generate_salt
from lightcone_sdk.rpc import require_connection
from lightcone_sdk.shared.signing import SigningStrategy

# Quote needed for the bid below (price * size, scaled to the deposit asset's
# decimals). Must stay in sync with the same constant in cancel_order.py,
# which withdraws this amount back out of the global pool after cancelling.
ORDER_QUOTE_AMOUNT = 1_100_000  # 0.55 * 2 USDC, 6 decimals


async def main():
    keypair = get_keypair()
    client = make_client()
    client.set_signing_strategy(SigningStrategy.native(keypair))
    await login(client, keypair)

    market, orderbook = await market_and_orderbook(client)
    mint = deposit_mint(market)
    connection = require_connection(client)

    # 1. Deposit collateral into the global pool.
    #
    # submit_order uses the client's default deposit source (Global), so the
    # global pool must cover price * size in the deposit asset's base units
    # before the order can be placed. The companion cancel_order example
    # cancels this order and withdraws the same amount back to the user's
    # token account, keeping the deposit/submit/cancel/withdraw cycle
    # net-neutral across CI runs.
    deposit_ix = (
        client.positions().deposit_to_global()
        .user(keypair.pubkey())
        .mint(mint)
        .amount(ORDER_QUOTE_AMOUNT)
        .build_ix()
    )
    blockhash = await client.rpc().get_latest_blockhash()
    deposit_tx = await client.rpc().build_transaction([deposit_ix])
    deposit_tx.sign([keypair], blockhash)
    deposit_result = await connection.send_raw_transaction(bytes(deposit_tx))
    await connection.confirm_transaction(deposit_result.value)
    print(f"deposit_to_global: confirmed {deposit_result.value}")

    # 2. Submit the limit order. Fetch and cache the on-chain nonce once —
    #    subsequent orders that omit .nonce() use this cached value.
    nonce = await client.orders().current_nonce(keypair.pubkey())
    client.set_order_nonce(nonce)

    response = await (
        client.orders().limit_order()
        .maker(keypair.pubkey())
        .bid()
        .price("0.55")
        .size("2")
        .salt(generate_salt())
        .submit(client, orderbook)
    )
    print(
        f"submitted: {response.order_hash} "
        f"filled={response.filled} remaining={response.remaining} "
        f"fills={len(response.fills)}"
    )

    await client.close()


asyncio.run(main())
