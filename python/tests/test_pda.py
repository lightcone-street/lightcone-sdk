"""Tests for PDA derivation functions."""

import pytest
from solders.pubkey import Pubkey

from src import (
    PROGRAM_ID,
    get_all_conditional_mints,
    get_conditional_mint_pda,
    get_exchange_pda,
    get_market_pda,
    get_mint_authority_pda,
    get_order_status_pda,
    get_position_pda,
    get_user_nonce_pda,
    get_vault_pda,
)


class TestGetExchangePda:
    def test_derives_valid_pda(self):
        pda, bump = get_exchange_pda()

        # Should return a valid pubkey and bump
        assert isinstance(pda, Pubkey)
        assert isinstance(bump, int)
        assert 0 <= bump <= 255

    def test_consistent_derivation(self):
        # Same inputs should produce same outputs
        pda1, bump1 = get_exchange_pda()
        pda2, bump2 = get_exchange_pda()

        assert pda1 == pda2
        assert bump1 == bump2

    def test_with_custom_program_id(self):
        custom_program = Pubkey.new_unique()
        pda1, _ = get_exchange_pda(PROGRAM_ID)
        pda2, _ = get_exchange_pda(custom_program)

        # Different program IDs should produce different PDAs
        assert pda1 != pda2


class TestGetMarketPda:
    def test_derives_valid_pda(self):
        pda, bump = get_market_pda(0)

        assert isinstance(pda, Pubkey)
        assert isinstance(bump, int)
        assert 0 <= bump <= 255

    def test_different_market_ids_produce_different_pdas(self):
        pda0, _ = get_market_pda(0)
        pda1, _ = get_market_pda(1)
        pda100, _ = get_market_pda(100)

        assert pda0 != pda1
        assert pda1 != pda100
        assert pda0 != pda100

    def test_consistent_derivation(self):
        pda1, bump1 = get_market_pda(42)
        pda2, bump2 = get_market_pda(42)

        assert pda1 == pda2
        assert bump1 == bump2


class TestGetVaultPda:
    def test_derives_valid_pda(self):
        deposit_mint = Pubkey.new_unique()
        market = Pubkey.new_unique()

        pda, bump = get_vault_pda(deposit_mint, market)

        assert isinstance(pda, Pubkey)
        assert isinstance(bump, int)

    def test_different_inputs_produce_different_pdas(self):
        mint1 = Pubkey.new_unique()
        mint2 = Pubkey.new_unique()
        market = Pubkey.new_unique()

        pda1, _ = get_vault_pda(mint1, market)
        pda2, _ = get_vault_pda(mint2, market)

        assert pda1 != pda2


class TestGetMintAuthorityPda:
    def test_derives_valid_pda(self):
        market = Pubkey.new_unique()
        pda, bump = get_mint_authority_pda(market)

        assert isinstance(pda, Pubkey)
        assert isinstance(bump, int)


class TestGetConditionalMintPda:
    def test_derives_valid_pda(self):
        market = Pubkey.new_unique()
        deposit_mint = Pubkey.new_unique()

        pda, bump = get_conditional_mint_pda(market, deposit_mint, 0)

        assert isinstance(pda, Pubkey)
        assert isinstance(bump, int)

    def test_different_outcomes_produce_different_pdas(self):
        market = Pubkey.new_unique()
        deposit_mint = Pubkey.new_unique()

        pda0, _ = get_conditional_mint_pda(market, deposit_mint, 0)
        pda1, _ = get_conditional_mint_pda(market, deposit_mint, 1)

        assert pda0 != pda1


class TestGetOrderStatusPda:
    def test_derives_valid_pda(self):
        order_hash = bytes(32)
        pda, bump = get_order_status_pda(order_hash)

        assert isinstance(pda, Pubkey)
        assert isinstance(bump, int)

    def test_different_hashes_produce_different_pdas(self):
        hash1 = bytes([1] * 32)
        hash2 = bytes([2] * 32)

        pda1, _ = get_order_status_pda(hash1)
        pda2, _ = get_order_status_pda(hash2)

        assert pda1 != pda2


class TestGetUserNoncePda:
    def test_derives_valid_pda(self):
        user = Pubkey.new_unique()
        pda, bump = get_user_nonce_pda(user)

        assert isinstance(pda, Pubkey)
        assert isinstance(bump, int)

    def test_different_users_produce_different_pdas(self):
        user1 = Pubkey.new_unique()
        user2 = Pubkey.new_unique()

        pda1, _ = get_user_nonce_pda(user1)
        pda2, _ = get_user_nonce_pda(user2)

        assert pda1 != pda2


class TestGetPositionPda:
    def test_derives_valid_pda(self):
        owner = Pubkey.new_unique()
        market = Pubkey.new_unique()

        pda, bump = get_position_pda(owner, market)

        assert isinstance(pda, Pubkey)
        assert isinstance(bump, int)

    def test_different_owners_produce_different_pdas(self):
        owner1 = Pubkey.new_unique()
        owner2 = Pubkey.new_unique()
        market = Pubkey.new_unique()

        pda1, _ = get_position_pda(owner1, market)
        pda2, _ = get_position_pda(owner2, market)

        assert pda1 != pda2

    def test_different_markets_produce_different_pdas(self):
        owner = Pubkey.new_unique()
        market1 = Pubkey.new_unique()
        market2 = Pubkey.new_unique()

        pda1, _ = get_position_pda(owner, market1)
        pda2, _ = get_position_pda(owner, market2)

        assert pda1 != pda2


class TestGetAllConditionalMints:
    def test_returns_correct_number_of_mints(self):
        market = Pubkey.new_unique()
        deposit_mint = Pubkey.new_unique()

        mints = get_all_conditional_mints(market, deposit_mint, 2)
        assert len(mints) == 2

        mints = get_all_conditional_mints(market, deposit_mint, 6)
        assert len(mints) == 6

    def test_mints_are_unique(self):
        market = Pubkey.new_unique()
        deposit_mint = Pubkey.new_unique()

        mints = get_all_conditional_mints(market, deposit_mint, 4)

        # All mints should be different
        assert len(set(str(m) for m in mints)) == 4

    def test_matches_individual_derivation(self):
        market = Pubkey.new_unique()
        deposit_mint = Pubkey.new_unique()

        mints = get_all_conditional_mints(market, deposit_mint, 3)

        for i, mint in enumerate(mints):
            expected, _ = get_conditional_mint_pda(market, deposit_mint, i)
            assert mint == expected
