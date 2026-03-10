"""Full auth lifecycle: sign message, login, check session, logout."""

import asyncio

from common import rest_client, wallet
from src.auth.client import sign_login_message


async def main():
    client = rest_client()
    keypair = wallet()

    # 1. Request a nonce
    nonce = await client.auth().get_nonce()
    print("nonce:", nonce)

    # 2. Sign the nonce
    message, signature_bs58, pubkey_bytes = sign_login_message(keypair, nonce)
    print("message:", message)

    # 3. Login
    user = await client.auth().login_with_message(
        message, signature_bs58, pubkey_bytes
    )
    print(f"logged in: {user.id} ({user.wallet_address})")

    # 4. Check auth state
    print("cached auth state:", client.auth().is_authenticated())

    # 5. Verify session
    me = await client.auth().check_session()
    print("session wallet:", me.wallet_address)

    # 6. Logout
    await client.auth().logout()
    print("logged out")

    await client.close()


asyncio.run(main())
