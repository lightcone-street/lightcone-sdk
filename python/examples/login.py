"""Full auth lifecycle: sign message, login, check session, logout."""

import asyncio

from common import rest_client, get_keypair
from lightcone_sdk.auth import identity_text
from lightcone_sdk.auth.client import sign_login_message


async def main():
    client = rest_client()
    keypair = get_keypair()

    nonce = await client.auth().get_nonce()
    message, signature_bs58, pubkey_bytes = sign_login_message(keypair, nonce)
    session = await client.auth().login_with_message(
        message, signature_bs58, pubkey_bytes
    )

    wallet = session.user.trading_wallet(session.auth_method)
    print(f"logged in: {session.user.user_id} ({wallet})")
    print("identity:", identity_text(session.user.identity))
    print("display name:", session.user.display_name())
    print("cached auth state:", client.auth().is_authenticated())
    me = await client.auth().check_session()
    print("session wallet:", me.user.trading_wallet(me.auth_method))
    await client.auth().logout()
    print("logged out")

    await client.close()


asyncio.run(main())
