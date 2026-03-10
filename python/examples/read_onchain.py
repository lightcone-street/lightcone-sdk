"""Read exchange state, market state, user nonce, and PDA derivations via RPC."""

import asyncio

from solders.pubkey import Pubkey

from common import rest_client, rpc_client, wallet, market_and_orderbook, deposit_mint
from src.program.pda import (
    get_exchange_pda,
    get_market_pda,
    get_position_pda,
    get_global_deposit_pda,
)


async def main():
    client = rest_client()
    rpc = rpc_client()
    keypair = wallet()

    m, orderbook = await market_and_orderbook(client)
    market_pubkey = Pubkey.from_string(m.pubkey)
    base_mint = Pubkey.from_string(orderbook.base_token)
    quote_mint = Pubkey.from_string(orderbook.quote_token)

    # 1. Exchange state
    exchange = await rpc.get_exchange()
    print(f"exchange: authority={exchange.authority} operator={exchange.operator} paused={exchange.paused}")

    # 2. Market state
    onchain_market = await rpc.get_market_by_pubkey(market_pubkey)
    print(f"market: id={onchain_market.market_id} outcomes={onchain_market.num_outcomes} status={onchain_market.status}")

    # 3. Orderbook
    onchain_orderbook = await rpc.get_orderbook(base_mint, quote_mint)
    if onchain_orderbook:
        print(f"orderbook: lookup_table={onchain_orderbook.lookup_table} bump={onchain_orderbook.bump}")
    else:
        print("orderbook: not found on-chain")

    # 4. User nonce
    nonce = await rpc.get_current_nonce(keypair.pubkey())
    print(f"user nonce: {nonce}")

    # 5. Position
    position = await rpc.get_position(keypair.pubkey(), market_pubkey)
    print(f"position exists: {position is not None}")

    # 6. PDA derivations
    d_mint = deposit_mint(m)
    exchange_pda, _ = get_exchange_pda()
    market_pda, _ = get_market_pda(onchain_market.market_id)
    position_pda, _ = get_position_pda(keypair.pubkey(), market_pubkey)
    global_deposit_pda, _ = get_global_deposit_pda(d_mint)
    print(
        f"pdas: exchange={exchange_pda} market={market_pda} "
        f"position={position_pda} global_deposit={global_deposit_pda}"
    )

    await client.close()
    await rpc.connection.close()


asyncio.run(main())
