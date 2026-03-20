"""Build, sign, and submit mint/merge complete set and increment nonce on-chain."""

import asyncio

from solders.pubkey import Pubkey

from common import client as make_client, wallet, market, deposit_mint, num_outcomes
from lightcone_sdk.rpc import require_connection


async def submit_transaction(name, connection, tx, keypair, blockhash):
    tx.sign([keypair], blockhash)
    print(
        f"{name}: {len(tx.message.instructions)} instruction(s), "
        f"{len(bytes(tx))} bytes, signature={tx.signatures[0]}"
    )
    result = await connection.send_raw_transaction(bytes(tx))
    await connection.confirm_transaction(result.value)
    print(f"{name}: confirmed {result.value}")


async def main():
    client = make_client()
    keypair = wallet()

    m = await market(client)
    market_pubkey = Pubkey.from_string(m.pubkey)
    d_mint = deposit_mint(m)
    outcomes = num_outcomes(m)
    amount = 1_000_000
    blockhash = await client.rpc().get_latest_blockhash()

    # Build transactions via fluent builders
    transactions = [
        (
            "mint_complete_set",
            client.markets().mint_complete_set()
                .user(keypair.pubkey())
                .market(market_pubkey)
                .mint(d_mint)
                .amount(amount)
                .num_outcomes(outcomes)
                .build_tx(),
        ),
        (
            "merge_complete_set",
            client.markets().merge_complete_set()
                .user(keypair.pubkey())
                .market(market_pubkey)
                .mint(d_mint)
                .amount(amount)
                .num_outcomes(outcomes)
                .build_tx(),
        ),
        (
            "increment_nonce",
            client.orders().increment_nonce_tx(keypair.pubkey()),
        ),
    ]

    connection = require_connection(client)
    for name, tx in transactions:
        await submit_transaction(name, connection, tx, keypair, blockhash)

    await client.close()


asyncio.run(main())
