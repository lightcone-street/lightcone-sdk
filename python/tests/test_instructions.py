"""Tests for on-chain instruction account layouts."""

from solders.pubkey import Pubkey

from lightcone_sdk.program import (
    DepositToGlobalAltContext,
    InvalidOutcomeCountError,
    InvalidPayoutNumeratorsError,
    ArithmeticOverflowError,
    MakerFill,
    OrderSide,
    OutcomeMetadata,
    SignedOrder,
    build_add_deposit_mint_instruction,
    build_cancel_order_instruction,
    build_create_market_instruction,
    build_create_orderbook_instruction,
    build_deposit_and_swap_instruction,
    build_deposit_to_global_instruction,
    build_extend_position_tokens_instruction,
    build_match_orders_multi_instruction,
    build_redeem_winnings_instruction,
    build_set_manager_instruction,
    build_settle_market_instruction,
    build_withdraw_from_global_instruction,
    derive_condition_id,
    get_associated_token_address,
    get_associated_token_address_2022,
    get_alt_pda,
    get_conditional_mint_pda,
    get_condition_tombstone_pda,
    get_exchange_pda,
    get_global_deposit_pda,
    get_market_pda,
    get_mint_authority_pda,
    get_order_status_pda,
    get_orderbook_pda,
    get_position_pda,
    get_user_nonce_pda,
    get_vault_pda,
    hash_order,
)

import pytest


def fixed_pubkey(value: int) -> Pubkey:
    return Pubkey.from_bytes(bytes([value] * 32))


def signed_order(
    maker: Pubkey,
    market: Pubkey,
    base_mint: Pubkey,
    quote_mint: Pubkey,
    side: OrderSide = OrderSide.BID,
    nonce: int = 1,
) -> SignedOrder:
    return SignedOrder(
        nonce=nonce,
        maker=maker,
        market=market,
        base_mint=base_mint,
        quote_mint=quote_mint,
        side=side,
        amount_in=1_000,
        amount_out=500,
        expiration=1_900_000_000,
        signature=bytes([nonce] * 64),
    )


def test_create_market_uses_manager_and_condition_tombstone():
    manager = Pubkey.new_unique()
    oracle = Pubkey.new_unique()
    question_id = bytes([9] * 32)

    ix = build_create_market_instruction(
        manager=manager,
        market_id=7,
        num_outcomes=2,
        oracle=oracle,
        question_id=question_id,
    )

    condition_id = derive_condition_id(oracle, question_id, 2)
    condition_tombstone, _ = get_condition_tombstone_pda(condition_id)

    assert len(ix.accounts) == 5
    assert ix.accounts[0].pubkey == manager
    assert ix.accounts[0].is_signer is True
    assert ix.accounts[4].pubkey == condition_tombstone
    assert ix.accounts[4].is_writable is True
    assert len(ix.data) == 66


def test_add_deposit_mint_uses_manager_and_global_deposit_token():
    manager = Pubkey.new_unique()
    market = Pubkey.new_unique()
    deposit_mint = Pubkey.new_unique()
    metadata = [
        OutcomeMetadata("Yes", "YES", "https://example.com/yes.json"),
        OutcomeMetadata("No", "NO", "https://example.com/no.json"),
    ]

    ix = build_add_deposit_mint_instruction(
        manager=manager,
        market=market,
        deposit_mint=deposit_mint,
        outcome_metadata=metadata,
        num_outcomes=2,
    )

    global_deposit_token, _ = get_global_deposit_pda(deposit_mint)

    assert len(ix.accounts) == 12
    assert ix.accounts[0].pubkey == manager
    assert ix.accounts[2].pubkey == market
    assert ix.accounts[2].is_writable is False
    assert ix.accounts[9].pubkey == global_deposit_token
    assert ix.accounts[9].is_writable is False


def test_create_orderbook_canonicalizes_mints_and_data():
    manager = Pubkey.new_unique()
    market = Pubkey.new_unique()
    high_mint = fixed_pubkey(9)
    low_mint = fixed_pubkey(1)
    high_deposit_mint = fixed_pubkey(8)
    low_deposit_mint = fixed_pubkey(2)

    ix = build_create_orderbook_instruction(
        manager=manager,
        market=market,
        mint_a=high_mint,
        mint_b=low_mint,
        mint_a_deposit_mint=high_deposit_mint,
        mint_b_deposit_mint=low_deposit_mint,
        recent_slot=123,
        base_index=0,
        mint_a_outcome_index=4,
        mint_b_outcome_index=1,
    )

    orderbook, _ = get_orderbook_pda(low_mint, high_mint)

    assert len(ix.accounts) == 11
    assert ix.accounts[0].pubkey == manager
    assert ix.accounts[2].pubkey == low_mint
    assert ix.accounts[3].pubkey == high_mint
    assert ix.accounts[4].pubkey == orderbook
    assert ix.accounts[9].pubkey == low_deposit_mint
    assert ix.accounts[10].pubkey == high_deposit_mint
    assert len(ix.data) == 12
    assert ix.data[9] == 1
    assert ix.data[10] == 1
    assert ix.data[11] == 4


def test_set_manager_instruction_layout():
    authority = Pubkey.new_unique()
    new_manager = Pubkey.new_unique()

    ix = build_set_manager_instruction(authority, new_manager)

    assert len(ix.accounts) == 2
    assert ix.accounts[0].pubkey == authority
    assert ix.accounts[0].is_signer is True
    assert ix.data == bytes([28]) + bytes(new_manager)


def test_settle_market_uses_payout_vector_layout():
    oracle = Pubkey.new_unique()
    market_id = 7
    exchange, _ = get_exchange_pda()
    market, _ = get_market_pda(market_id)

    ix = build_settle_market_instruction(
        oracle=oracle,
        market_id=market_id,
        payout_numerators=[7, 3],
    )

    assert len(ix.accounts) == 3
    assert ix.accounts[0].pubkey == oracle
    assert ix.accounts[0].is_signer is True
    assert ix.accounts[0].is_writable is False
    assert ix.accounts[1].pubkey == exchange
    assert ix.accounts[2].pubkey == market
    assert ix.accounts[2].is_writable is True
    assert len(ix.data) == 9
    assert ix.data[0] == 7
    assert int.from_bytes(ix.data[1:5], "little") == 7
    assert int.from_bytes(ix.data[5:9], "little") == 3


def test_settle_market_rejects_invalid_payout_vectors():
    oracle = Pubkey.new_unique()

    with pytest.raises(InvalidPayoutNumeratorsError):
        build_settle_market_instruction(oracle, 1, [0, 0])

    with pytest.raises(InvalidOutcomeCountError):
        build_settle_market_instruction(oracle, 1, [1])

    with pytest.raises(ArithmeticOverflowError):
        build_settle_market_instruction(oracle, 1, [0xFFFFFFFF, 1])


def test_cancel_order_uses_operator_exchange_market_status_layout():
    operator = Pubkey.new_unique()
    market = Pubkey.new_unique()
    order = signed_order(
        maker=Pubkey.new_unique(),
        market=market,
        base_mint=Pubkey.new_unique(),
        quote_mint=Pubkey.new_unique(),
    )
    order_status, _ = get_order_status_pda(hash_order(order))
    exchange, _ = get_exchange_pda()

    ix = build_cancel_order_instruction(operator, market, order)

    assert [meta.pubkey for meta in ix.accounts] == [
        operator,
        exchange,
        market,
        order_status,
    ]
    assert ix.accounts[0].is_signer is True


def test_match_orders_multi_includes_orderbook_at_fixed_index():
    operator = Pubkey.new_unique()
    market = Pubkey.new_unique()
    base_mint = Pubkey.new_unique()
    quote_mint = Pubkey.new_unique()
    taker_order = signed_order(Pubkey.new_unique(), market, base_mint, quote_mint)
    maker_order = signed_order(
        Pubkey.new_unique(), market, base_mint, quote_mint, OrderSide.ASK, nonce=2
    )
    orderbook, _ = get_orderbook_pda(base_mint, quote_mint)

    ix = build_match_orders_multi_instruction(
        operator=operator,
        market=market,
        base_mint=base_mint,
        quote_mint=quote_mint,
        taker_order=taker_order,
        maker_orders=[maker_order],
        maker_fill_amounts=[100],
        taker_fill_amounts=[50],
    )

    assert ix.accounts[3].pubkey == orderbook
    assert ix.accounts[3].is_writable is False


def test_deposit_and_swap_includes_orderbook_at_fixed_index():
    operator = Pubkey.new_unique()
    market = Pubkey.new_unique()
    base_mint = Pubkey.new_unique()
    quote_mint = Pubkey.new_unique()
    taker_order = signed_order(Pubkey.new_unique(), market, base_mint, quote_mint)
    maker_order = signed_order(
        Pubkey.new_unique(), market, base_mint, quote_mint, OrderSide.ASK, nonce=2
    )
    orderbook, _ = get_orderbook_pda(base_mint, quote_mint)

    ix = build_deposit_and_swap_instruction(
        operator=operator,
        market=market,
        base_mint=base_mint,
        quote_mint=quote_mint,
        taker_order=taker_order,
        makers=[
            MakerFill(
                order=maker_order,
                maker_fill_amount=100,
                taker_fill_amount=50,
                deposit_mint=Pubkey.new_unique(),
            )
        ],
    )

    assert ix.accounts[3].pubkey == orderbook


def test_deposit_to_global_includes_exchange_and_optional_alt_context():
    user = Pubkey.new_unique()
    mint = Pubkey.new_unique()
    exchange, _ = get_exchange_pda()

    ix = build_deposit_to_global_instruction(user, mint, 1_000)

    assert len(ix.accounts) == 8
    assert ix.accounts[7].pubkey == exchange
    assert len(ix.data) == 9

    alt_ix = build_deposit_to_global_instruction(
        user,
        mint,
        1_000,
        alt_context=DepositToGlobalAltContext.create(123),
    )
    user_nonce, _ = get_user_nonce_pda(user)
    lookup_table, _ = get_alt_pda(user_nonce, 123)

    assert len(alt_ix.accounts) == 11
    assert alt_ix.accounts[8].pubkey == user_nonce
    assert alt_ix.accounts[9].pubkey == lookup_table
    assert len(alt_ix.data) == 17


def test_withdraw_from_global_includes_exchange():
    user = Pubkey.new_unique()
    mint = Pubkey.new_unique()
    exchange, _ = get_exchange_pda()

    ix = build_withdraw_from_global_instruction(user, mint, 1_000)

    assert len(ix.accounts) == 7
    assert ix.accounts[6].pubkey == exchange


def test_redeem_winnings_uses_outcome_index_and_exchange():
    user = Pubkey.new_unique()
    market = Pubkey.new_unique()
    deposit_mint = Pubkey.new_unique()
    outcome_index = 2
    exchange, _ = get_exchange_pda()
    vault, _ = get_vault_pda(deposit_mint, market)
    conditional_mint, _ = get_conditional_mint_pda(market, deposit_mint, outcome_index)
    position, _ = get_position_pda(user, market)
    position_conditional_ata = get_associated_token_address_2022(
        position, conditional_mint
    )
    user_deposit_ata = get_associated_token_address(user, deposit_mint)
    mint_authority, _ = get_mint_authority_pda(market)

    ix = build_redeem_winnings_instruction(
        user=user,
        market=market,
        deposit_mint=deposit_mint,
        outcome_index=outcome_index,
        amount=123,
    )

    assert len(ix.accounts) == 12
    assert ix.accounts[3].pubkey == vault
    assert ix.accounts[4].pubkey == conditional_mint
    assert ix.accounts[5].pubkey == position
    assert ix.accounts[5].is_writable is False
    assert ix.accounts[6].pubkey == position_conditional_ata
    assert ix.accounts[7].pubkey == user_deposit_ata
    assert ix.accounts[8].pubkey == mint_authority
    assert ix.accounts[11].pubkey == exchange
    assert len(ix.data) == 10
    assert ix.data[0] == 8
    assert int.from_bytes(ix.data[1:9], "little") == 123
    assert ix.data[9] == outcome_index


def test_extend_position_tokens_uses_operator_signer():
    operator = Pubkey.new_unique()

    ix = build_extend_position_tokens_instruction(
        operator=operator,
        user=Pubkey.new_unique(),
        market=Pubkey.new_unique(),
        lookup_table=Pubkey.new_unique(),
        deposit_mints=[Pubkey.new_unique()],
        num_outcomes=2,
    )

    assert ix.accounts[0].pubkey == operator
    assert ix.accounts[0].is_signer is True
