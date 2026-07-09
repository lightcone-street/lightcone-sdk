"""Tests for on-chain instruction account layouts."""

import pytest
from solders.pubkey import Pubkey

from lightcone_sdk.program import (
    ALT_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID,
    TOKEN_PROGRAM_ID,
    AcceptRoleParams,
    ArithmeticOverflowError,
    CloseOrderbookAltParams,
    CloseOrderbookParams,
    CloseOrderStatusParams,
    ClosePositionAltParams,
    ClosePositionTokenAccountsParams,
    ConditionalMetadataParams,
    DepositToGlobalAltContext,
    InvalidOracleError,
    InvalidOutcomeCountError,
    InvalidOutcomeIndexError,
    InvalidPayoutNumeratorsError,
    MakerFill,
    MarketFeeUpdate,
    MissingFieldError,
    OrderSide,
    RefreshOrderbookAltParams,
    SetDepositTokenStatusParams,
    SetFeeReceiverParams,
    SetFeeReceiverWithAtasParams,
    SetMarketFeesParams,
    SetOracleParams,
    SignedOrder,
    build_accept_authority_instruction,
    build_accept_manager_instruction,
    build_accept_operator_instruction,
    build_add_deposit_mint_instruction,
    build_cancel_order_instruction,
    build_close_order_status_instruction,
    build_close_orderbook_alt_instruction,
    build_close_orderbook_instruction,
    build_close_position_alt_instruction,
    build_close_position_token_accounts_instruction,
    build_create_conditional_metadata_instruction,
    build_create_market_instruction,
    build_create_orderbook_instruction,
    build_deposit_and_swap_instruction,
    build_deposit_to_global_instruction,
    build_extend_position_tokens_instruction,
    build_global_to_market_deposit_instruction,
    build_increment_nonce_instruction,
    build_match_orders_multi_instruction,
    build_mint_complete_set_instruction,
    build_redeem_winnings_instruction,
    build_refresh_orderbook_alt_instruction,
    build_set_deposit_token_status_instruction,
    build_set_fee_receiver_instruction,
    build_set_fee_receiver_with_atas_instruction,
    build_set_manager_instruction,
    build_set_market_fees_instruction,
    build_set_oracle_instruction,
    build_settle_market_instruction,
    build_update_conditional_metadata_instruction,
    build_withdraw_conditional_from_position_instruction,
    build_withdraw_from_global_instruction,
    build_withdraw_from_position_instruction,
    derive_condition_id,
    get_alt_pda,
    get_associated_token_address,
    get_condition_tombstone_pda,
    get_conditional_mint_pda,
    get_conditional_token_ata,
    get_exchange_pda,
    get_global_deposit_pda,
    get_market_pda,
    get_mint_authority_pda,
    get_mpl_metadata_pda,
    get_order_status_pda,
    get_orderbook_pda,
    get_position_pda,
    get_user_global_deposit_pda,
    get_user_nonce_pda,
    get_vault_pda,
    hash_order,
)


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
        maker_fee_bps=10,
        taker_fee_bps=20,
    )

    condition_id = derive_condition_id(oracle, question_id, 2)
    condition_tombstone, _ = get_condition_tombstone_pda(condition_id)

    assert len(ix.accounts) == 5
    assert ix.accounts[0].pubkey == manager
    assert ix.accounts[0].is_signer is True
    assert ix.accounts[4].pubkey == condition_tombstone
    assert ix.accounts[4].is_writable is True
    assert len(ix.data) == 70
    assert int.from_bytes(ix.data[66:68], "little", signed=True) == 10
    assert int.from_bytes(ix.data[68:70], "little", signed=True) == 20


def test_add_deposit_mint_uses_manager_and_global_deposit_token():
    manager = Pubkey.new_unique()
    market = Pubkey.new_unique()
    deposit_mint = Pubkey.new_unique()

    ix = build_add_deposit_mint_instruction(
        manager=manager,
        market=market,
        deposit_mint=deposit_mint,
        num_outcomes=2,
    )

    global_deposit_token, _ = get_global_deposit_pda(deposit_mint)

    assert len(ix.accounts) == 11
    assert ix.accounts[0].pubkey == manager
    assert ix.accounts[2].pubkey == market
    assert ix.accounts[2].is_writable is False
    assert ix.accounts[8].pubkey == global_deposit_token
    assert ix.accounts[8].is_writable is False
    assert ix.data == bytes([2])


def test_mint_complete_set_matches_canonical_account_layout():
    user = Pubkey.new_unique()
    market = Pubkey.new_unique()
    deposit_mint = Pubkey.new_unique()
    position, _ = get_position_pda(user, market)
    mint_authority, _ = get_mint_authority_pda(market)

    ix = build_mint_complete_set_instruction(
        user=user,
        market=market,
        deposit_mint=deposit_mint,
        amount=1_000,
        num_outcomes=3,
    )

    assert len(ix.accounts) == 17
    assert ix.accounts[6].pubkey == position
    assert ix.accounts[7].pubkey == mint_authority
    assert ix.accounts[7].is_writable is False
    assert (
        ix.accounts[11].pubkey
        == get_conditional_mint_pda(
            market,
            deposit_mint,
            0,
        )[0]
    )
    assert len(ix.data) == 9
    assert ix.data[0] == 3


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
        fee_receiver=fixed_pubkey(7),
        mint_a_deposit_mint=high_deposit_mint,
        mint_b_deposit_mint=low_deposit_mint,
        recent_slot=123,
        base_index=0,
        mint_a_outcome_index=4,
        mint_b_outcome_index=1,
    )

    orderbook, _ = get_orderbook_pda(low_mint, high_mint)

    assert len(ix.accounts) == 15
    assert ix.accounts[0].pubkey == manager
    assert ix.accounts[2].pubkey == low_mint
    assert ix.accounts[3].pubkey == high_mint
    assert ix.accounts[4].pubkey == orderbook
    assert ix.accounts[9].pubkey == low_deposit_mint
    assert ix.accounts[10].pubkey == high_deposit_mint
    assert ix.accounts[13].pubkey == fixed_pubkey(7)
    assert len(ix.data) == 12
    assert ix.data[9] == 1
    assert ix.data[10] == 1
    assert ix.data[11] == 4


def test_refresh_orderbook_alt_instruction_layout():
    manager = Pubkey.new_unique()
    market = Pubkey.new_unique()
    orderbook = Pubkey.new_unique()
    lookup_table = Pubkey.new_unique()
    quote_mint = Pubkey.new_unique()
    fee_receiver = Pubkey.new_unique()

    ix = build_refresh_orderbook_alt_instruction(
        RefreshOrderbookAltParams(
            manager=manager,
            market=market,
            orderbook=orderbook,
            lookup_table=lookup_table,
            quote_mint=quote_mint,
            fee_receiver=fee_receiver,
        )
    )

    assert len(ix.accounts) == 12
    assert ix.accounts[0].pubkey == manager
    assert ix.accounts[0].is_signer is True
    assert ix.accounts[0].is_writable is True
    assert ix.accounts[2].pubkey == market
    assert ix.accounts[3].pubkey == orderbook
    assert ix.accounts[4].pubkey == lookup_table
    assert ix.accounts[4].is_writable is True
    assert ix.accounts[5].pubkey == quote_mint
    assert ix.accounts[6].pubkey == fee_receiver
    assert ix.accounts[7].pubkey == get_conditional_token_ata(
        fee_receiver,
        quote_mint,
    )
    assert ix.accounts[9].pubkey == ASSOCIATED_TOKEN_PROGRAM_ID
    assert ix.data == bytes([34])


def test_set_manager_instruction_layout():
    authority = Pubkey.new_unique()
    new_manager = Pubkey.new_unique()

    ix = build_set_manager_instruction(authority, new_manager)

    assert len(ix.accounts) == 2
    assert ix.accounts[0].pubkey == authority
    assert ix.accounts[0].is_signer is True
    assert ix.data == bytes([28]) + bytes(new_manager)


def test_accept_role_instruction_layouts():
    incoming_role = Pubkey.new_unique()
    exchange, _ = get_exchange_pda()
    params = AcceptRoleParams(incoming_role=incoming_role)

    authority_ix = build_accept_authority_instruction(params)
    manager_ix = build_accept_manager_instruction(params)
    operator_ix = build_accept_operator_instruction(params)

    for ix in (authority_ix, manager_ix, operator_ix):
        assert len(ix.accounts) == 2
        assert ix.accounts[0].pubkey == incoming_role
        assert ix.accounts[0].is_signer is True
        assert ix.accounts[0].is_writable is False
        assert ix.accounts[1].pubkey == exchange
        assert ix.accounts[1].is_writable is True
        assert len(ix.data) == 1

    assert authority_ix.data == bytes([35])
    assert manager_ix.data == bytes([36])
    assert operator_ix.data == bytes([37])


def test_set_oracle_instruction_layout_and_zero_validation():
    authority = Pubkey.new_unique()
    market = Pubkey.new_unique()
    new_oracle = Pubkey.new_unique()
    exchange, _ = get_exchange_pda()

    ix = build_set_oracle_instruction(
        SetOracleParams(
            authority=authority,
            market=market,
            new_oracle=new_oracle,
        )
    )

    assert len(ix.accounts) == 3
    assert ix.accounts[0].pubkey == authority
    assert ix.accounts[0].is_signer is True
    assert ix.accounts[0].is_writable is False
    assert ix.accounts[1].pubkey == exchange
    assert ix.accounts[2].pubkey == market
    assert ix.accounts[2].is_writable is True
    assert ix.data == bytes([33]) + bytes(new_oracle)

    with pytest.raises(InvalidOracleError):
        build_set_oracle_instruction(
            SetOracleParams(
                authority=authority,
                market=market,
                new_oracle=Pubkey.from_bytes(bytes(32)),
            )
        )


def test_fee_admin_instruction_layouts():
    manager = Pubkey.new_unique()
    market = Pubkey.new_unique()

    fees_ix = build_set_market_fees_instruction(
        SetMarketFeesParams(
            manager=manager,
            updates=[
                MarketFeeUpdate(
                    market=market,
                    maker_fee_bps=-10,
                    taker_fee_bps=25,
                )
            ],
        )
    )

    assert len(fees_ix.accounts) == 3
    assert fees_ix.accounts[2].pubkey == market
    assert fees_ix.data[0] == 29
    assert int.from_bytes(fees_ix.data[1:3], "little", signed=True) == -10
    assert int.from_bytes(fees_ix.data[3:5], "little", signed=True) == 25

    authority = Pubkey.new_unique()
    fee_receiver = Pubkey.new_unique()
    receiver_ix = build_set_fee_receiver_instruction(
        SetFeeReceiverParams(authority=authority, new_fee_receiver=fee_receiver)
    )

    assert len(receiver_ix.accounts) == 2
    assert receiver_ix.data == bytes([30]) + bytes(fee_receiver)

    quote_mint_a = Pubkey.new_unique()
    quote_mint_b = Pubkey.new_unique()
    receiver_with_atas = build_set_fee_receiver_with_atas_instruction(
        SetFeeReceiverWithAtasParams(
            authority=authority,
            new_fee_receiver=fee_receiver,
            quote_mints=[quote_mint_a, quote_mint_b],
        )
    )

    assert len(receiver_with_atas.accounts) == 10
    assert receiver_with_atas.accounts[2].pubkey == fee_receiver
    assert receiver_with_atas.accounts[6].pubkey == quote_mint_a
    assert receiver_with_atas.accounts[7].pubkey == get_conditional_token_ata(
        fee_receiver,
        quote_mint_a,
    )
    assert receiver_with_atas.accounts[7].is_writable is True
    assert receiver_with_atas.accounts[8].pubkey == quote_mint_b
    assert receiver_with_atas.data == bytes([30]) + bytes(fee_receiver)

    with pytest.raises(MissingFieldError):
        build_set_fee_receiver_with_atas_instruction(
            SetFeeReceiverWithAtasParams(
                authority=authority,
                new_fee_receiver=fee_receiver,
                quote_mints=[],
            )
        )


def test_conditional_metadata_instruction_layouts():
    manager = Pubkey.new_unique()
    market = Pubkey.new_unique()
    deposit_mint = Pubkey.new_unique()
    params = ConditionalMetadataParams(
        manager=manager,
        market=market,
        deposit_mint=deposit_mint,
        outcome_index=1,
        name="Yes",
        symbol="YES",
        uri="https://example.com/yes.json",
    )
    conditional_mint, _ = get_conditional_mint_pda(market, deposit_mint, 1)
    metadata, _ = get_mpl_metadata_pda(conditional_mint)

    create_ix = build_create_conditional_metadata_instruction(params)
    assert len(create_ix.accounts) == 10
    assert create_ix.accounts[5].pubkey == metadata
    assert create_ix.data[0] == 31
    assert create_ix.data[1] == 1
    assert int.from_bytes(create_ix.data[2:6], "little") == 3

    update_ix = build_update_conditional_metadata_instruction(params)
    assert len(update_ix.accounts) == 8
    assert update_ix.accounts[0].is_writable is False
    assert update_ix.data[0] == 32


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


def test_increment_nonce_includes_exchange():
    user = Pubkey.new_unique()
    exchange, _ = get_exchange_pda()

    ix = build_increment_nonce_instruction(user)

    assert len(ix.accounts) == 4
    assert ix.accounts[0].pubkey == user
    assert ix.accounts[3].pubkey == exchange
    assert ix.accounts[3].is_writable is False
    assert ix.data == bytes([6])


def test_match_orders_multi_includes_orderbook_at_fixed_index():
    operator = Pubkey.new_unique()
    market = Pubkey.new_unique()
    base_mint = Pubkey.new_unique()
    quote_mint = Pubkey.new_unique()
    taker_order = signed_order(Pubkey.new_unique(), market, base_mint, quote_mint)
    maker_order = signed_order(
        Pubkey.new_unique(), market, base_mint, quote_mint, OrderSide.ASK, nonce=2
    )
    fee_receiver = Pubkey.new_unique()
    orderbook, _ = get_orderbook_pda(base_mint, quote_mint)

    ix = build_match_orders_multi_instruction(
        operator=operator,
        market=market,
        base_mint=base_mint,
        quote_mint=quote_mint,
        fee_receiver=fee_receiver,
        taker_order=taker_order,
        maker_orders=[maker_order],
        maker_fill_amounts=[100],
        taker_fill_amounts=[50],
    )

    assert ix.accounts[3].pubkey == orderbook
    assert ix.accounts[3].is_writable is False
    assert ix.accounts[13].pubkey == get_conditional_token_ata(
        fee_receiver,
        quote_mint,
    )
    assert ix.accounts[14].pubkey == fee_receiver
    assert ix.accounts[15].pubkey == ASSOCIATED_TOKEN_PROGRAM_ID


def test_deposit_and_swap_includes_orderbook_at_fixed_index():
    operator = Pubkey.new_unique()
    market = Pubkey.new_unique()
    base_mint = Pubkey.new_unique()
    quote_mint = Pubkey.new_unique()
    taker_order = signed_order(Pubkey.new_unique(), market, base_mint, quote_mint)
    maker_order = signed_order(
        Pubkey.new_unique(), market, base_mint, quote_mint, OrderSide.ASK, nonce=2
    )
    fee_receiver = Pubkey.new_unique()
    orderbook, _ = get_orderbook_pda(base_mint, quote_mint)

    ix = build_deposit_and_swap_instruction(
        operator=operator,
        market=market,
        base_mint=base_mint,
        quote_mint=quote_mint,
        fee_receiver=fee_receiver,
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
    assert ix.accounts[6].pubkey == get_conditional_token_ata(
        fee_receiver,
        quote_mint,
    )
    assert ix.accounts[7].pubkey == fee_receiver
    assert ix.accounts[8].pubkey == ASSOCIATED_TOKEN_PROGRAM_ID


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


def test_set_deposit_token_status_instruction_layout():
    manager = Pubkey.new_unique()
    mint = Pubkey.new_unique()
    exchange, _ = get_exchange_pda()
    global_deposit_token, _ = get_global_deposit_pda(mint)

    ix = build_set_deposit_token_status_instruction(
        SetDepositTokenStatusParams(
            manager=manager,
            mint=mint,
            active=False,
        )
    )

    assert len(ix.accounts) == 3
    assert ix.accounts[0].pubkey == manager
    assert ix.accounts[0].is_signer is True
    assert ix.accounts[0].is_writable is False
    assert ix.accounts[1].pubkey == exchange
    assert ix.accounts[2].pubkey == global_deposit_token
    assert ix.accounts[2].is_writable is True
    assert ix.data == bytes([38, 0])


def test_withdraw_from_global_includes_exchange():
    user = Pubkey.new_unique()
    mint = Pubkey.new_unique()
    exchange, _ = get_exchange_pda()

    ix = build_withdraw_from_global_instruction(user, mint, 1_000)

    assert len(ix.accounts) == 7
    assert ix.accounts[6].pubkey == exchange


def test_global_to_market_deposit_matches_canonical_account_layout():
    user = Pubkey.new_unique()
    market = Pubkey.new_unique()
    deposit_mint = Pubkey.new_unique()
    exchange, _ = get_exchange_pda()
    vault, _ = get_vault_pda(deposit_mint, market)
    global_deposit_token, _ = get_global_deposit_pda(deposit_mint)
    user_global_deposit, _ = get_user_global_deposit_pda(user, deposit_mint)
    position, _ = get_position_pda(user, market)
    mint_authority, _ = get_mint_authority_pda(market)

    ix = build_global_to_market_deposit_instruction(
        user=user,
        market=market,
        deposit_mint=deposit_mint,
        amount=1_000,
        num_outcomes=3,
    )

    assert len(ix.accounts) == 18
    assert [meta.pubkey for meta in ix.accounts[:9]] == [
        user,
        exchange,
        market,
        deposit_mint,
        vault,
        global_deposit_token,
        user_global_deposit,
        position,
        mint_authority,
    ]
    assert (
        ix.accounts[12].pubkey
        == get_conditional_mint_pda(
            market,
            deposit_mint,
            0,
        )[0]
    )
    assert len(ix.data) == 9
    assert ix.data[0] == 18


def test_withdraw_conditional_from_position_matches_canonical_account_layout():
    user = Pubkey.new_unique()
    market = Pubkey.new_unique()
    deposit_mint = Pubkey.new_unique()
    outcome_index = 1
    exchange, _ = get_exchange_pda()
    position, _ = get_position_pda(user, market)
    conditional_mint, _ = get_conditional_mint_pda(
        market, deposit_mint, outcome_index
    )
    position_conditional_ata = get_conditional_token_ata(position, conditional_mint)
    user_conditional_ata = get_conditional_token_ata(user, conditional_mint)

    ix = build_withdraw_conditional_from_position_instruction(
        user=user,
        market=market,
        deposit_mint=deposit_mint,
        amount=1_000,
        outcome_index=outcome_index,
    )

    assert len(ix.accounts) == 9
    assert [meta.pubkey for meta in ix.accounts] == [
        user,
        exchange,
        market,
        position,
        deposit_mint,
        conditional_mint,
        position_conditional_ata,
        user_conditional_ata,
        TOKEN_PROGRAM_ID,
    ]
    assert ix.accounts[0].is_signer is True
    assert ix.accounts[0].is_writable is True
    assert ix.accounts[3].is_writable is False
    assert ix.accounts[5].is_writable is False
    assert ix.accounts[6].is_writable is True
    assert ix.accounts[7].is_writable is True
    assert len(ix.data) == 10
    assert ix.data[0] == 11
    assert int.from_bytes(ix.data[1:9], "little") == 1_000
    assert ix.data[9] == outcome_index


def test_withdraw_conditional_from_position_rejects_out_of_range_outcome_index():
    user = Pubkey.new_unique()
    market = Pubkey.new_unique()
    deposit_mint = Pubkey.new_unique()

    for outcome_index in (-1, 256):
        with pytest.raises(InvalidOutcomeIndexError):
            build_withdraw_conditional_from_position_instruction(
                user=user,
                market=market,
                deposit_mint=deposit_mint,
                amount=1_000,
                outcome_index=outcome_index,
            )


def test_withdraw_from_position_wrapper_uses_conditional_contract():
    user = Pubkey.new_unique()
    market = Pubkey.new_unique()
    deposit_mint = Pubkey.new_unique()

    ix = build_withdraw_from_position_instruction(
        user=user,
        market=market,
        deposit_mint=deposit_mint,
        amount=1_000,
        outcome_index=1,
    )

    assert len(ix.accounts) == 9


def test_redeem_winnings_uses_outcome_index_and_exchange():
    user = Pubkey.new_unique()
    market = Pubkey.new_unique()
    deposit_mint = Pubkey.new_unique()
    outcome_index = 2
    exchange, _ = get_exchange_pda()
    vault, _ = get_vault_pda(deposit_mint, market)
    conditional_mint, _ = get_conditional_mint_pda(market, deposit_mint, outcome_index)
    position, _ = get_position_pda(user, market)
    position_conditional_ata = get_conditional_token_ata(position, conditional_mint)
    user_deposit_ata = get_associated_token_address(user, deposit_mint)
    mint_authority, _ = get_mint_authority_pda(market)

    ix = build_redeem_winnings_instruction(
        user=user,
        market=market,
        deposit_mint=deposit_mint,
        outcome_index=outcome_index,
        amount=123,
    )

    assert len(ix.accounts) == 11
    assert ix.accounts[3].pubkey == vault
    assert ix.accounts[4].pubkey == conditional_mint
    assert ix.accounts[5].pubkey == position
    assert ix.accounts[5].is_writable is False
    assert ix.accounts[6].pubkey == position_conditional_ata
    assert ix.accounts[7].pubkey == user_deposit_ata
    assert ix.accounts[8].pubkey == mint_authority
    assert ix.accounts[10].pubkey == exchange
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


def test_close_order_status_instruction_layout():
    operator = Pubkey.new_unique()
    order_hash = bytes([7] * 32)
    exchange, _ = get_exchange_pda()
    order_status, _ = get_order_status_pda(order_hash)

    ix = build_close_order_status_instruction(
        CloseOrderStatusParams(operator=operator, order_hash=order_hash)
    )

    assert len(ix.accounts) == 3
    assert [meta.pubkey for meta in ix.accounts] == [
        operator,
        exchange,
        order_status,
    ]
    assert ix.accounts[0].is_signer is True
    assert ix.accounts[2].is_writable is True
    assert ix.data == bytes([24]) + order_hash


def test_close_position_token_accounts_instruction_layout():
    operator = Pubkey.new_unique()
    market = Pubkey.new_unique()
    position = Pubkey.new_unique()
    deposit_mints = [Pubkey.new_unique(), Pubkey.new_unique()]
    exchange, _ = get_exchange_pda()

    ix = build_close_position_token_accounts_instruction(
        ClosePositionTokenAccountsParams(
            operator=operator,
            market=market,
            position=position,
            deposit_mints=deposit_mints,
        ),
        num_outcomes=2,
    )

    first_conditional_mint, _ = get_conditional_mint_pda(market, deposit_mints[0], 0)
    first_position_ata = get_conditional_token_ata(
        position,
        first_conditional_mint,
    )

    assert len(ix.accounts) == 15
    assert [meta.pubkey for meta in ix.accounts[:6]] == [
        operator,
        exchange,
        market,
        position,
        TOKEN_PROGRAM_ID,
        deposit_mints[0],
    ]
    assert ix.accounts[6].pubkey == first_conditional_mint
    assert ix.accounts[6].is_writable is False
    assert ix.accounts[7].pubkey == first_position_ata
    assert ix.accounts[7].is_writable is True
    assert ix.data == bytes([25])

    with pytest.raises(InvalidOutcomeCountError):
        build_close_position_token_accounts_instruction(
            ClosePositionTokenAccountsParams(
                operator=operator,
                market=market,
                position=position,
                deposit_mints=deposit_mints,
            ),
            num_outcomes=1,
        )


def test_close_alt_and_orderbook_instruction_layouts():
    operator = Pubkey.new_unique()
    market = Pubkey.new_unique()
    position = Pubkey.new_unique()
    orderbook = Pubkey.new_unique()
    lookup_table = Pubkey.new_unique()
    exchange, _ = get_exchange_pda()

    position_alt_ix = build_close_position_alt_instruction(
        ClosePositionAltParams(
            operator=operator,
            position=position,
            market=market,
            lookup_table=lookup_table,
        )
    )
    assert len(position_alt_ix.accounts) == 6
    assert [meta.pubkey for meta in position_alt_ix.accounts] == [
        operator,
        exchange,
        position,
        market,
        lookup_table,
        ALT_PROGRAM_ID,
    ]
    assert position_alt_ix.accounts[4].is_writable is True
    assert position_alt_ix.data == bytes([23])

    orderbook_alt_ix = build_close_orderbook_alt_instruction(
        CloseOrderbookAltParams(
            operator=operator,
            orderbook=orderbook,
            market=market,
            lookup_table=lookup_table,
        )
    )
    assert len(orderbook_alt_ix.accounts) == 6
    assert [meta.pubkey for meta in orderbook_alt_ix.accounts] == [
        operator,
        exchange,
        orderbook,
        market,
        lookup_table,
        ALT_PROGRAM_ID,
    ]
    assert orderbook_alt_ix.data == bytes([26])

    close_orderbook_ix = build_close_orderbook_instruction(
        CloseOrderbookParams(
            operator=operator,
            orderbook=orderbook,
            market=market,
            lookup_table=lookup_table,
        )
    )
    assert len(close_orderbook_ix.accounts) == 5
    assert [meta.pubkey for meta in close_orderbook_ix.accounts] == [
        operator,
        exchange,
        orderbook,
        market,
        lookup_table,
    ]
    assert close_orderbook_ix.accounts[2].is_writable is True
    assert close_orderbook_ix.accounts[4].is_writable is False
    assert close_orderbook_ix.data == bytes([27])
