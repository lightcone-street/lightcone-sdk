"""Regression coverage for the program's matching and signer boundaries."""

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
    build_extend_position_tokens_instruction,
    build_init_position_tokens_instruction,
    build_match_orders_multi_instruction,
    build_set_oracle_instruction,
    build_set_paused_instruction,
    build_whitelist_deposit_token_instruction,
    get_event_authority_pda,
    get_exchange_pda,
    get_global_deposit_pda,
)


def wallet(seed: int) -> Pubkey:
    return Keypair.from_seed(bytes([seed]) * 32).pubkey()


def test_program_constants():
    assert MAX_MAKERS == 4
    assert str(INITIALIZE_AUTHORITY) == "3vYRAzr5X41hrmKMnDCoQJJmPH89S4LLwmFpk8UtwCqr"


@pytest.mark.parametrize("deposit", [False, True])
def test_matching_four_maker_boundary_preserves_exact_fills(deposit):
    market, base, quote = wallet(3), wallet(4), wallet(5)
    orders = [
        SignedOrder(
            nonce=i,
            salt=i,
            maker=wallet(i + 10),
            market=market,
            base_mint=base,
            quote_mint=quote,
            side=OrderSide.BID if i == 0 else OrderSide.ASK,
            amount_in=2**63 + 11,
            amount_out=2**53 + 7,
            expiration=0,
            signature=bytes([i]) * 64,
        )
        for i in range(6)
    ]
    common = {
        "operator": wallet(1),
        "market": market,
        "base_mint": base,
        "quote_mint": quote,
        "fee_receiver": wallet(2),
        "taker_order": orders[0],
    }
    maker_amount, taker_amount = 2**53 + 7, 2**63 + 11

    def build(count):
        if deposit:
            return build_deposit_and_swap_instruction(
                **common,
                taker_is_full_fill=True,
                taker_deposit_mint=base,
                makers=[
                    MakerFill(
                        order=o,
                        maker_fill_amount=maker_amount,
                        taker_fill_amount=taker_amount,
                        is_full_fill=True,
                        is_deposit=False,
                        deposit_mint=base,
                    )
                    for o in orders[1 : count + 1]
                ],
            )
        return build_match_orders_multi_instruction(
            **common,
            maker_orders=orders[1 : count + 1],
            maker_fill_amounts=[maker_amount] * count,
            taker_fill_amounts=[taker_amount] * count,
            full_fill_bitmask=0x8F,
        )

    data = build(4).data
    assert data[0] == (20 if deposit else 13)
    assert data[102:104] == bytes([4, 0x8F])
    start = 105 if deposit else 104
    if deposit:
        assert data[104] == 0
    for i in range(4):
        offset = start + i * 117
        assert data[offset + 37 : offset + 101] == orders[i + 1].signature
        assert (
            int.from_bytes(data[offset + 101 : offset + 109], "little") == maker_amount
        )
        assert (
            int.from_bytes(data[offset + 109 : offset + 117], "little") == taker_amount
        )
    with pytest.raises(TooManyMakersError) as exc:
        build(5)
    assert exc.value.count == 5
    assert exc.value.max_count == 4


@pytest.mark.parametrize(
    "oracle", [Pubkey.default(), Pubkey.find_program_address([b"oracle"], wallet(9))[0]]
)
def test_market_creation_and_rotation_reject_invalid_oracles(oracle):
    with pytest.raises(InvalidOracleError):
        build_create_market_instruction(wallet(1), 0, 2, oracle, bytes(32), 0, 0)
    with pytest.raises(InvalidOracleError):
        build_set_oracle_instruction(SetOracleParams(wallet(1), wallet(2), oracle))


def test_position_setup_rejects_pda_beneficiaries_without_restricting_governance():
    user = Pubkey.find_program_address([b"user"], wallet(9))[0]
    with pytest.raises(InvalidPubkeyError):
        build_init_position_tokens_instruction(
            wallet(1), user, wallet(2), [wallet(3)], 2, 99
        )
    with pytest.raises(InvalidPubkeyError):
        build_extend_position_tokens_instruction(
            wallet(1), user, wallet(2), wallet(4), [wallet(3)], 2
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
