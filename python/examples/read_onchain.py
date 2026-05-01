"""Read exchange state, market state, user nonce, and PDA derivations via RPC."""

import asyncio

from solders.pubkey import Pubkey

from common import client as make_client, get_keypair, market_and_orderbook, quote_deposit_mint
from lightcone_sdk.program.errors import AccountNotFoundError


async def main():
    client = make_client()
    keypair = get_keypair()

    m, orderbook = await market_and_orderbook(client)
    market_pubkey = Pubkey.from_string(m.pubkey)
    base_mint = Pubkey.from_string(orderbook.base.pubkey)
    quote_mint = Pubkey.from_string(orderbook.quote.pubkey)

    # 1. Exchange state
    exchange = await client.rpc().get_exchange()
    print(f"exchange: authority={exchange.authority} operator={exchange.operator} paused={exchange.paused}")

    # 2. Market state
    onchain_market = await client.markets().get_onchain(market_pubkey)
    print(f"market: id={onchain_market.market_id} outcomes={onchain_market.num_outcomes} status={onchain_market.status}")

    # 3. Orderbook
    try:
        onchain_orderbook = await client.orderbooks().get_onchain(base_mint, quote_mint)
        print(f"orderbook: lookup_table={onchain_orderbook.lookup_table} base_index={onchain_orderbook.base_index} bump={onchain_orderbook.bump}")
    except AccountNotFoundError:
        print("orderbook: not found on-chain")

    # 4. User nonce
    nonce = await client.orders().current_nonce(keypair.pubkey())
    print(f"user nonce: {nonce}")

    # 5. Position
    position = await client.positions().get_onchain(keypair.pubkey(), market_pubkey)
    print(f"position exists: {position is not None}")

    # 6. PDA derivations (via sub-client accessors)
    d_mint = quote_deposit_mint(orderbook)
    exchange_pda = client.rpc().get_exchange_pda()
    market_pda = client.markets().pda(onchain_market.market_id)
    position_pda = client.positions().pda(keypair.pubkey(), market_pubkey)
    global_deposit_pda = client.rpc().get_global_deposit_token_pda(d_mint)
    print(
        f"pdas: exchange={exchange_pda} market={market_pda} "
        f"position={position_pda} global_deposit={global_deposit_pda}"
    )

    await client.close()


asyncio.run(main())
