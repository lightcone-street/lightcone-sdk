"""
Devnet integration tests for the Lightcone SDK.

These tests run against Solana devnet and test the full end-to-end flow
of the Lightcone protocol.

To run:
    DEVNET_TESTS=1 pytest tests/test_devnet.py -v -s

Environment variables:
    DEVNET_RPC: Solana devnet RPC URL (default: https://api.devnet.solana.com)
    AUTHORITY_KEYPAIR: Path to authority keypair JSON file
"""

import asyncio
import json
import os
import time
from pathlib import Path

import pytest
from solana.rpc.async_api import AsyncClient
from solana.rpc.commitment import Confirmed
from solders.keypair import Keypair
from solders.message import Message
from solders.pubkey import Pubkey
from solders.signature import Signature
from solders.system_program import TransferParams, transfer
from solders.transaction import Transaction
from spl.token.async_client import AsyncToken
from spl.token.constants import TOKEN_PROGRAM_ID as SPL_TOKEN_PROGRAM_ID

from src import (
    PROGRAM_ID,
    ActivateMarketParams,
    AskOrderParams,
    BidOrderParams,
    LightconePinocchioClient,
    MakerFill,
    MarketStatus,
    MergeCompleteSetParams,
    MintCompleteSetParams,
    OutcomeMetadata,
    RedeemWinningsParams,
    SettleMarketParams,
    WithdrawFromPositionParams,
    build_add_deposit_mint_instruction,
    get_conditional_mint_pda,
    get_position_pda,
    hash_order,
)
from src.shared import get_associated_token_address, get_associated_token_address_2022

# Test configuration
DEVNET_RPC = os.environ.get("DEVNET_RPC", "https://api.devnet.solana.com")
LAMPORTS_PER_SOL = 1_000_000_000
TOKEN_DECIMALS = 6
TOKENS_TO_MINT = 10 * (10**TOKEN_DECIMALS)  # 10 tokens
COMPLETE_SET_AMOUNT = 1 * (10**TOKEN_DECIMALS)  # 1 token


def log(emoji: str, message: str, data: any = None):
    """Log with emoji prefix."""
    if data:
        print(f"{emoji} {message}: {data}")
    else:
        print(f"{emoji} {message}")


def success(message: str):
    log("✅", message)


def error(message: str):
    log("❌", message)


def warn(message: str):
    log("⚠️", message)


def info(message: str):
    log("ℹ️", message)


def load_keypair(path: str) -> Keypair:
    """Load a keypair from a JSON file."""
    with open(path, "r") as f:
        secret = json.load(f)
    return Keypair.from_bytes(bytes(secret))


def get_authority_keypair() -> Keypair:
    """Get the authority keypair from file or environment."""
    # Try keypairs directory first
    keypair_path = Path(__file__).parent.parent / "keypairs" / "devnet-authority.json"
    if keypair_path.exists():
        return load_keypair(str(keypair_path))

    # Try AUTHORITY_KEYPAIR env var
    env_path = os.environ.get("AUTHORITY_KEYPAIR")
    if env_path and Path(env_path).exists():
        return load_keypair(env_path)

    # Try default Solana CLI path
    default_path = Path.home() / ".config" / "solana" / "id.json"
    if default_path.exists():
        return load_keypair(str(default_path))

    raise FileNotFoundError(
        "No keypair found. Create keypairs/devnet-authority.json or set AUTHORITY_KEYPAIR"
    )


async def confirm_tx(connection: AsyncClient, signature: Signature, max_retries: int = 30):
    """Wait for transaction confirmation."""
    for _ in range(max_retries):
        response = await connection.get_signature_statuses([signature])
        if response.value[0] is not None:
            if response.value[0].err:
                raise Exception(f"Transaction failed: {response.value[0].err}")
            return
        await asyncio.sleep(1)
    raise TimeoutError(f"Transaction {signature} not confirmed after {max_retries}s")


async def airdrop_if_needed(connection: AsyncClient, pubkey: Pubkey, min_balance: int = LAMPORTS_PER_SOL):
    """Request airdrop if balance is below minimum."""
    balance_resp = await connection.get_balance(pubkey)
    balance = balance_resp.value

    if balance < min_balance:
        info(f"Balance low ({balance / LAMPORTS_PER_SOL:.2f} SOL), requesting airdrop...")
        try:
            sig_resp = await connection.request_airdrop(pubkey, 2 * LAMPORTS_PER_SOL)
            await confirm_tx(connection, sig_resp.value)
            success("Airdrop received")
        except Exception as e:
            warn(f"Airdrop failed: {e}")


async def fund_account(
    connection: AsyncClient,
    payer: Keypair,
    recipient: Pubkey,
    lamports: int,
):
    """Transfer lamports to an account."""
    blockhash_resp = await connection.get_latest_blockhash()
    blockhash = blockhash_resp.value.blockhash

    ix = transfer(
        TransferParams(
            from_pubkey=payer.pubkey(),
            to_pubkey=recipient,
            lamports=lamports,
        )
    )

    msg = Message.new_with_blockhash([ix], payer.pubkey(), blockhash)
    tx = Transaction.new_unsigned(msg)
    tx.sign([payer], blockhash)

    result = await connection.send_raw_transaction(bytes(tx))
    await confirm_tx(connection, result.value)


@pytest.mark.asyncio
class TestDevnetIntegration:
    """Integration tests running against Solana devnet.

    Tests are structured as a single flow to ensure proper ordering
    regardless of pytest plugins like pytest-randomly.
    """

    async def test_full_market_lifecycle(self):
        """
        Complete end-to-end test of the Lightcone protocol:
        1. Check authority balance
        2. Initialize or verify exchange
        3. Create market
        4. Create deposit mint
        5. Add deposit mint to market
        6. Activate market
        7. Setup users with tokens
        8. Mint complete sets
        9. Merge complete set
        10. Create and match orders
        11. Settle market
        12. Redeem winnings
        """
        # Setup
        connection = AsyncClient(DEVNET_RPC, commitment=Confirmed)
        authority = get_authority_keypair()
        client = LightconePinocchioClient(connection, PROGRAM_ID)

        user1 = Keypair()
        user2 = Keypair()

        try:
            # ===== Step 1: Check authority balance =====
            info(f"Authority: {authority.pubkey()}")
            await airdrop_if_needed(connection, authority.pubkey(), 2 * LAMPORTS_PER_SOL)

            balance_resp = await connection.get_balance(authority.pubkey())
            balance = balance_resp.value / LAMPORTS_PER_SOL
            success(f"Authority balance: {balance:.2f} SOL")
            assert balance > 0.1, "Authority needs more SOL"

            # ===== Step 2: Initialize or verify exchange =====
            exchange = None
            try:
                exchange = await client.get_exchange()
                success(f"Exchange already initialized, market_count={exchange.market_count}")
                info(f"Exchange authority: {exchange.authority}")
                info(f"Exchange operator: {exchange.operator}")

                # Check if we're the authority
                if exchange.authority != authority.pubkey():
                    warn(f"Exchange authority ({exchange.authority}) does not match our keypair ({authority.pubkey()})")
                    warn("Skipping market creation tests - you need to use the correct authority keypair")
                    pytest.skip("Not the exchange authority - cannot create markets")

            except Exception:
                info("Initializing exchange...")
                tx = await client.initialize(authority.pubkey())

                blockhash_resp = await connection.get_latest_blockhash()
                tx.sign([authority], blockhash_resp.value.blockhash)

                result = await connection.send_raw_transaction(bytes(tx))
                await confirm_tx(connection, result.value)
                success("Exchange initialized")

                exchange = await client.get_exchange()

            # ===== Step 3: Create market =====
            market_id = exchange.market_count
            info(f"Creating market {market_id}...")

            question_id = bytes([market_id % 256] * 32)

            tx = await client.create_market(
                authority=authority.pubkey(),
                num_outcomes=2,
                oracle=authority.pubkey(),
                question_id=question_id,
            )

            blockhash_resp = await connection.get_latest_blockhash()
            tx.sign([authority], blockhash_resp.value.blockhash)

            result = await connection.send_raw_transaction(bytes(tx))
            await confirm_tx(connection, result.value)

            market = await client.get_market(market_id)
            market_pda = client.get_market_address(market_id)

            success(f"Market {market_id} created")
            assert market.market_id == market_id
            assert market.num_outcomes == 2
            assert market.status == MarketStatus.PENDING

            # ===== Step 4: Create deposit mint =====
            info("Creating deposit mint...")

            token = await AsyncToken.create_mint(
                connection,
                authority,
                authority.pubkey(),
                TOKEN_DECIMALS,
                SPL_TOKEN_PROGRAM_ID,
            )

            deposit_mint = token.pubkey
            success(f"Deposit mint created: {deposit_mint}")

            # ===== Step 5: Add deposit mint to market =====
            info("Adding deposit mint to market...")

            outcome_metadata = [
                OutcomeMetadata(name="Yes", symbol="YES", uri=""),
                OutcomeMetadata(name="No", symbol="NO", uri=""),
            ]

            ix = build_add_deposit_mint_instruction(
                payer=authority.pubkey(),
                market=market_pda,
                deposit_mint=deposit_mint,
                outcome_metadata=outcome_metadata,
                num_outcomes=2,
            )

            blockhash_resp = await connection.get_latest_blockhash()
            msg = Message.new_with_blockhash([ix], authority.pubkey(), blockhash_resp.value.blockhash)
            tx = Transaction.new_unsigned(msg)
            tx.sign([authority], blockhash_resp.value.blockhash)

            result = await connection.send_raw_transaction(bytes(tx))
            await confirm_tx(connection, result.value)

            success("Deposit mint added to market")

            # ===== Step 6: Activate market =====
            info("Activating market...")

            tx = await client.activate_market(
                ActivateMarketParams(
                    authority=authority.pubkey(),
                    market=market_pda,
                )
            )

            blockhash_resp = await connection.get_latest_blockhash()
            tx.sign([authority], blockhash_resp.value.blockhash)

            result = await connection.send_raw_transaction(bytes(tx))
            await confirm_tx(connection, result.value)

            market = await client.get_market(market_id)
            success(f"Market activated, status={market.status.name}")
            assert market.status == MarketStatus.ACTIVE

            # ===== Step 7: Setup user1 =====
            info(f"Setting up user1: {user1.pubkey()}")

            await fund_account(connection, authority, user1.pubkey(), int(0.1 * LAMPORTS_PER_SOL))

            token_client = AsyncToken(connection, deposit_mint, SPL_TOKEN_PROGRAM_ID, authority)
            user1_ata = await token_client.create_associated_token_account(user1.pubkey())
            await token_client.mint_to(user1_ata, authority, TOKENS_TO_MINT)
            # Wait for transaction finality on devnet
            await asyncio.sleep(2)

            success(f"User1 funded with {TOKENS_TO_MINT / 10**TOKEN_DECIMALS} tokens")

            # ===== Step 8: User1 mints complete set =====
            info("User1 minting complete set...")

            # Debug: Verify ATA addresses match
            from src.shared import get_associated_token_address
            sdk_user_ata = get_associated_token_address(user1.pubkey(), deposit_mint)
            info(f"User1 ATA from test: {user1_ata}")
            info(f"User1 ATA from SDK:  {sdk_user_ata}")
            if str(user1_ata) != str(sdk_user_ata):
                error(f"ATA MISMATCH! Test created {user1_ata}, SDK expects {sdk_user_ata}")

            # Debug: Check token balance before mint_complete_set
            balance_resp = await connection.get_token_account_balance(user1_ata)
            info(f"User1 ATA balance: {balance_resp.value.ui_amount} tokens")
            info(f"User1 ATA raw amount: {balance_resp.value.amount}")
            info(f"Attempting to transfer: {COMPLETE_SET_AMOUNT} ({COMPLETE_SET_AMOUNT / 10**TOKEN_DECIMALS} tokens)")

            tx = await client.mint_complete_set(
                MintCompleteSetParams(
                    user=user1.pubkey(),
                    market=market_pda,
                    deposit_mint=deposit_mint,
                    amount=COMPLETE_SET_AMOUNT,
                ),
                num_outcomes=2,
            )

            blockhash_resp = await connection.get_latest_blockhash()
            tx.sign([user1], blockhash_resp.value.blockhash)

            result = await connection.send_raw_transaction(bytes(tx))
            await confirm_tx(connection, result.value)

            position = await client.get_position(user1.pubkey(), market_pda)
            assert position is not None
            success("User1 minted complete set")

            # ===== Step 9: Setup user2 =====
            info(f"Setting up user2: {user2.pubkey()}")

            await fund_account(connection, authority, user2.pubkey(), int(0.1 * LAMPORTS_PER_SOL))

            user2_ata = await token_client.create_associated_token_account(user2.pubkey())
            await token_client.mint_to(user2_ata, authority, TOKENS_TO_MINT * 2)
            # Wait for transaction finality on devnet
            await asyncio.sleep(2)

            success(f"User2 funded with {TOKENS_TO_MINT * 2 / 10**TOKEN_DECIMALS} tokens")

            # ===== Step 10: User2 mints complete set =====
            info("User2 minting complete set...")

            tx = await client.mint_complete_set(
                MintCompleteSetParams(
                    user=user2.pubkey(),
                    market=market_pda,
                    deposit_mint=deposit_mint,
                    amount=COMPLETE_SET_AMOUNT * 5,
                ),
                num_outcomes=2,
            )

            blockhash_resp = await connection.get_latest_blockhash()
            tx.sign([user2], blockhash_resp.value.blockhash)

            result = await connection.send_raw_transaction(bytes(tx))
            await confirm_tx(connection, result.value)

            success("User2 minted complete set")

            # ===== Step 11: User2 merges complete set =====
            info("User2 merging complete set...")

            tx = await client.merge_complete_set(
                MergeCompleteSetParams(
                    user=user2.pubkey(),
                    market=market_pda,
                    deposit_mint=deposit_mint,
                    amount=COMPLETE_SET_AMOUNT,
                ),
                num_outcomes=2,
            )

            blockhash_resp = await connection.get_latest_blockhash()
            tx.sign([user2], blockhash_resp.value.blockhash)

            result = await connection.send_raw_transaction(bytes(tx))
            await confirm_tx(connection, result.value)

            success("User2 merged complete set")

            # Get conditional mint addresses (needed for withdraw and orders)
            yes_mint, _ = get_conditional_mint_pda(market_pda, deposit_mint, 0)
            no_mint, _ = get_conditional_mint_pda(market_pda, deposit_mint, 1)

            # ===== Step 11.5: Withdraw from position =====
            info("User2 withdrawing conditional tokens from position...")

            user2_position_pda, _ = get_position_pda(user2.pubkey(), market_pda)

            # First create user2's personal ATA for the YES conditional token (Token-2022)
            from spl.token._layouts import ACCOUNT_LAYOUT
            from spl.token.instructions import create_associated_token_account
            from spl.token.constants import ASSOCIATED_TOKEN_PROGRAM_ID

            TOKEN_2022_PROG_ID = Pubkey.from_string("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb")
            user2_yes_personal_ata = get_associated_token_address_2022(user2.pubkey(), yes_mint)

            # Create the ATA instruction
            create_ata_ix = create_associated_token_account(
                payer=authority.pubkey(),
                owner=user2.pubkey(),
                mint=yes_mint,
                token_program_id=TOKEN_2022_PROG_ID,
            )

            blockhash_resp = await connection.get_latest_blockhash()
            msg = Message.new_with_blockhash([create_ata_ix], authority.pubkey(), blockhash_resp.value.blockhash)
            create_ata_tx = Transaction.new_unsigned(msg)
            create_ata_tx.sign([authority], blockhash_resp.value.blockhash)

            result = await connection.send_raw_transaction(bytes(create_ata_tx))
            await confirm_tx(connection, result.value)
            info(f"User2 personal conditional ATA created: {user2_yes_personal_ata}")

            # Now withdraw from position
            tx = await client.withdraw_from_position(
                WithdrawFromPositionParams(
                    user=user2.pubkey(),
                    position=user2_position_pda,
                    mint=yes_mint,
                    amount=100_000,  # Withdraw 0.1 conditional tokens
                ),
                is_token_2022=True,
            )

            blockhash_resp = await connection.get_latest_blockhash()
            tx.sign([user2], blockhash_resp.value.blockhash)

            result = await connection.send_raw_transaction(bytes(tx))
            await confirm_tx(connection, result.value)

            success("User2 withdrew conditional tokens from position")

            # ===== Step 12: Cancel order test =====
            info("Testing cancel order...")

            expiration = int(time.time()) + 3600  # 1 hour

            # Increment nonce first to create the nonce account
            tx_nonce = await client.increment_nonce(user1.pubkey())
            blockhash_resp = await connection.get_latest_blockhash()
            tx_nonce.sign([user1], blockhash_resp.value.blockhash)
            await connection.send_raw_transaction(bytes(tx_nonce))
            await asyncio.sleep(1)

            cancel_nonce = await client.get_next_nonce(user1.pubkey())
            cancel_order = client.create_signed_bid_order(
                BidOrderParams(
                    nonce=cancel_nonce,
                    maker=user1.pubkey(),
                    market=market_pda,
                    base_mint=yes_mint,
                    quote_mint=no_mint,
                    maker_amount=50_000,
                    taker_amount=50_000,
                    expiration=expiration,
                ),
                user1,
            )

            tx = await client.cancel_order(user1.pubkey(), cancel_order)
            blockhash_resp = await connection.get_latest_blockhash()
            tx.sign([user1], blockhash_resp.value.blockhash)

            result = await connection.send_raw_transaction(bytes(tx))
            await confirm_tx(connection, result.value)

            # Verify order is cancelled
            cancel_status = await client.get_order_status(hash_order(cancel_order))
            assert cancel_status is not None
            assert cancel_status.is_cancelled == True
            success(f"Order cancelled, is_cancelled={cancel_status.is_cancelled}")

            # ===== Step 13: Create and match orders =====
            info("Creating orders for matching...")

            # Increment nonces for both users (user1 needs fresh nonce after cancel test)
            tx1 = await client.increment_nonce(user1.pubkey())
            blockhash_resp = await connection.get_latest_blockhash()
            tx1.sign([user1], blockhash_resp.value.blockhash)
            await connection.send_raw_transaction(bytes(tx1))
            await asyncio.sleep(1)

            tx2 = await client.increment_nonce(user2.pubkey())
            blockhash_resp = await connection.get_latest_blockhash()
            tx2.sign([user2], blockhash_resp.value.blockhash)
            await connection.send_raw_transaction(bytes(tx2))
            await asyncio.sleep(1)

            user1_nonce = await client.get_next_nonce(user1.pubkey())
            user2_nonce = await client.get_next_nonce(user2.pubkey())
            info(f"User1 nonce: {user1_nonce}, User2 nonce: {user2_nonce}")

            # User1 BID: wants YES tokens, gives NO tokens
            user1_order = client.create_signed_bid_order(
                BidOrderParams(
                    nonce=user1_nonce,
                    maker=user1.pubkey(),
                    market=market_pda,
                    base_mint=yes_mint,
                    quote_mint=no_mint,
                    maker_amount=100_000,
                    taker_amount=100_000,
                    expiration=expiration,
                ),
                user1,
            )

            # User2 ASK: sells YES tokens, receives NO tokens
            user2_order = client.create_signed_ask_order(
                AskOrderParams(
                    nonce=user2_nonce,
                    maker=user2.pubkey(),
                    market=market_pda,
                    base_mint=yes_mint,
                    quote_mint=no_mint,
                    maker_amount=100_000,
                    taker_amount=100_000,
                    expiration=expiration,
                ),
                user2,
            )

            success(f"User1 order hash: {hash_order(user1_order).hex()[:16]}...")
            success(f"User2 order hash: {hash_order(user2_order).hex()[:16]}...")

            # ===== Step 14: Match orders =====
            info("Matching orders...")

            # Debug: Print order details
            info(f"User1 order maker: {user1_order.maker}")
            info(f"User1 order nonce: {user1_order.nonce}")
            info(f"User1 order side: {user1_order.side}")
            info(f"User2 order maker: {user2_order.maker}")
            info(f"User2 order nonce: {user2_order.nonce}")
            info(f"User2 order side: {user2_order.side}")

            # Verify signature lengths
            info(f"User1 signature length: {len(user1_order.signature)}")
            info(f"User2 signature length: {len(user2_order.signature)}")

            # Verify positions exist
            user1_pos = await client.get_position(user1.pubkey(), market_pda)
            user2_pos = await client.get_position(user2.pubkey(), market_pda)
            info(f"User1 position exists: {user1_pos is not None}")
            info(f"User2 position exists: {user2_pos is not None}")

            # Use cross_ref for efficient Ed25519 verification
            # The on-chain program requires Ed25519 verify instructions in the transaction
            info(f"Using operator: {exchange.operator}")
            tx = await client.match_orders_multi_cross_ref(
                operator=exchange.operator,
                market=market_pda,
                base_mint=yes_mint,
                quote_mint=no_mint,
                taker_order=user1_order,
                maker_fills=[
                    MakerFill(order=user2_order, fill_amount=100_000),
                ],
            )

            blockhash_resp = await connection.get_latest_blockhash()
            tx.sign([authority], blockhash_resp.value.blockhash)

            result = await connection.send_raw_transaction(bytes(tx))
            await confirm_tx(connection, result.value)

            user1_status = await client.get_order_status(hash_order(user1_order))
            user2_status = await client.get_order_status(hash_order(user2_order))

            success(f"Orders matched! User1 remaining={user1_status.remaining}, User2 remaining={user2_status.remaining}")

            # ===== Step 14: Settle market =====
            info("Settling market (outcome 0 = YES wins)...")

            tx = await client.settle_market(
                SettleMarketParams(
                    oracle=authority.pubkey(),
                    market=market_pda,
                    winning_outcome=0,
                )
            )

            blockhash_resp = await connection.get_latest_blockhash()
            tx.sign([authority], blockhash_resp.value.blockhash)

            result = await connection.send_raw_transaction(bytes(tx))
            await confirm_tx(connection, result.value)

            market = await client.get_market(market_id)
            success(f"Market settled, status={market.status.name}, winner={market.winning_outcome}")
            assert market.status == MarketStatus.RESOLVED
            assert market.winning_outcome == 0

            # ===== Step 15: Redeem winnings =====
            info("User1 redeeming winnings...")

            tx = await client.redeem_winnings(
                RedeemWinningsParams(
                    user=user1.pubkey(),
                    market=market_pda,
                    deposit_mint=deposit_mint,
                    amount=100_000,
                ),
                winning_outcome=0,
            )

            blockhash_resp = await connection.get_latest_blockhash()
            tx.sign([user1], blockhash_resp.value.blockhash)

            result = await connection.send_raw_transaction(bytes(tx))
            await confirm_tx(connection, result.value)

            success("User1 redeemed winnings")

            # ===== Step 16: Set paused test =====
            info("Testing set paused (pause exchange)...")

            tx = await client.set_paused(authority.pubkey(), True)
            blockhash_resp = await connection.get_latest_blockhash()
            tx.sign([authority], blockhash_resp.value.blockhash)

            result = await connection.send_raw_transaction(bytes(tx))
            await confirm_tx(connection, result.value)

            exchange = await client.get_exchange()
            assert exchange.paused == True
            success(f"Exchange paused, paused={exchange.paused}")

            # Unpause
            info("Unpausing exchange...")
            tx = await client.set_paused(authority.pubkey(), False)
            blockhash_resp = await connection.get_latest_blockhash()
            tx.sign([authority], blockhash_resp.value.blockhash)

            result = await connection.send_raw_transaction(bytes(tx))
            await confirm_tx(connection, result.value)

            exchange = await client.get_exchange()
            assert exchange.paused == False
            success(f"Exchange unpaused, paused={exchange.paused}")

            # ===== Step 17: Set operator test =====
            info("Testing set operator...")

            # Create a new operator keypair
            new_operator = Keypair()
            original_operator = exchange.operator

            tx = await client.set_operator(authority.pubkey(), new_operator.pubkey())
            blockhash_resp = await connection.get_latest_blockhash()
            tx.sign([authority], blockhash_resp.value.blockhash)

            result = await connection.send_raw_transaction(bytes(tx))
            await confirm_tx(connection, result.value)

            exchange = await client.get_exchange()
            assert exchange.operator == new_operator.pubkey()
            success(f"Operator changed to {new_operator.pubkey()}")

            # Revert operator back to original
            info("Reverting operator back...")
            tx = await client.set_operator(authority.pubkey(), original_operator)
            blockhash_resp = await connection.get_latest_blockhash()
            tx.sign([authority], blockhash_resp.value.blockhash)

            result = await connection.send_raw_transaction(bytes(tx))
            await confirm_tx(connection, result.value)

            exchange = await client.get_exchange()
            assert exchange.operator == original_operator
            success(f"Operator reverted to {original_operator}")

            # ===== Summary =====
            print("\n" + "=" * 70)
            print("🎉 ALL DEVNET TESTS PASSED!")
            print("=" * 70)
            print("\nAll 14 Instructions Tested:")
            print("   0. INITIALIZE          ✅")
            print("   1. CREATE_MARKET       ✅")
            print("   2. ADD_DEPOSIT_MINT    ✅")
            print("   3. MINT_COMPLETE_SET   ✅")
            print("   4. MERGE_COMPLETE_SET  ✅")
            print("   5. CANCEL_ORDER        ✅")
            print("   6. INCREMENT_NONCE     ✅")
            print("   7. SETTLE_MARKET       ✅")
            print("   8. REDEEM_WINNINGS     ✅")
            print("   9. SET_PAUSED          ✅")
            print("  10. SET_OPERATOR        ✅")
            print("  11. WITHDRAW_FROM_POSITION ✅")
            print("  12. ACTIVATE_MARKET     ✅")
            print("  13. MATCH_ORDERS_MULTI  ✅")
            print(f"\nTest Market ID: {market_id}")
            print(f"Program: {PROGRAM_ID}")

        finally:
            await connection.close()
