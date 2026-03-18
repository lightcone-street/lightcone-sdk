"""Global deposit workflow: init position, deposit to global, move to market, extend ALT."""

import asyncio

from solders.pubkey import Pubkey

from common import (
    client as make_client,
    deposit_mint,
    login,
    market,
    num_outcomes,
    wallet,
)
from lightcone_sdk.program.pda import get_position_alt_pda, get_position_pda
from lightcone_sdk.program.types import (
    DepositToGlobalParams,
    ExtendPositionTokensParams,
    GlobalToMarketDepositParams,
    InitPositionTokensParams,
)
from lightcone_sdk.rpc import require_connection


async def main():
    client = make_client()
    keypair = wallet()
    await login(client, keypair)

    m = await market(client)
    market_pubkey = Pubkey.from_string(m.pubkey)
    d_mint = deposit_mint(m)
    outcomes = num_outcomes(m)
    amount = 1_000_000

    connection = require_connection(client)

    # 1. Init position tokens — one-time setup per market (creates position + ALT)
    recent_slot = (await connection.get_slot()).value

    # 4 uses the ALT address derived from position PDA + the slot used during init
    position_pda, _ = get_position_pda(keypair.pubkey(), market_pubkey)
    lookup_table, _ = get_position_alt_pda(position_pda, recent_slot)

    init_ix = client.positions().init_position_tokens_ix(
        InitPositionTokensParams(
            payer=keypair.pubkey(),
            user=keypair.pubkey(),
            market=market_pubkey,
            deposit_mints=[d_mint],
            recent_slot=recent_slot,
        ),
        outcomes,
    )

    # 2. Deposit to global — fund the global pool with collateral
    deposit_ix = client.positions().deposit_to_global_ix(
        DepositToGlobalParams(
            user=keypair.pubkey(),
            mint=d_mint,
            amount=amount,
        )
    )

    # 3. Global to market deposit — move capital into a specific market
    move_ix = client.positions().global_to_market_deposit_ix(
        GlobalToMarketDepositParams(
            user=keypair.pubkey(),
            market=market_pubkey,
            deposit_mint=d_mint,
            amount=amount,
        ),
        outcomes,
    )

    # 4. Extend position tokens — add a new deposit mint to an existing ALT
    extend_ix = client.positions().extend_position_tokens_ix(
        ExtendPositionTokensParams(
            payer=keypair.pubkey(),
            user=keypair.pubkey(),
            market=market_pubkey,
            lookup_table=lookup_table,
            deposit_mints=[d_mint],
        ),
        outcomes,
    )

    instructions = [
        ("init_position_tokens", init_ix),
        ("deposit_to_global", deposit_ix),
        ("global_to_market_deposit", move_ix),
        ("extend_position_tokens", extend_ix),
    ]

    for name, ix in instructions:
        blockhash = await client.rpc().get_latest_blockhash()
        tx = await client.rpc().build_transaction([ix])
        tx.sign([keypair], blockhash)
        result = await connection.send_raw_transaction(bytes(tx))
        await connection.confirm_transaction(result.value)
        print(f"{name}: confirmed {result.value}")

    await client.close()


asyncio.run(main())
