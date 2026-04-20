"""Featured markets, paginated listing, fetch by pubkey, search."""

import asyncio

from common import rest_client, market


async def main():
    client = rest_client()

    # 1. Featured markets
    featured = await client.markets().featured()
    print("featured markets:", len(featured))
    if featured:
        print(f"  featured: {featured[0].market_name} ({featured[0].slug})")

    # 2. Paginated listing
    page = await client.markets().get(None, 5)
    print(
        f"paginated listing: {len(page.markets)} markets, "
        f"{len(page.validation_errors)} validation errors"
    )

    # 3. Lookup by pubkey
    m = await market(client)
    print(f"by slug: {m.slug} -> {m.pubkey}")
    by_pubkey = await client.markets().get_by_pubkey(m.pubkey)
    print(f"by pubkey: {by_pubkey.name}")

    # 4. Search
    query = next((word for word in m.name.split() if len(word) > 3), "market")
    results = await client.markets().search(query, 5)
    print(f"search '{query}': {len(results)} result(s)")
    for result in results:
        print(f"  - {result.slug}")

    # 5. Global deposit asset whitelist
    global_assets = await client.markets().global_deposit_assets()
    print(
        f"global deposit assets: {len(global_assets.assets)}, "
        f"{len(global_assets.validation_errors)} validation error(s)"
    )
    for asset in global_assets.assets:
        print(f"  - {asset.symbol}")

    await client.close()


asyncio.run(main())
