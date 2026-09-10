"""Regression coverage for the program's matching and signer boundaries."""

import struct

import pytest
from solders.keypair import Keypair
from solders.pubkey import Pubkey

from lightcone_sdk.program import (
    INITIALIZE_AUTHORITY,
    MAX_MAKERS,
    SYSTEM_PROGRAM_ID,
    InvalidOracleError,
    InvalidPubkeyError,
    MakerFill,
    OrderSide,
    SetOracleParams,
    SignedOrder,
    TooManyMakersError,
    build_create_market_instruction,
    build_deposit_and_swap_instruction,
    build_init_position_tokens_instruction,
    build_match_orders_multi_instruction,
    build_set_oracle_instruction,
    build_set_paused_instruction,
    build_whitelist_deposit_token_instruction,
    get_conditional_mint_pda,
    get_event_authority_pda,
    get_exchange_pda,
    get_global_deposit_pda,
)


def wallet(seed: int) -> Pubkey:
    return Keypair.from_seed(bytes([seed]) * 32).pubkey()


def test_program_constants():
    assert MAX_MAKERS == 11
    assert str(INITIALIZE_AUTHORITY) == "3vYRAzr5X41hrmKMnDCoQJJmPH89S4LLwmFpk8UtwCqr"


@pytest.mark.parametrize("deposit", [False, True])
@pytest.mark.parametrize("full_fill_mask", [0x87FF, 0x8180])
def test_eleven_maker_records_masks_and_exact_fills(deposit, full_fill_mask):
    market, base_deposit_mint, quote_deposit_mint = wallet(3), wallet(70), wallet(71)
    base_mint = get_conditional_mint_pda(market, base_deposit_mint, 1)[0]
    quote_mint = get_conditional_mint_pda(market, quote_deposit_mint, 1)[0]
    maker_amount, taker_amount = 2**53 + 7, 2**63 + 11
    orders = [
        SignedOrder(
            nonce=index,
            salt=2**53 + index,
            maker=wallet(index + 10),
            market=market,
            base_mint=base_mint,
            quote_mint=quote_mint,
            side=OrderSide.BID if index == 0 else OrderSide.ASK,
            amount_in=taker_amount,
            amount_out=maker_amount,
            expiration=-1,
            signature=bytes([index]) * 64,
        )
        for index in range(13)
    ]
    common = {
        "operator": wallet(1),
        "market": market,
        "base_mint": base_mint,
        "quote_mint": quote_mint,
        "fee_receiver": wallet(2),
        "taker_order": orders[0],
        "base_deposit_mint": base_deposit_mint,
        "quote_deposit_mint": quote_deposit_mint,
    }

    def build(count):
        if deposit:
            return build_deposit_and_swap_instruction(
                **common,
                taker_is_full_fill=True,
                taker_is_deposit=True,
                taker_deposit_mint=quote_deposit_mint,
                num_outcomes=2,
                makers=[
                    MakerFill(
                        order=order,
                        maker_fill_amount=maker_amount,
                        taker_fill_amount=taker_amount,
                        is_full_fill=bool(full_fill_mask & (1 << index)),
                        is_deposit=index in (8, 10),
                        deposit_mint=base_deposit_mint,
                    )
                    for index, order in enumerate(orders[1 : count + 1])
                ],
            )
        return build_match_orders_multi_instruction(
            **common,
            maker_orders=orders[1 : count + 1],
            maker_fill_amounts=[maker_amount] * count,
            taker_fill_amounts=[taker_amount] * count,
            full_fill_bitmask=full_fill_mask,
        )

    ix = build(11)
    data = ix.data
    header_size = 107 if deposit else 105
    assert len(data) == header_size + 11 * 117
    assert data[0] == (20 if deposit else 13)
    assert data[102] == 11
    assert data[103:105] == full_fill_mask.to_bytes(2, "little")
    if deposit:
        assert data[105:107] == bytes([0x00, 0x85])  # taker and makers 8, 10
    for index, order in enumerate(orders[:12]):
        offset = 1 if index == 0 else header_size + (index - 1) * 117
        assert struct.unpack_from("<IQBQQq", data, offset) == (
            order.nonce,
            order.salt,
            int(order.side),
            taker_amount,
            maker_amount,
            -1,
        )
        assert data[offset + 37 : offset + 101] == order.signature
        if index:
            assert struct.unpack_from("<QQ", data, offset + 101) == (
                maker_amount,
                taker_amount,
            )
    collateral = (base_deposit_mint, quote_deposit_mint)
    if bytes(base_mint) > bytes(quote_mint):
        collateral = collateral[::-1]
    assert [(m.pubkey, m.is_signer, m.is_writable) for m in ix.accounts[4:6]] == [
        (get_global_deposit_pda(mint)[0], False, False) for mint in collateral
    ]
    with pytest.raises(TooManyMakersError, match="12"):
        build(12)


@pytest.mark.parametrize(
    "oracle", [Pubkey.default(), Pubkey.find_program_address([b"oracle"], wallet(9))[0]]
)
def test_market_creation_and_rotation_reject_invalid_oracles(oracle):
    with pytest.raises(InvalidOracleError):
        build_create_market_instruction(wallet(1), 0, 2, oracle, bytes(32), 0, 0)
    with pytest.raises(InvalidOracleError):
        build_set_oracle_instruction(SetOracleParams(wallet(1), wallet(2), oracle))


def test_position_setup_rejects_zero_and_pda_beneficiaries_without_restricting_governance():
    user = Pubkey.find_program_address([b"user"], wallet(9))[0]
    for beneficiary in (Pubkey.default(), user):
        with pytest.raises(InvalidPubkeyError, match=str(beneficiary)):
            build_init_position_tokens_instruction(
                wallet(1), beneficiary, wallet(2), [wallet(3)], 2
            )
    ix = build_set_paused_instruction(user, True)
    assert ix.accounts[0].pubkey == user
    assert ix.accounts[0].is_signer
    ix = build_set_oracle_instruction(SetOracleParams(user, wallet(2), wallet(1)))
    assert ix.data[1:] == bytes(wallet(1))


def test_whitelisting_can_increment_the_exchange_deposit_token_count():
    program, authority, mint = wallet(9), wallet(1), wallet(3)
    ix = build_whitelist_deposit_token_instruction(authority, mint, program)
    assert ix.data == bytes([16])
    assert [a.pubkey for a in ix.accounts] == [
        authority,
        get_exchange_pda(program)[0],
        mint,
        get_global_deposit_pda(mint, program)[0],
        SYSTEM_PROGRAM_ID,
        get_event_authority_pda(program)[0],
        program,
    ]
    assert [a.is_writable for a in ix.accounts] == [
        True,
        True,
        False,
        True,
        False,
        False,
        False,
    ]
    assert [a.is_signer for a in ix.accounts] == [
        True,
        False,
        False,
        False,
        False,
        False,
        False,
    ]
