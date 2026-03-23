"""Init position tokens, deposit to global pool, move capital into a market, extend an existing ALT, and withdraw from global."""

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
    deposit_amount = amount * 2  # deposit extra so global has funds after market transfer

    connection = require_connection(client)

    position_pda, _ = get_position_pda(keypair.pubkey(), market_pubkey)

    # Check if position already exists (init_position_tokens is one-time)
    position_account = await connection.get_account_info(position_pda)
    needs_init = position_account.value is None

    instructions: list[tuple[str, object]] = []

    if needs_init:
        recent_slot = (await connection.get_slot()).value
        lookup_table, _ = get_position_alt_pda(position_pda, recent_slot)

        # 1. Init position tokens — one-time setup per market (creates position + ALT)
        instructions.append((
            "init_position_tokens",
            client.positions().init_position_tokens()
                .payer(keypair.pubkey())
                .user(keypair.pubkey())
                .market(market_pubkey)
                .deposit_mints([d_mint])
                .recent_slot(recent_slot)
                .num_outcomes(outcomes)
                .build_ix(),
        ))

        # 2. Extend position tokens — add deposit mint to ALT
        instructions.append((
            "extend_position_tokens",
            client.positions().extend_position_tokens()
                .payer(keypair.pubkey())
                .user(keypair.pubkey())
                .market(market_pubkey)
                .lookup_table(lookup_table)
                .deposit_mints([d_mint])
                .num_outcomes(outcomes)
                .build_ix(),
        ))
    else:
        print("position already initialized, skipping init_position_tokens + extend")

    # 3. Deposit to global — fund the global pool with collateral
    instructions.append((
        "deposit_to_global",
        client.positions().deposit_to_global()
            .user(keypair.pubkey())
            .mint(d_mint)
            .amount(deposit_amount)
            .build_ix(),
    ))

    # 4. Global to market deposit — move capital into a specific market
    instructions.append((
        "global_to_market_deposit",
        client.positions().global_to_market_deposit()
            .user(keypair.pubkey())
            .market(market_pubkey)
            .mint(d_mint)
            .amount(amount)
            .num_outcomes(outcomes)
            .build_ix(),
    ))

    # 5. Withdraw from global — pull remaining tokens back out
    instructions.append((
        "withdraw_from_global",
        client.positions().withdraw_from_global()
            .user(keypair.pubkey())
            .mint(d_mint)
            .amount(amount)
            .build_ix(),
    ))

    for name, ix in instructions:
        blockhash = await client.rpc().get_latest_blockhash()
        tx = await client.rpc().build_transaction([ix])
        tx.sign([keypair], blockhash)
        result = await connection.send_raw_transaction(bytes(tx))
        await connection.confirm_transaction(result.value)
        print(f"{name}: confirmed {result.value}")

    # ── Unified deposit/withdraw builders ──────────────────────────────
    #
    # These builders dispatch based on the client's deposit source setting
    # (or a per-call override).

    # Deposit — explicitly override to Global
    global_deposit_ix = (client.positions().deposit()
        .user(keypair.pubkey())
        .mint(d_mint)
        .amount(amount)
        .with_global_deposit_source()
        .build_ix())
    print(f"builder global deposit ix: {len(global_deposit_ix.accounts)} accounts")

    # Deposit — explicitly override to Market (mints conditional tokens)
    market_deposit_ix = (client.positions().deposit()
        .user(keypair.pubkey())
        .mint(d_mint)
        .amount(amount)
        .with_market_deposit_source(m)
        .build_ix())
    print(f"builder market deposit ix: {len(market_deposit_ix.accounts)} accounts")

    # Withdraw — Global mode (global pool -> wallet)
    global_withdraw_ix = (client.positions().withdraw()
        .user(keypair.pubkey())
        .mint(d_mint)
        .amount(amount)
        .with_global_deposit_source()
        .build_ix())
    print(f"builder global withdraw ix: {len(global_withdraw_ix.accounts)} accounts")

    # Withdraw — Market mode (burns conditional tokens -> wallet collateral)
    market_withdraw_ix = (client.positions().withdraw()
        .user(keypair.pubkey())
        .mint(d_mint)
        .amount(amount)
        .with_market_deposit_source(m)
        .build_ix())
    print(f"builder market withdraw ix: {len(market_withdraw_ix.accounts)} accounts")

    await client.close()


asyncio.run(main())
