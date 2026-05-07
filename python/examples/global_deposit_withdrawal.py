"""Deposit to global pool, move capital into a market, and withdraw from global."""

import asyncio

from solders.pubkey import Pubkey

from common import (
    client as make_client,
    login,
    market_and_orderbook,
    num_outcomes,
    quote_deposit_mint,
    get_keypair,
)
from lightcone_sdk.rpc import require_connection


async def main():
    client = make_client()
    keypair = get_keypair()
    await login(client, keypair)

    m, ob = await market_and_orderbook(client)
    market_pubkey = Pubkey.from_string(m.pubkey)
    d_mint = quote_deposit_mint(ob)
    outcomes = num_outcomes(m)
    amount = 1_000_000
    deposit_amount = (
        amount * 2
    )  # deposit extra so global has funds after market transfer

    connection = require_connection(client)

    instructions: list[tuple[str, object]] = []

    # 1. Deposit to global — fund the global pool with collateral
    instructions.append(
        (
            "deposit_to_global",
            client.positions()
            .deposit_to_global()
            .user(keypair.pubkey())
            .mint(d_mint)
            .amount(deposit_amount)
            .build_ix(),
        )
    )

    # 2. Global to market deposit — move capital into a specific market
    instructions.append(
        (
            "global_to_market_deposit",
            client.positions()
            .global_to_market_deposit()
            .user(keypair.pubkey())
            .market(market_pubkey)
            .mint(d_mint)
            .amount(amount)
            .num_outcomes(outcomes)
            .build_ix(),
        )
    )

    # 3. Withdraw from global — pull remaining tokens back out
    instructions.append(
        (
            "withdraw_from_global",
            client.positions()
            .withdraw_from_global()
            .user(keypair.pubkey())
            .mint(d_mint)
            .amount(amount)
            .build_ix(),
        )
    )

    # 4. Merge — burn the complete set of conditional tokens minted in step 2
    #    back to the deposit asset, returning the collateral to the user's
    #    token account. Closes out the market position so the full example is
    #    net-neutral on the wallet's balance, the global pool, and the market
    #    position across CI runs.
    instructions.append(
        (
            "merge",
            client.positions()
            .merge()
            .user(keypair.pubkey())
            .market(m)
            .mint(d_mint)
            .amount(amount)
            .build_ix(),
        )
    )

    for index, (name, ix) in enumerate(instructions):
        if index > 0:
            await asyncio.sleep(1)  # avoid devnet RPC rate limits
        blockhash = await client.rpc().get_latest_blockhash()
        tx = await client.rpc().build_transaction([ix])
        tx.sign([keypair], blockhash)
        result = await connection.send_raw_transaction(bytes(tx))
        await connection.confirm_transaction(result.value)
        print(f"{name}: confirmed {result.value}")

    # ── Unified deposit/withdraw/merge builders ─────────────────────────
    #
    # Deposit and withdraw builders dispatch based on the client's deposit
    # source setting (or a per-call override). Merge is market-only.

    # Deposit — explicitly override to Global
    global_deposit_ix = (
        client.positions()
        .deposit()
        .user(keypair.pubkey())
        .mint(d_mint)
        .amount(amount)
        .with_global_deposit_source()
        .build_ix()
    )
    print(f"builder global deposit ix: {len(global_deposit_ix.accounts)} accounts")

    # Deposit — explicitly override to Market (mints conditional tokens)
    market_deposit_ix = (
        client.positions()
        .deposit()
        .user(keypair.pubkey())
        .mint(d_mint)
        .amount(amount)
        .with_market_deposit_source(m)
        .build_ix()
    )
    print(f"builder market deposit ix: {len(market_deposit_ix.accounts)} accounts")

    # Withdraw — Global mode (global pool -> wallet)
    global_withdraw_ix = (
        client.positions()
        .withdraw()
        .user(keypair.pubkey())
        .mint(d_mint)
        .amount(amount)
        .with_global_deposit_source()
        .build_ix()
    )
    print(f"builder global withdraw ix: {len(global_withdraw_ix.accounts)} accounts")

    # Withdraw — Market mode (position ATA -> user's wallet)
    market_withdraw_ix = (
        client.positions()
        .withdraw()
        .user(keypair.pubkey())
        .mint(d_mint)
        .amount(amount)
        .with_market_deposit_source(m)
        .outcome_index(0)
        .token_2022(True)
        .build_ix()
    )
    print(f"builder market withdraw ix: {len(market_withdraw_ix.accounts)} accounts")

    # Merge — burns complete set of conditional tokens, releases collateral
    merge_ix = (
        client.positions()
        .merge()
        .user(keypair.pubkey())
        .market(m)
        .mint(d_mint)
        .amount(amount)
        .build_ix()
    )
    print(f"builder merge ix: {len(merge_ix.accounts)} accounts")

    await client.close()


asyncio.run(main())
