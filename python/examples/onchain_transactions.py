"""Build, sign, and submit deposit, merge, and increment nonce on-chain."""

import asyncio

from common import client as make_client
from common import get_keypair, market_and_orderbook, quote_deposit_mint

from lightcone_sdk.shared.signing import SigningStrategy


async def submit_transaction(name, client, tx):
    confirmed = await client.sign_and_submit_tx_confirmed_with_slot(tx)
    print(f"{name}: confirmed {confirmed.signature} at slot {confirmed.slot}")


async def main():
    client = make_client()
    keypair = get_keypair()
    client.signing_strategy = SigningStrategy.native(keypair)

    m, ob = await market_and_orderbook(client)
    d_mint = quote_deposit_mint(ob)
    amount = 1_000_000
    context = await client.transaction_context()

    # Build transactions via fluent builders
    transactions = [
        (
            "deposit",
            client.positions()
            .deposit()
            .user(keypair.pubkey())
            .mint(d_mint)
            .amount(amount)
            .with_market_deposit_source(m)
            .build_tx(context),
        ),
        (
            "merge",
            client.positions()
            .merge()
            .user(keypair.pubkey())
            .market(m)
            .mint(d_mint)
            .amount(amount)
            .build_tx(context),
        ),
        (
            "increment_nonce",
            client.orders().increment_nonce_tx(keypair.pubkey(), context),
        ),
    ]

    for index, (name, tx) in enumerate(transactions):
        if index > 0:
            await asyncio.sleep(1)  # avoid devnet RPC rate limits
        await submit_transaction(name, client, tx)

    await client.close()


asyncio.run(main())
