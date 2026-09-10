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
    SetOracleParams,
    build_create_market_instruction,
    build_init_position_tokens_instruction,
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
    assert MAX_MAKERS == 11
    assert str(INITIALIZE_AUTHORITY) == "3vYRAzr5X41hrmKMnDCoQJJmPH89S4LLwmFpk8UtwCqr"


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
