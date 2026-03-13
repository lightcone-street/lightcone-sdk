"""Build, sign, and submit mint/merge complete set and increment nonce on-chain."""

import asyncio

from solders.pubkey import Pubkey

from common import rest_client, rpc_client, wallet, market, deposit_mint, num_outcomes
from src.program.types import MintCompleteSetParams, MergeCompleteSetParams


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

    # 1. Mint complete set (deposit collateral -> outcome tokens)
    mint_tx = await rpc.mint_complete_set(
        MintCompleteSetParams(
            user=keypair.pubkey(),
            market=market_pubkey,
            deposit_mint=d_mint,
            amount=amount,
        ),
        outcomes,
    )
    mint_tx.sign([keypair], blockhash)
    print(
        f"mint_complete_set: {len(mint_tx.message.instructions)} instruction(s), "
        f"signature={mint_tx.signatures[0]}"
    )

    # 2. Merge complete set (outcome tokens -> withdraw collateral)
    merge_tx = await rpc.merge_complete_set(
        MergeCompleteSetParams(
            user=keypair.pubkey(),
            market=market_pubkey,
            deposit_mint=d_mint,
            amount=amount,
        ),
        outcomes,
    )
    merge_tx.sign([keypair], blockhash)
    print(
        f"merge_complete_set: {len(merge_tx.message.instructions)} instruction(s), "
        f"signature={merge_tx.signatures[0]}"
    )

    # 3. Increment nonce
    nonce_tx = await rpc.increment_nonce(keypair.pubkey())
    nonce_tx.sign([keypair], blockhash)
    print(
        f"increment_nonce: {len(nonce_tx.message.instructions)} instruction(s), "
        f"signature={nonce_tx.signatures[0]}"
    )

    # To actually submit:
    # sig = await rpc.connection.send_raw_transaction(bytes(mint_tx))
    # await rpc.connection.confirm_transaction(sig.value)
    # print("submitted:", sig.value)

    await client.close()
    await rpc.connection.close()


asyncio.run(main())
