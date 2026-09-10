"""Tests for on-chain instruction account layouts."""

import pytest
from solders.instruction import Instruction
from solders.keypair import Keypair
from solders.pubkey import Pubkey

from lightcone_sdk.program import (
    ASSOCIATED_TOKEN_PROGRAM_ID,
    MAX_DEPOSIT_MINTS_PER_IX,
    PROGRAM_ID,
    SYSTEM_PROGRAM_ID,
    TOKEN_PROGRAM_ID,
    AcceptRoleParams,
    ArithmeticOverflowError,
    CloseOrderbookParams,
    CloseOrderStatusParams,
    ClosePositionTokenAccountsParams,
    ConditionalMetadataParams,
    InvalidOracleError,
    InvalidOutcomeCountError,
    InvalidOutcomeIndexError,
    InvalidPayoutNumeratorsError,
    MakerFill,
    MarketFeeUpdate,
    MissingFieldError,
    OrderSide,
    SetDepositTokenStatusParams,
    SetFeeReceiverParams,
    SetFeeReceiverWithAtasParams,
    SetMarketFeesParams,
    SetOracleParams,
    SignedOrder,
    TooManyDepositMintsError,
    build_accept_authority_instruction,
    build_accept_manager_instruction,
    build_accept_operator_instruction,
    build_activate_market_instruction,
    build_add_deposit_mint_instruction,
    build_cancel_order_instruction,
    build_close_order_status_instruction,
    build_close_orderbook_instruction,
    build_close_position_token_accounts_instruction,
    build_create_conditional_metadata_instruction,
    build_create_market_instruction,
    build_create_orderbook_instruction,
    build_deposit_and_swap_instruction,
    build_deposit_to_global_instruction,
    build_global_to_market_deposit_instruction,
    build_increment_nonce_instruction,
    build_init_position_tokens_instruction,
    build_initialize_instruction,
    build_match_orders_multi_instruction,
    build_merge_complete_set_instruction,
    build_mint_complete_set_instruction,
    build_redeem_winnings_instruction,
    build_set_authority_instruction,
    build_set_deposit_token_status_instruction,
    build_set_fee_receiver_instruction,
    build_set_fee_receiver_with_atas_instruction,
    build_set_manager_instruction,
    build_set_market_fees_instruction,
    build_set_operator_instruction,
    build_set_oracle_instruction,
    build_set_paused_instruction,
    build_settle_market_instruction,
    build_update_conditional_metadata_instruction,
    build_whitelist_deposit_token_instruction,
    build_withdraw_conditional_from_position_instruction,
    build_withdraw_from_global_instruction,
    build_withdraw_from_position_instruction,
    derive_condition_id,
    get_associated_token_address,
    get_condition_tombstone_pda,
    get_conditional_mint_pda,
    get_conditional_token_ata,
    get_event_authority_pda,
    get_exchange_pda,
    get_global_deposit_pda,
    get_market_pda,
    get_mint_authority_pda,
    get_mpl_metadata_pda,
    get_order_status_pda,
    get_orderbook_pda,
    get_position_pda,
    get_user_global_deposit_pda,
    get_vault_pda,
    hash_order,
)


def fixed_pubkey(value: int) -> Pubkey:
    return Pubkey.from_bytes(bytes([value] * 32))


def event_transport_trailer(program_id: Pubkey = PROGRAM_ID) -> list[Pubkey]:
    """The two read-only accounts every public instruction must end with."""
    event_authority, _ = get_event_authority_pda(program_id)
    return [event_authority, program_id]


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


@pytest.mark.parametrize("reverse", [False, True])
@pytest.mark.parametrize("base_index", [0, 1])
def test_create_orderbook_encodes_canonical_collateral_and_base_orientation(
    reverse, base_index
):
    manager, market, fee_receiver, program = map(fixed_pubkey, [31, 32, 33, 34])
    outcome = 5
    canonical = sorted(
        [
            (get_conditional_mint_pda(market, mint, outcome, program)[0], mint)
            for mint in (fixed_pubkey(41), fixed_pubkey(42))
        ],
        key=lambda pair: bytes(pair[0]),
    )
    supplied = canonical[::-1] if reverse else canonical
    ix = build_create_orderbook_instruction(
        manager,
        market,
        supplied[0][0],
        supplied[1][0],
        fee_receiver,
        supplied[0][1],
        supplied[1][1],
        base_index,
        outcome,
        program,
    )
    mint_a, deposit_a = canonical[0]
    mint_b, deposit_b = canonical[1]
    canonical_base_index = base_index ^ int(reverse)
    quote_mint = supplied[1 - base_index][0]

    assert ix.program_id == program
    assert ix.data == bytes([15, canonical_base_index, outcome])
    assert [
        (meta.pubkey, meta.is_signer, meta.is_writable) for meta in ix.accounts
    ] == [
        (manager, True, True),
        (market, False, False),
        (mint_a, False, False),
        (mint_b, False, False),
        (get_orderbook_pda(mint_a, mint_b, program)[0], False, True),
        (get_global_deposit_pda(deposit_a, program)[0], False, False),
        (get_global_deposit_pda(deposit_b, program)[0], False, False),
        (get_exchange_pda(program)[0], False, False),
        (SYSTEM_PROGRAM_ID, False, False),
        (deposit_a, False, False),
        (deposit_b, False, False),
        (TOKEN_PROGRAM_ID, False, False),
        (ASSOCIATED_TOKEN_PROGRAM_ID, False, False),
        (fee_receiver, False, False),
        (get_conditional_token_ata(fee_receiver, quote_mint), False, True),
        (get_event_authority_pda(program)[0], False, False),
        (program, False, False),
    ]


def test_position_preparation_rejects_empty_mints():
    user = Keypair.from_seed(bytes([1]) * 32).pubkey()
    with pytest.raises(MissingFieldError, match="deposit_mints"):
        build_init_position_tokens_instruction(user, user, fixed_pubkey(2), [], 2)


def test_position_preparation_rejects_too_many_mint_groups():
    user = Keypair.from_seed(bytes([1]) * 32).pubkey()
    mints = [fixed_pubkey(index + 10) for index in range(MAX_DEPOSIT_MINTS_PER_IX + 1)]
    with pytest.raises(TooManyDepositMintsError):
        build_init_position_tokens_instruction(user, user, fixed_pubkey(2), mints, 2)


def test_create_market_uses_manager_and_condition_tombstone():
    manager = Pubkey.new_unique()
    oracle = Keypair().pubkey()
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

    assert len(ix.accounts) == 7
    assert ix.accounts[0].pubkey == manager
    assert ix.accounts[0].is_signer is True
    assert ix.accounts[4].pubkey == condition_tombstone
    assert ix.accounts[4].is_writable is True
    assert len(ix.data) == 70
    assert int.from_bytes(ix.data[66:68], "little", signed=True) == 10
    assert int.from_bytes(ix.data[68:70], "little", signed=True) == 20


def test_add_deposit_mint_writes_market_and_reads_global_deposit_token():
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

    assert len(ix.accounts) == 13
    assert ix.accounts[0].pubkey == manager
    assert ix.accounts[2].pubkey == market
    assert ix.accounts[2].is_writable is True
    assert ix.accounts[8].pubkey == global_deposit_token
    assert ix.accounts[8].is_writable is False
    assert ix.data == bytes([2])


def test_mint_complete_set_matches_canonical_account_layout():
    user = Keypair().pubkey()
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

    assert len(ix.accounts) == 19
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


def test_set_manager_instruction_layout():
    authority = Pubkey.new_unique()
    new_manager = Pubkey.new_unique()

    ix = build_set_manager_instruction(authority, new_manager)

    assert len(ix.accounts) == 4
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
        assert len(ix.accounts) == 4
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
    new_oracle = Keypair().pubkey()
    exchange, _ = get_exchange_pda()

    ix = build_set_oracle_instruction(
        SetOracleParams(
            authority=authority,
            market=market,
            new_oracle=new_oracle,
        )
    )

    assert len(ix.accounts) == 5
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

    assert len(fees_ix.accounts) == 5
    assert fees_ix.accounts[2].pubkey == market
    assert fees_ix.data[0] == 29
    assert int.from_bytes(fees_ix.data[1:3], "little", signed=True) == -10
    assert int.from_bytes(fees_ix.data[3:5], "little", signed=True) == 25

    authority = Pubkey.new_unique()
    fee_receiver = Pubkey.new_unique()
    receiver_ix = build_set_fee_receiver_instruction(
        SetFeeReceiverParams(authority=authority, new_fee_receiver=fee_receiver)
    )

    assert len(receiver_ix.accounts) == 4
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

    assert len(receiver_with_atas.accounts) == 12
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
    assert len(create_ix.accounts) == 12
    assert create_ix.accounts[5].pubkey == metadata
    assert create_ix.data[0] == 31
    assert create_ix.data[1] == 1
    assert int.from_bytes(create_ix.data[2:6], "little") == 3

    update_ix = build_update_conditional_metadata_instruction(params)
    assert len(update_ix.accounts) == 10
    assert update_ix.accounts[0].is_writable is False
    assert update_ix.data[0] == 32


def test_settle_market_uses_payout_vector_layout():
    oracle = Keypair().pubkey()
    market_id = 7
    exchange, _ = get_exchange_pda()
    market, _ = get_market_pda(market_id)

    ix = build_settle_market_instruction(
        oracle=oracle,
        market_id=market_id,
        payout_numerators=[7, 3],
    )

    assert len(ix.accounts) == 5
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
    oracle = Keypair().pubkey()

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
        *event_transport_trailer(),
    ]
    assert ix.accounts[0].is_signer is True


def test_increment_nonce_includes_exchange():
    user = Keypair().pubkey()
    exchange, _ = get_exchange_pda()

    ix = build_increment_nonce_instruction(user)

    assert len(ix.accounts) == 6
    assert ix.accounts[0].pubkey == user
    assert ix.accounts[3].pubkey == exchange
    assert ix.accounts[3].is_writable is False
    assert ix.data == bytes([6])


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

    assert len(ix.accounts) == 5
    assert ix.accounts[0].pubkey == manager
    assert ix.accounts[0].is_signer is True
    assert ix.accounts[0].is_writable is False
    assert ix.accounts[1].pubkey == exchange
    assert ix.accounts[2].pubkey == global_deposit_token
    assert ix.accounts[2].is_writable is True
    assert ix.data == bytes([38, 0])


def test_withdraw_from_global_includes_exchange():
    user = Keypair().pubkey()
    mint = Pubkey.new_unique()
    exchange, _ = get_exchange_pda()

    ix = build_withdraw_from_global_instruction(user, mint, 1_000)

    assert len(ix.accounts) == 9
    assert ix.accounts[6].pubkey == exchange


def test_global_to_market_deposit_matches_canonical_account_layout():
    user = Keypair().pubkey()
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

    assert len(ix.accounts) == 20
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
    user = Keypair().pubkey()
    market = Pubkey.new_unique()
    deposit_mint = Pubkey.new_unique()
    outcome_index = 1
    exchange, _ = get_exchange_pda()
    position, _ = get_position_pda(user, market)
    conditional_mint, _ = get_conditional_mint_pda(market, deposit_mint, outcome_index)
    position_conditional_ata = get_conditional_token_ata(position, conditional_mint)
    user_conditional_ata = get_conditional_token_ata(user, conditional_mint)

    ix = build_withdraw_conditional_from_position_instruction(
        user=user,
        market=market,
        deposit_mint=deposit_mint,
        amount=1_000,
        outcome_index=outcome_index,
    )

    assert len(ix.accounts) == 11
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
        *event_transport_trailer(),
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
    user = Keypair().pubkey()
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
    user = Keypair().pubkey()
    market = Pubkey.new_unique()
    deposit_mint = Pubkey.new_unique()

    ix = build_withdraw_from_position_instruction(
        user=user,
        market=market,
        deposit_mint=deposit_mint,
        amount=1_000,
        outcome_index=1,
    )

    assert len(ix.accounts) == 11


def test_redeem_winnings_uses_outcome_index_and_exchange():
    user = Keypair().pubkey()
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

    assert len(ix.accounts) == 13
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


def test_close_order_status_instruction_layout():
    operator = Pubkey.new_unique()
    order_hash = bytes([7] * 32)
    exchange, _ = get_exchange_pda()
    order_status, _ = get_order_status_pda(order_hash)

    ix = build_close_order_status_instruction(
        CloseOrderStatusParams(operator=operator, order_hash=order_hash)
    )

    assert len(ix.accounts) == 5
    assert [meta.pubkey for meta in ix.accounts] == [
        operator,
        exchange,
        order_status,
        *event_transport_trailer(),
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

    assert len(ix.accounts) == 17
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


def all_public_builders() -> list[tuple[str, Instruction]]:
    """One valid instance of every public instruction builder.

    Register new builders here so the trailer test covers them.
    """
    signer = Pubkey.new_unique()
    market = Pubkey.new_unique()
    deposit_mint = Pubkey.new_unique()
    base_deposit_mint = Pubkey.new_unique()
    base_mint = get_conditional_mint_pda(market, base_deposit_mint, 0)[0]
    quote_mint = get_conditional_mint_pda(market, deposit_mint, 0)[0]
    fee_receiver = Pubkey.new_unique()
    taker_order = signed_order(Pubkey.new_unique(), market, base_mint, quote_mint)
    maker_order = signed_order(
        Pubkey.new_unique(), market, base_mint, quote_mint, OrderSide.ASK, nonce=2
    )
    accept_role = AcceptRoleParams(incoming_role=signer)
    metadata = ConditionalMetadataParams(
        manager=signer,
        market=market,
        deposit_mint=deposit_mint,
        outcome_index=0,
        name="Yes",
        symbol="YES",
        uri="https://example.com/yes.json",
    )

    return [
        ("initialize", build_initialize_instruction(signer)),
        (
            "create_market",
            build_create_market_instruction(
                signer, 0, 2, Keypair().pubkey(), bytes([1] * 32), 0, 0
            ),
        ),
        (
            "add_deposit_mint",
            build_add_deposit_mint_instruction(signer, market, deposit_mint, 2),
        ),
        (
            "mint_complete_set",
            build_mint_complete_set_instruction(signer, market, deposit_mint, 1, 2),
        ),
        (
            "merge_complete_set",
            build_merge_complete_set_instruction(signer, market, deposit_mint, 1, 2),
        ),
        ("cancel_order", build_cancel_order_instruction(signer, market, taker_order)),
        ("increment_nonce", build_increment_nonce_instruction(signer)),
        ("settle_market", build_settle_market_instruction(signer, 0, [1, 0])),
        (
            "redeem_winnings",
            build_redeem_winnings_instruction(signer, market, deposit_mint, 0, 1),
        ),
        ("set_paused", build_set_paused_instruction(signer, True)),
        ("set_operator", build_set_operator_instruction(signer, Pubkey.new_unique())),
        (
            "withdraw_conditional_from_position",
            build_withdraw_conditional_from_position_instruction(
                signer, market, deposit_mint, 1, 0
            ),
        ),
        (
            "withdraw_from_position",
            build_withdraw_from_position_instruction(
                signer, market, deposit_mint, 1, 0
            ),
        ),
        ("activate_market", build_activate_market_instruction(signer, 0)),
        (
            "match_orders_multi",
            build_match_orders_multi_instruction(
                signer,
                market,
                base_mint,
                quote_mint,
                fee_receiver,
                taker_order,
                [maker_order],
                [50],
                [100],
                base_deposit_mint=base_deposit_mint,
                quote_deposit_mint=deposit_mint,
            ),
        ),
        (
            "create_orderbook",
            build_create_orderbook_instruction(
                signer,
                market,
                base_mint,
                quote_mint,
                fee_receiver,
                base_deposit_mint,
                deposit_mint,
                0,
                0,
            ),
        ),
        (
            "set_authority",
            build_set_authority_instruction(signer, Pubkey.new_unique()),
        ),
        ("set_manager", build_set_manager_instruction(signer, Pubkey.new_unique())),
        ("accept_authority", build_accept_authority_instruction(accept_role)),
        ("accept_manager", build_accept_manager_instruction(accept_role)),
        ("accept_operator", build_accept_operator_instruction(accept_role)),
        (
            "set_oracle",
            build_set_oracle_instruction(
                SetOracleParams(
                    authority=signer, market=market, new_oracle=Keypair().pubkey()
                )
            ),
        ),
        (
            "set_market_fees",
            build_set_market_fees_instruction(
                SetMarketFeesParams(
                    manager=signer,
                    updates=[
                        MarketFeeUpdate(market=market, maker_fee_bps=0, taker_fee_bps=0)
                    ],
                )
            ),
        ),
        (
            "set_fee_receiver",
            build_set_fee_receiver_instruction(
                SetFeeReceiverParams(authority=signer, new_fee_receiver=fee_receiver)
            ),
        ),
        (
            "set_fee_receiver_with_atas",
            build_set_fee_receiver_with_atas_instruction(
                SetFeeReceiverWithAtasParams(
                    authority=signer,
                    new_fee_receiver=fee_receiver,
                    quote_mints=[quote_mint],
                )
            ),
        ),
        (
            "create_conditional_metadata",
            build_create_conditional_metadata_instruction(metadata),
        ),
        (
            "update_conditional_metadata",
            build_update_conditional_metadata_instruction(metadata),
        ),
        (
            "whitelist_deposit_token",
            build_whitelist_deposit_token_instruction(signer, deposit_mint),
        ),
        (
            "set_deposit_token_status",
            build_set_deposit_token_status_instruction(
                SetDepositTokenStatusParams(
                    manager=signer, mint=deposit_mint, active=True
                )
            ),
        ),
        (
            "deposit_to_global",
            build_deposit_to_global_instruction(signer, deposit_mint, 1),
        ),
        (
            "global_to_market_deposit",
            build_global_to_market_deposit_instruction(
                signer, market, deposit_mint, 1, 2
            ),
        ),
        (
            "init_position_tokens",
            build_init_position_tokens_instruction(
                signer, Keypair().pubkey(), market, [deposit_mint], 2
            ),
        ),
        (
            "deposit_and_swap",
            build_deposit_and_swap_instruction(
                signer,
                market,
                base_mint,
                quote_mint,
                fee_receiver,
                taker_order,
                base_deposit_mint=base_deposit_mint,
                quote_deposit_mint=deposit_mint,
                taker_is_full_fill=True,
                taker_is_deposit=True,
                taker_deposit_mint=deposit_mint,
                num_outcomes=2,
                makers=[
                    MakerFill(
                        order=maker_order,
                        maker_fill_amount=50,
                        taker_fill_amount=100,
                        deposit_mint=deposit_mint,
                        is_full_fill=True,
                    )
                ],
            ),
        ),
        (
            "withdraw_from_global",
            build_withdraw_from_global_instruction(signer, deposit_mint, 1),
        ),
        (
            "close_order_status",
            build_close_order_status_instruction(
                CloseOrderStatusParams(operator=signer, order_hash=bytes([2] * 32))
            ),
        ),
        (
            "close_position_token_accounts",
            build_close_position_token_accounts_instruction(
                ClosePositionTokenAccountsParams(
                    operator=signer,
                    market=market,
                    position=Pubkey.new_unique(),
                    deposit_mints=[deposit_mint],
                ),
                num_outcomes=2,
            ),
        ),
        (
            "close_orderbook",
            build_close_orderbook_instruction(
                CloseOrderbookParams(
                    operator=signer,
                    orderbook=Pubkey.new_unique(),
                    market=market,
                )
            ),
        ),
    ]


def test_every_public_builder_ends_with_event_transport_trailer():
    builders = all_public_builders()
    assert len(builders) == 37, "register new builders in all_public_builders"
    expected = event_transport_trailer()

    for name, ix in builders:
        assert ix.program_id == PROGRAM_ID, name
        body, trailer = ix.accounts[:-2], ix.accounts[-2:]
        assert [meta.pubkey for meta in trailer] == expected, name
        assert all(
            not meta.is_signer and not meta.is_writable for meta in trailer
        ), name
        assert all(meta.pubkey != expected[0] for meta in body), name


def test_event_transport_trailer_follows_custom_program_id():
    program_id = Pubkey.new_unique()
    event_authority, _ = get_event_authority_pda(program_id)

    ix = build_increment_nonce_instruction(Pubkey.new_unique(), program_id)

    assert ix.program_id == program_id
    assert [meta.pubkey for meta in ix.accounts[-2:]] == [event_authority, program_id]


def test_set_fee_receiver_with_atas_keeps_trailer_after_optional_block():
    fee_receiver = Pubkey.new_unique()
    quote_mint = Pubkey.new_unique()

    ix = build_set_fee_receiver_with_atas_instruction(
        SetFeeReceiverWithAtasParams(
            authority=Pubkey.new_unique(),
            new_fee_receiver=fee_receiver,
            quote_mints=[quote_mint],
        )
    )

    assert len(ix.accounts) == 10
    assert ix.accounts[7].pubkey == get_conditional_token_ata(fee_receiver, quote_mint)
    assert [meta.pubkey for meta in ix.accounts[8:]] == event_transport_trailer()
