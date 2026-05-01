"""Init position tokens, deposit to global pool, move capital into a market, extend an existing ALT, and withdraw from global."""

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
from lightcone_sdk.program.pda import get_position_alt_pda, get_position_pda
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
    deposit_amount = amount * 2  # deposit extra so global has funds after market transfer

    connection = require_connection(client)

    position_pda, _ = get_position_pda(keypair.pubkey(), market_pubkey, client.program_id)

    # Check if position already exists (init_position_tokens is one-time)
    position_account = await connection.get_account_info(position_pda)
    needs_init = position_account.value is None

    if needs_init:
        # Init + extend in a single transaction.
        # Use "processed" commitment for get_slot to minimize staleness — the
        # on-chain CreateLookupTable instruction rejects slots that are too old.
        from solana.rpc.commitment import Processed

        max_attempts = 5
        for attempt in range(1, max_attempts + 1):
            try:
                recent_slot = (await connection.get_slot(Processed)).value
                lookup_table, _ = get_position_alt_pda(position_pda, recent_slot)

                init_ix = (client.positions().init_position_tokens()
                    .payer(keypair.pubkey())
                    .user(keypair.pubkey())
                    .market(market_pubkey)
                    .deposit_mints([d_mint])
                    .recent_slot(recent_slot)
                    .num_outcomes(outcomes)
                    .build_ix())

                extend_ix = (client.positions().extend_position_tokens()
                    .payer(keypair.pubkey())
                    .user(keypair.pubkey())
                    .market(market_pubkey)
                    .lookup_table(lookup_table)
                    .deposit_mints([d_mint])
                    .num_outcomes(outcomes)
                    .build_ix())

                blockhash = await client.rpc().get_latest_blockhash()
                tx = await client.rpc().build_transaction([init_ix, extend_ix])
                tx.sign([keypair], blockhash)
                from solana.rpc.types import TxOpts
                result = await connection.send_raw_transaction(
                    bytes(tx), opts=TxOpts(skip_preflight=True)
                )
                await connection.confirm_transaction(result.value)
                print(f"init_position_tokens: confirmed {result.value}")
                break
            except Exception as error:
                message = str(error)
                retryable = (
                    "is not a recent slot" in message
                    or "UninitializedAccount" in message
                    or "already in use" in message
                )
                if attempt < max_attempts and retryable:
                    print(f"init_position_tokens: retrying ({attempt}/{max_attempts}): {message[:80]}")
                    await asyncio.sleep(2)
                    continue
                raise
    else:
        print("position already initialized, skipping init_position_tokens + extend")

    instructions: list[tuple[str, object]] = []

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

    # 6. Merge — burn the complete set of conditional tokens minted in step 4
    #    back to the deposit asset, returning the collateral to the user's
    #    token account. Closes out the market position so the full example is
    #    net-neutral on the wallet's balance, the global pool, and the market
    #    position across CI runs.
    instructions.append((
        "merge",
        client.positions().merge()
            .user(keypair.pubkey())
            .market(m)
            .mint(d_mint)
            .amount(amount)
            .build_ix(),
    ))

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

    # Withdraw — Market mode (position ATA -> user's wallet)
    market_withdraw_ix = (client.positions().withdraw()
        .user(keypair.pubkey())
        .mint(d_mint)
        .amount(amount)
        .with_market_deposit_source(m)
        .outcome_index(0)
        .token_2022(True)
        .build_ix())
    print(f"builder market withdraw ix: {len(market_withdraw_ix.accounts)} accounts")

    # Merge — burns complete set of conditional tokens, releases collateral
    merge_ix = (client.positions().merge()
        .user(keypair.pubkey())
        .market(m)
        .mint(d_mint)
        .amount(amount)
        .build_ix())
    print(f"builder merge ix: {len(merge_ix.accounts)} accounts")

    await client.close()


asyncio.run(main())
