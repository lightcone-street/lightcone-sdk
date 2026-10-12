"""Batch orderbook tickers — best_bid / best_ask / midpoint per orderbook."""

import asyncio
import sys

from common import rest_client


async def main():
    client = rest_client()
    deposit_asset = sys.argv[1] if len(sys.argv) > 1 else None

    response = await client.metrics().orderbook_tickers(deposit_asset)

    print(f"orderbooks with tickers: {len(response.tickers)}")
    if response.has_more:
        # The page cap was reached; resume with orderbook_tickers_page().
        print(f"more tickers after cursor {response.next_cursor}")
    for entry in response.tickers[:10]:
        mid = entry.midpoint or "—"
        print(
            f"  {entry.orderbook_id} (market {entry.market_pubkey}, "
            f"outcome {entry.outcome_index}) mid={mid}"
        )

    await client.close()


asyncio.run(main())
