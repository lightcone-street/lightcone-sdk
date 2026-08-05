"""Deposit the quote amount into the global pool, then place a limit order; cancel_order cleans it up."""

import asyncio

from common import (
    client as make_client,
)
from common import (
    get_keypair,
    login,
    market_and_orderbook,
    quote_deposit_mint,
    wait_for_global_balance,
)

from lightcone_sdk.program.orders import generate_salt
from lightcone_sdk.program.types import OrderSide
from lightcone_sdk.rpc import require_connection
from lightcone_sdk.shared.scaling import scale_price_size
from lightcone_sdk.shared.signing import SigningStrategy


async def main():
    keypair = get_keypair()
    client = make_client()
    client.set_signing_strategy(SigningStrategy.native(keypair))
    await login(client, keypair)

    market, orderbook = await market_and_orderbook(client)
    rules = await client.orderbooks().decimals(orderbook.orderbook_id)
    order_price = rules.trading_rules.price_quantum
    order_size = "1"
    order_quote_amount = scale_price_size(
        order_price, order_size, int(OrderSide.BID), rules
    ).quote_atoms
    required_balance = order_quote_amount / 10**rules.quote_decimals
    mint = quote_deposit_mint(orderbook)
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
        client.positions()
        .deposit_to_global()
        .user(keypair.pubkey())
        .mint(mint)
        .amount(order_quote_amount)
        .build_ix()
    )
    blockhash = await client.rpc().get_latest_blockhash()
    deposit_tx = await client.rpc().build_transaction([deposit_ix])
    deposit_tx.sign([keypair], blockhash)
    deposit_result = await connection.send_raw_transaction(bytes(deposit_tx))
    await connection.confirm_transaction(deposit_result.value)
    print(f"deposit_to_global: confirmed {deposit_result.value}")

    await wait_for_global_balance(client, mint, required_balance)

    # 2. Submit the limit order. Fetch and cache the on-chain nonce once —
    #    subsequent orders that omit .nonce() use this cached value.
    nonce = await client.orders().current_nonce(keypair.pubkey())
    client.set_order_nonce(nonce)

    response = await (
        client.orders()
        .limit_order()
        .maker(keypair.pubkey())
        .bid()
        .price(order_price)
        .size(order_size)
        .salt(generate_salt())
        .submit(client, orderbook)
    )
    print(
        f"submitted: {response.order_hash} "
        f"status={response.status.value} filled={response.filled} remaining={response.remaining} "
        f"fills={len(response.fills)}"
    )

    await client.close()


asyncio.run(main())
