"""Build, sign, and submit mint/merge complete set and increment nonce on-chain."""

import asyncio

from solders.pubkey import Pubkey

from common import rest_client, rpc_client, wallet, market, deposit_mint, num_outcomes
from src.program.types import MintCompleteSetParams, MergeCompleteSetParams


async def submit_transaction(name, rpc, tx, keypair, blockhash):
    tx.sign([keypair], blockhash)
    print(
        f"{name}: {len(tx.message.instructions)} instruction(s), "
        f"{len(bytes(tx))} bytes, signature={tx.signatures[0]}"
    )
    result = await rpc.connection.send_raw_transaction(bytes(tx))
    await rpc.connection.confirm_transaction(result.value)
    print(f"{name}: confirmed {result.value}")


async def main():
    client = rest_client()
    rpc = rpc_client()
    keypair = wallet()

    m = await market(client)
    market_pubkey = Pubkey.from_string(m.pubkey)
    d_mint = deposit_mint(m)
    outcomes = num_outcomes(m)
    amount = 1_000_000
    blockhash = await rpc.get_latest_blockhash()

    transactions = [
        (
            "mint_complete_set",
            await rpc.mint_complete_set(
                MintCompleteSetParams(
                    user=keypair.pubkey(),
                    market=market_pubkey,
                    deposit_mint=d_mint,
                    amount=amount,
                ),
                outcomes,
            ),
        ),
        (
            "merge_complete_set",
            await rpc.merge_complete_set(
                MergeCompleteSetParams(
                    user=keypair.pubkey(),
                    market=market_pubkey,
                    deposit_mint=d_mint,
                    amount=amount,
                ),
                outcomes,
            ),
        ),
        (
            "increment_nonce",
            await rpc.increment_nonce(keypair.pubkey()),
        ),
    ]

    for name, tx in transactions:
        await submit_transaction(name, rpc, tx, keypair, blockhash)

    await client.close()
    await rpc.connection.close()


asyncio.run(main())
