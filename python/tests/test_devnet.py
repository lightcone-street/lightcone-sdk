"""
Devnet integration tests for the Lightcone SDK.

These tests run against Solana devnet and test the full end-to-end flow
of the Lightcone protocol, mirroring the Rust SDK devnet integration tests.

To run:
    DEVNET_TESTS=1 uv run pytest tests/test_devnet.py -v -s

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
from solana.rpc.commitment import Confirmed, Finalized
from solana.rpc.types import TxOpts
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
    CreateOrderbookParams,
    LightconePinocchioClient,
    MarketStatus,
    MatchOrdersMultiParams,
    MergeCompleteSetParams,
    MintCompleteSetParams,
    OutcomeMetadata,
    RedeemWinningsParams,
    SetAuthorityParams,
    SettleMarketParams,
    WithdrawFromPositionParams,
    build_add_deposit_mint_instruction,
    get_conditional_mint_pda,
    get_position_pda,
    hash_order,
    hash_order_hex,
    verify_order_signature,
    to_order,
)
from src.program.utils import get_associated_token_address, get_associated_token_address_2022

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


async def sign_and_send(connection: AsyncClient, tx: Transaction, signers: list[Keypair]) -> Signature:
    """Sign and send a transaction, returning the signature."""
    blockhash_resp = await connection.get_latest_blockhash()
    blockhash = blockhash_resp.value.blockhash
    tx.sign(signers, blockhash)
    result = await connection.send_raw_transaction(bytes(tx))
    await confirm_tx(connection, result.value)
    return result.value


@pytest.mark.asyncio
class TestDevnetIntegration:
    """Integration tests running against Solana devnet.

    Mirrors the Rust SDK devnet integration tests (rust/tests/devnet_integration.rs).
    Tests are structured as a single flow to ensure proper ordering.
    """

    async def test_full_market_lifecycle(self):
        """
        Complete end-to-end test of the Lightcone protocol:
        1.  Check protocol state
        2.  Initialize (if needed)
        3.  Create market
        4.  Create deposit mint & add to market
        5.  Create orderbook
        6.  Activate market
        7.  User1 deposit flow (mint complete set)
        8.  User2 setup & deposit
        9.  User3 setup & deposit (for multi-maker)
        10. Merge complete set
        11. Withdraw from position
        12. Order creation, signing & roundtrip
        13. Cancel order
        14. Match orders multi (partial fill, bitmask=0)
        15. Match orders multi (full fill with bitmask)
        16. Match orders multi (2 makers)
        17. Settle market
        18. Redeem winnings
        19. Set paused
        20. Set operator
        21. Set authority
        """
        # Setup
        connection = AsyncClient(DEVNET_RPC, commitment=Confirmed)
        authority = get_authority_keypair()
        client = LightconePinocchioClient(connection, PROGRAM_ID)

        user1 = Keypair()
        user2 = Keypair()
        user3 = Keypair()

        try:
            print(f"\n{'=' * 70}")
            print("LIGHTCONE PYTHON SDK - DEVNET INTEGRATION TESTS")
            print(f"{'=' * 70}\n")

            info(f"Program ID: {PROGRAM_ID}")
            info(f"Authority: {authority.pubkey()}")

            # ===== Step 1: Check protocol state =====
            print("\n1. Checking protocol state...")
            await airdrop_if_needed(connection, authority.pubkey(), 2 * LAMPORTS_PER_SOL)

            exchange = None
            is_initialized = False
            initial_market_count = 0
            try:
                exchange = await client.get_exchange()
                is_initialized = True
                initial_market_count = exchange.market_count
                success(f"Protocol initialized, market_count={exchange.market_count}")
                info(f"Authority: {exchange.authority}")
                info(f"Operator: {exchange.operator}")
                info(f"Paused: {exchange.paused}")

                if exchange.authority != authority.pubkey():
                    warn(f"Exchange authority ({exchange.authority}) != our keypair ({authority.pubkey()})")
                    pytest.skip("Not the exchange authority - cannot create markets")
            except Exception:
                info("Protocol not initialized - will initialize")

            # ===== Step 2: Initialize (if needed) =====
            if not is_initialized:
                print("\n2. Initializing protocol...")
                tx = await client.initialize(authority.pubkey())
                sig = await sign_and_send(connection, tx, [authority])
                success(f"Protocol initialized, sig={sig}")
            else:
                print("\n2. Skipping initialization (already done)")

            # Refresh state
            exchange = await client.get_exchange()
            market_id = exchange.market_count
            info(f"Next Market ID: {market_id}")

            # ===== Step 3: Create market =====
            print("\n3. Creating market...")
            question_id = bytes([market_id % 256] * 32)

            tx = await client.create_market(
                authority=authority.pubkey(),
                num_outcomes=2,
                oracle=authority.pubkey(),
                question_id=question_id,
            )
            sig = await sign_and_send(connection, tx, [authority])

            market_pda = client.get_market_address(market_id)
            market = await client.get_market(market_id)
            success(f"Market {market_id} created, PDA={market_pda}")
            assert market.market_id == market_id
            assert market.num_outcomes == 2
            assert market.status == MarketStatus.PENDING

            # ===== Step 4: Create deposit mint & add to market =====
            print("\n4. Creating deposit mint and adding to market...")
            token = await AsyncToken.create_mint(
                connection,
                authority,
                authority.pubkey(),
                TOKEN_DECIMALS,
                SPL_TOKEN_PROGRAM_ID,
            )
            deposit_mint = token.pubkey
            success(f"Deposit mint created: {deposit_mint}")

            outcome_metadata = [
                OutcomeMetadata(name="YES-TOKEN", symbol="YES", uri="https://arweave.net/test-yes.json"),
                OutcomeMetadata(name="NO-TOKEN", symbol="NO", uri="https://arweave.net/test-no.json"),
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

            conditional_mints = client.get_conditional_mints(market_pda, deposit_mint, 2)
            info(f"Conditional Mint 0 (YES): {conditional_mints[0]}")
            info(f"Conditional Mint 1 (NO): {conditional_mints[1]}")
            success("Deposit mint added to market")

            # ===== Step 5: Create orderbook =====
            print("\n5. Creating orderbook...")

            # Determine canonical order: mint_a < mint_b
            if bytes(conditional_mints[0]) < bytes(conditional_mints[1]):
                mint_a, mint_b = conditional_mints[0], conditional_mints[1]
            else:
                mint_a, mint_b = conditional_mints[1], conditional_mints[0]

            # Get a recent finalized slot for the ALT program
            slot_resp = await connection.get_slot(Finalized)
            recent_slot = slot_resp.value
            info(f"Using slot: {recent_slot} (finalized)")

            tx = await client.create_orderbook(
                CreateOrderbookParams(
                    payer=authority.pubkey(),
                    market=market_pda,
                    mint_a=mint_a,
                    mint_b=mint_b,
                    recent_slot=recent_slot,
                )
            )

            # Skip preflight for ALT slot validation (same as Rust test)
            blockhash_resp = await connection.get_latest_blockhash()
            tx.sign([authority], blockhash_resp.value.blockhash)
            result = await connection.send_raw_transaction(
                bytes(tx),
                opts=TxOpts(skip_preflight=True),
            )
            info(f"Sent tx (skip_preflight): {result.value}")

            await asyncio.sleep(3)
            await confirm_tx(connection, result.value)

            # Verify orderbook was created
            orderbook = await client.get_orderbook(mint_a, mint_b)
            assert orderbook is not None
            info(f"Orderbook Market: {orderbook.market}")
            info(f"Orderbook Mint A: {orderbook.mint_a}")
            info(f"Orderbook Mint B: {orderbook.mint_b}")
            info(f"Orderbook Lookup Table: {orderbook.lookup_table}")
            assert orderbook.market == market_pda
            assert orderbook.mint_a == mint_a
            assert orderbook.mint_b == mint_b
            success("Orderbook created")

            # ===== Step 6: Activate market =====
            print("\n6. Activating market...")
            tx = await client.activate_market(
                ActivateMarketParams(
                    authority=authority.pubkey(),
                    market_id=market_id,
                )
            )
            sig = await sign_and_send(connection, tx, [authority])

            market = await client.get_market(market_id)
            success(f"Market activated, status={market.status.name}")
            assert market.status == MarketStatus.ACTIVE

            # ===== Step 7: User1 deposit flow =====
            print("\n7. Setting up user1 and minting complete set...")
            info(f"User1: {user1.pubkey()}")

            await fund_account(connection, authority, user1.pubkey(), int(0.1 * LAMPORTS_PER_SOL))

            token_client = AsyncToken(connection, deposit_mint, SPL_TOKEN_PROGRAM_ID, authority)
            user1_ata = await token_client.create_associated_token_account(user1.pubkey())
            await token_client.mint_to(user1_ata, authority, TOKENS_TO_MINT)
            await asyncio.sleep(2)
            success(f"User1 funded with {TOKENS_TO_MINT / 10**TOKEN_DECIMALS} tokens")

            tx = await client.mint_complete_set(
                MintCompleteSetParams(
                    user=user1.pubkey(),
                    market=market_pda,
                    deposit_mint=deposit_mint,
                    amount=COMPLETE_SET_AMOUNT,
                ),
                num_outcomes=2,
            )
            sig = await sign_and_send(connection, tx, [user1])

            position = await client.get_position(user1.pubkey(), market_pda)
            assert position is not None
            success("User1 minted complete set")

            # ===== Step 8: User2 setup & deposit =====
            print("\n8. Setting up user2...")
            info(f"User2: {user2.pubkey()}")

            await fund_account(connection, authority, user2.pubkey(), int(0.1 * LAMPORTS_PER_SOL))
            user2_ata = await token_client.create_associated_token_account(user2.pubkey())
            await token_client.mint_to(user2_ata, authority, TOKENS_TO_MINT * 2)
            await asyncio.sleep(2)
            success(f"User2 funded with {TOKENS_TO_MINT * 2 / 10**TOKEN_DECIMALS} tokens")

            tx = await client.mint_complete_set(
                MintCompleteSetParams(
                    user=user2.pubkey(),
                    market=market_pda,
                    deposit_mint=deposit_mint,
                    amount=COMPLETE_SET_AMOUNT * 5,
                ),
                num_outcomes=2,
            )
            sig = await sign_and_send(connection, tx, [user2])
            success("User2 minted complete set")

            # ===== Step 9: User3 setup & deposit (for multi-maker) =====
            print("\n9. Setting up user3 (for multi-maker matching)...")
            info(f"User3: {user3.pubkey()}")

            await fund_account(connection, authority, user3.pubkey(), int(0.1 * LAMPORTS_PER_SOL))
            user3_ata = await token_client.create_associated_token_account(user3.pubkey())
            await token_client.mint_to(user3_ata, authority, TOKENS_TO_MINT * 2)
            await asyncio.sleep(2)
            success(f"User3 funded with {TOKENS_TO_MINT * 2 / 10**TOKEN_DECIMALS} tokens")

            tx = await client.mint_complete_set(
                MintCompleteSetParams(
                    user=user3.pubkey(),
                    market=market_pda,
                    deposit_mint=deposit_mint,
                    amount=COMPLETE_SET_AMOUNT * 5,
                ),
                num_outcomes=2,
            )
            sig = await sign_and_send(connection, tx, [user3])
            success("User3 minted complete set")

            # ===== Step 10: Merge complete set =====
            print("\n10. Testing merge complete set...")
            tx = await client.merge_complete_set(
                MergeCompleteSetParams(
                    user=user2.pubkey(),
                    market=market_pda,
                    deposit_mint=deposit_mint,
                    amount=COMPLETE_SET_AMOUNT,
                ),
                num_outcomes=2,
            )
            sig = await sign_and_send(connection, tx, [user2])
            success("User2 merged complete set")

            # Get conditional mint addresses
            yes_mint, _ = get_conditional_mint_pda(market_pda, deposit_mint, 0)
            no_mint, _ = get_conditional_mint_pda(market_pda, deposit_mint, 1)

            # ===== Step 11: Withdraw from position =====
            print("\n11. Testing withdraw from position...")

            # Create user2's personal ATA for conditional token (Token-2022)
            from spl.token.instructions import create_associated_token_account
            TOKEN_2022_PROG_ID = Pubkey.from_string("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb")

            user2_yes_ata = get_associated_token_address_2022(user2.pubkey(), yes_mint)

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
            info(f"User2 conditional ATA created: {user2_yes_ata}")

            tx = await client.withdraw_from_position(
                WithdrawFromPositionParams(
                    user=user2.pubkey(),
                    market=market_pda,
                    mint=yes_mint,
                    amount=100_000,
                    outcome_index=0,
                ),
                is_token_2022=True,
            )
            sig = await sign_and_send(connection, tx, [user2])
            success("User2 withdrew conditional tokens from position")

            # ===== Step 12: Order creation, signing & roundtrip =====
            print("\n12. Testing order creation and signing...")

            expiration = int(time.time()) + 3600  # 1 hour

            user1_nonce_val = await client.get_next_nonce(user1.pubkey())
            info(f"User1 Nonce: {user1_nonce_val}")

            bid_order = client.create_signed_bid_order(
                BidOrderParams(
                    nonce=user1_nonce_val,
                    maker=user1.pubkey(),
                    market=market_pda,
                    base_mint=yes_mint,
                    quote_mint=no_mint,
                    maker_amount=100,
                    taker_amount=50,
                    expiration=expiration,
                ),
                user1,
            )
            info(f"BID Order Hash: {hash_order_hex(bid_order)[:32]}...")

            ask_order = client.create_signed_ask_order(
                AskOrderParams(
                    nonce=user1_nonce_val + 1,
                    maker=user1.pubkey(),
                    market=market_pda,
                    base_mint=yes_mint,
                    quote_mint=no_mint,
                    maker_amount=50,
                    taker_amount=100,
                    expiration=expiration,
                ),
                user1,
            )
            info(f"ASK Order Hash: {hash_order_hex(ask_order)[:32]}...")

            # Verify signatures
            assert verify_order_signature(bid_order), "BID signature should be valid"
            assert verify_order_signature(ask_order), "ASK signature should be valid"

            # Test Order <-> FullOrder roundtrip
            compact = to_order(bid_order)
            assert compact.nonce == bid_order.nonce & 0xFFFFFFFF
            assert compact.maker_amount == bid_order.maker_amount
            info("Order <-> FullOrder roundtrip verified")

            success("Orders created and signed")

            # ===== Step 13: Cancel order =====
            print("\n13. Testing cancel order...")

            tx_nonce = await client.increment_nonce(user1.pubkey())
            sig = await sign_and_send(connection, tx_nonce, [user1])
            await asyncio.sleep(2)

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

            tx = await client.cancel_order(user1.pubkey(), market_pda, cancel_order)
            sig = await sign_and_send(connection, tx, [user1])

            cancel_status = await client.get_order_status(hash_order(cancel_order))
            assert cancel_status is not None
            assert cancel_status.is_cancelled == True
            success(f"Order cancelled, is_cancelled={cancel_status.is_cancelled}")

            # ===== Step 14: Match orders multi (partial fill, bitmask=0) =====
            print("\n14. Testing match orders multi (partial fill, bitmask=0)...")

            tx1 = await client.increment_nonce(user1.pubkey())
            await sign_and_send(connection, tx1, [user1])

            tx2 = await client.increment_nonce(user2.pubkey())
            await sign_and_send(connection, tx2, [user2])

            await asyncio.sleep(2)

            u1_nonce = await client.get_next_nonce(user1.pubkey())
            u2_nonce = await client.get_next_nonce(user2.pubkey())
            info(f"User1 Nonce: {u1_nonce}, User2 Nonce: {u2_nonce}")

            # User1 BID: wants YES tokens, pays NO tokens
            match_bid = client.create_signed_bid_order(
                BidOrderParams(
                    nonce=u1_nonce,
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
            info(f"BID Order (User1): {hash_order_hex(match_bid)[:32]}...")

            # User2 ASK: sells YES tokens, wants NO tokens
            match_ask = client.create_signed_ask_order(
                AskOrderParams(
                    nonce=u2_nonce,
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
            info(f"ASK Order (User2): {hash_order_hex(match_ask)[:32]}...")

            # Partial fill: 50_000 out of 100_000 (bitmask=0 means order_status tracked)
            tx = await client.match_orders_multi(
                MatchOrdersMultiParams(
                    operator=exchange.operator,
                    market=market_pda,
                    base_mint=yes_mint,
                    quote_mint=no_mint,
                    taker_order=match_bid,
                    maker_orders=[match_ask],
                    maker_fill_amounts=[50_000],
                    taker_fill_amounts=[50_000],
                    full_fill_bitmask=0,
                )
            )
            sig = await sign_and_send(connection, tx, [authority])

            # Verify partial fill remaining
            taker_status = await client.get_order_status(hash_order(match_bid))
            if taker_status:
                info(f"Taker Remaining: {taker_status.remaining}")
                assert taker_status.remaining == 50_000

            maker_status = await client.get_order_status(hash_order(match_ask))
            if maker_status:
                info(f"Maker Remaining: {maker_status.remaining}")
                assert maker_status.remaining == 50_000

            success("Partial fill match completed")

            # ===== Step 15: Match orders multi (full fill with bitmask) =====
            print("\n15. Testing match orders multi (full fill with bitmask)...")

            tx1 = await client.increment_nonce(user1.pubkey())
            await sign_and_send(connection, tx1, [user1])

            tx2 = await client.increment_nonce(user2.pubkey())
            await sign_and_send(connection, tx2, [user2])

            await asyncio.sleep(2)

            u1n = await client.get_next_nonce(user1.pubkey())
            u2n = await client.get_next_nonce(user2.pubkey())

            ff_bid = client.create_signed_bid_order(
                BidOrderParams(
                    nonce=u1n,
                    maker=user1.pubkey(),
                    market=market_pda,
                    base_mint=yes_mint,
                    quote_mint=no_mint,
                    maker_amount=80_000,
                    taker_amount=80_000,
                    expiration=expiration,
                ),
                user1,
            )

            ff_ask = client.create_signed_ask_order(
                AskOrderParams(
                    nonce=u2n,
                    maker=user2.pubkey(),
                    market=market_pda,
                    base_mint=yes_mint,
                    quote_mint=no_mint,
                    maker_amount=80_000,
                    taker_amount=80_000,
                    expiration=expiration,
                ),
                user2,
            )

            # bit 0 = maker full fill, bit 7 = taker full fill = 0b10000001 = 0x81
            tx = await client.match_orders_multi(
                MatchOrdersMultiParams(
                    operator=exchange.operator,
                    market=market_pda,
                    base_mint=yes_mint,
                    quote_mint=no_mint,
                    taker_order=ff_bid,
                    maker_orders=[ff_ask],
                    maker_fill_amounts=[80_000],
                    taker_fill_amounts=[80_000],
                    full_fill_bitmask=0b10000001,
                )
            )
            sig = await sign_and_send(connection, tx, [authority])
            success("Full fill match with bitmask completed")

            # ===== Step 16: Match orders multi (2 makers) =====
            print("\n16. Testing match orders multi (2 makers)...")

            tx1 = await client.increment_nonce(user1.pubkey())
            await sign_and_send(connection, tx1, [user1])

            tx2 = await client.increment_nonce(user2.pubkey())
            await sign_and_send(connection, tx2, [user2])

            tx3 = await client.increment_nonce(user3.pubkey())
            await sign_and_send(connection, tx3, [user3])

            await asyncio.sleep(2)

            u1_nonce = await client.get_next_nonce(user1.pubkey())
            u2_nonce = await client.get_next_nonce(user2.pubkey())
            u3_nonce = await client.get_next_nonce(user3.pubkey())
            info(f"User1 Nonce: {u1_nonce}, User2 Nonce: {u2_nonce}, User3 Nonce: {u3_nonce}")

            # User1 BID: wants 200_000 YES
            taker_bid = client.create_signed_bid_order(
                BidOrderParams(
                    nonce=u1_nonce,
                    maker=user1.pubkey(),
                    market=market_pda,
                    base_mint=yes_mint,
                    quote_mint=no_mint,
                    maker_amount=200_000,
                    taker_amount=200_000,
                    expiration=expiration,
                ),
                user1,
            )

            # User2 ASK: sells 100_000 YES
            maker1_ask = client.create_signed_ask_order(
                AskOrderParams(
                    nonce=u2_nonce,
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

            # User3 ASK: sells 100_000 YES
            maker2_ask = client.create_signed_ask_order(
                AskOrderParams(
                    nonce=u3_nonce,
                    maker=user3.pubkey(),
                    market=market_pda,
                    base_mint=yes_mint,
                    quote_mint=no_mint,
                    maker_amount=100_000,
                    taker_amount=100_000,
                    expiration=expiration,
                ),
                user3,
            )

            # bit 0 = maker0 full, bit 1 = maker1 full, bit 7 = taker full = 0b10000011
            tx = await client.match_orders_multi(
                MatchOrdersMultiParams(
                    operator=exchange.operator,
                    market=market_pda,
                    base_mint=yes_mint,
                    quote_mint=no_mint,
                    taker_order=taker_bid,
                    maker_orders=[maker1_ask, maker2_ask],
                    maker_fill_amounts=[100_000, 100_000],
                    taker_fill_amounts=[100_000, 100_000],
                    full_fill_bitmask=0b10000011,
                )
            )
            sig = await sign_and_send(connection, tx, [authority])
            success("Multi-maker match completed")

            # ===== Step 17: Settle market =====
            print("\n17. Settling market (outcome 0 = YES wins)...")
            tx = await client.settle_market(
                SettleMarketParams(
                    oracle=authority.pubkey(),
                    market_id=market_id,
                    winning_outcome=0,
                )
            )
            sig = await sign_and_send(connection, tx, [authority])

            market = await client.get_market(market_id)
            success(f"Market settled, status={market.status.name}, winner={market.winning_outcome}")
            assert market.status == MarketStatus.RESOLVED
            assert market.winning_outcome == 0

            # ===== Step 18: Redeem winnings =====
            print("\n18. User1 redeeming winnings...")
            tx = await client.redeem_winnings(
                RedeemWinningsParams(
                    user=user1.pubkey(),
                    market=market_pda,
                    deposit_mint=deposit_mint,
                    amount=100_000,
                ),
                winning_outcome=0,
            )
            sig = await sign_and_send(connection, tx, [user1])
            success("User1 redeemed winnings")

            # ===== Step 19: Set paused =====
            print("\n19. Testing set paused...")
            tx = await client.set_paused(authority.pubkey(), True)
            sig = await sign_and_send(connection, tx, [authority])

            exchange = await client.get_exchange()
            assert exchange.paused == True
            success(f"Exchange paused, paused={exchange.paused}")

            # Unpause
            tx = await client.set_paused(authority.pubkey(), False)
            sig = await sign_and_send(connection, tx, [authority])

            exchange = await client.get_exchange()
            assert exchange.paused == False
            success(f"Exchange unpaused, paused={exchange.paused}")

            # ===== Step 20: Set operator =====
            print("\n20. Testing set operator...")
            original_operator = exchange.operator

            new_operator = Keypair()
            info(f"New Operator: {new_operator.pubkey()}")

            tx = await client.set_operator(authority.pubkey(), new_operator.pubkey())
            sig = await sign_and_send(connection, tx, [authority])

            exchange = await client.get_exchange()
            assert exchange.operator == new_operator.pubkey()
            success(f"Operator changed to {new_operator.pubkey()}")

            # Revert operator
            tx = await client.set_operator(authority.pubkey(), original_operator)
            sig = await sign_and_send(connection, tx, [authority])

            exchange = await client.get_exchange()
            assert exchange.operator == original_operator
            success(f"Operator reverted to {original_operator}")

            # ===== Step 21: Set authority =====
            print("\n21. Testing set authority...")
            new_authority_kp = Keypair()
            info(f"New Authority: {new_authority_kp.pubkey()}")

            # Transfer authority
            tx = await client.set_authority(
                SetAuthorityParams(
                    current_authority=authority.pubkey(),
                    new_authority=new_authority_kp.pubkey(),
                )
            )
            sig = await sign_and_send(connection, tx, [authority])

            exchange = await client.get_exchange()
            assert exchange.authority == new_authority_kp.pubkey()
            success(f"Authority changed to {exchange.authority}")

            # Transfer authority back (need to fund new authority first)
            await fund_account(connection, authority, new_authority_kp.pubkey(), int(0.01 * LAMPORTS_PER_SOL))

            tx = await client.set_authority(
                SetAuthorityParams(
                    current_authority=new_authority_kp.pubkey(),
                    new_authority=authority.pubkey(),
                )
            )
            sig = await sign_and_send(connection, tx, [new_authority_kp])

            exchange = await client.get_exchange()
            assert exchange.authority == authority.pubkey()
            success(f"Authority reverted to {exchange.authority}")

            # ===== Summary =====
            print(f"\n{'=' * 70}")
            print("🎉 ALL DEVNET TESTS PASSED!")
            print(f"{'=' * 70}")
            print("\nAll Instructions Tested:")
            print("   1.  INITIALIZE              ✅")
            print("   2.  CREATE_MARKET            ✅")
            print("   3.  ADD_DEPOSIT_MINT         ✅")
            print("   4.  CREATE_ORDERBOOK         ✅ (NEW)")
            print("   5.  ACTIVATE_MARKET          ✅")
            print("   6.  MINT_COMPLETE_SET        ✅")
            print("   7.  MERGE_COMPLETE_SET       ✅")
            print("   8.  WITHDRAW_FROM_POSITION   ✅")
            print("   9.  CANCEL_ORDER             ✅")
            print("  10.  INCREMENT_NONCE          ✅")
            print("  11.  MATCH_ORDERS_MULTI       ✅ (partial fill)")
            print("  12.  MATCH_ORDERS_MULTI       ✅ (full fill, bitmask)")
            print("  13.  MATCH_ORDERS_MULTI       ✅ (2 makers)")
            print("  14.  SETTLE_MARKET            ✅")
            print("  15.  REDEEM_WINNINGS          ✅")
            print("  16.  SET_PAUSED               ✅")
            print("  17.  SET_OPERATOR             ✅")
            print("  18.  SET_AUTHORITY            ✅ (NEW)")
            print(f"\nTest Market ID: {market_id}")
            print(f"Program: {PROGRAM_ID}")

        finally:
            await connection.close()
