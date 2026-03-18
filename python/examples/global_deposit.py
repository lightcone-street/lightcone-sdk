"""Build, sign, and submit deposit-to-global and global-to-market transactions."""

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
from lightcone_sdk.program.errors import AccountNotFoundError
from lightcone_sdk.program.types import (
    DepositToGlobalParams,
    GlobalToMarketDepositParams,
)
from lightcone_sdk.rpc import require_connection


def print_global_balance(snapshot, mint: Pubkey) -> None:
    mint_str = str(mint)
    entry = next(
        (item for item in snapshot.global_deposits if item.deposit_mint == mint_str),
        None,
    )
    if entry is None:
        print(f"global balance: {mint_str} not present")
        return
    symbol = entry.symbol or mint_str
    print(f"global balance: {symbol}={entry.balance}")


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
    user = await login(client, keypair)

    m = await market(client)
    market_pubkey = Pubkey.from_string(m.pubkey)
    d_mint = deposit_mint(m)
    outcomes = num_outcomes(m)
    amount = 1_000_000

    print(f"market: {m.slug}")
    print(f"deposit mint: {d_mint}")

    try:
        global_token = await client.rpc().get_global_deposit_token(d_mint)
    except AccountNotFoundError:
        print(
            "global deposit token not found on-chain; "
            "the deposit mint must be whitelisted first"
        )
        await client.close()
        return

    print(
        f"global deposit token: active={global_token.active} "
        f"index={global_token.index}"
    )

    before = await client.positions().get(user.wallet_address)
    print_global_balance(before, d_mint)

    deposit_ix = client.positions().deposit_to_global_ix(
        DepositToGlobalParams(
            user=keypair.pubkey(),
            mint=d_mint,
            amount=amount,
        )
    )
    market_ix = client.positions().global_to_market_deposit_ix(
        GlobalToMarketDepositParams(
            user=keypair.pubkey(),
            market=market_pubkey,
            deposit_mint=d_mint,
            amount=amount,
        ),
        outcomes,
    )

    transactions = [
        ("deposit_to_global", await client.rpc().build_transaction([deposit_ix])),
        (
            "global_to_market_deposit",
            await client.rpc().build_transaction([market_ix]),
        ),
    ]

    connection = require_connection(client)
    for name, tx in transactions:
        blockhash = await client.rpc().get_latest_blockhash()
        await submit_transaction(name, connection, tx, keypair, blockhash)

    after = await client.positions().get(user.wallet_address)
    print_global_balance(after, d_mint)

    per_market = await client.positions().get_for_market(
        user.wallet_address, m.pubkey
    )
    print(f"positions in {m.slug}: {len(per_market.positions)}")

    await client.close()


asyncio.run(main())
