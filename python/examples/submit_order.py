"""Deposit the quote amount into the global pool, place a limit order, then cancel and withdraw to keep the example net-neutral."""

import asyncio

from common import (
    client as make_client,
    deposit_mint,
    get_keypair,
    login,
    market_and_orderbook,
)
from lightcone_sdk.domain.order import CancelBody
from lightcone_sdk.program.orders import generate_salt, sign_cancel_order
from lightcone_sdk.rpc import require_connection
from lightcone_sdk.shared.signing import SigningStrategy

# Quote needed for the bid below (price * size, scaled to the deposit asset's
# decimals). Keeping this as a constant so deposit, order, and withdraw stay
# in sync.
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
    # before the order can be placed.
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

    # 3. Clean up so the example is net-neutral on the wallet's token balance.
    #    Cancel the open order to release the locked collateral, then withdraw
    #    it from the global pool back to the user's token account.
    pubkey = str(keypair.pubkey())
    cancel_signature = sign_cancel_order(response.order_hash, keypair)
    cancelled = await client.orders().cancel(
        CancelBody(order_hash=response.order_hash, maker=pubkey, signature=cancel_signature)
    )
    print(f"cancelled: {cancelled.order_hash} remaining={cancelled.remaining}")

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
