from solders.pubkey import Pubkey

import pytest

from lightcone_sdk.client import LightconeClient
from lightcone_sdk.error import SdkError
from lightcone_sdk.http import LightconeHttp
from lightcone_sdk.program.errors import InvalidOutcomeIndexError


def builder(client: LightconeClient):
    return (
        client.positions()
        .withdraw_from_position()
        .user(Pubkey.new_unique())
        .market(Pubkey.new_unique())
        .deposit_mint(Pubkey.new_unique())
        .amount(1)
        .outcome_index(2)
    )


def test_withdraw_from_position_requires_num_outcomes() -> None:
    client = LightconeClient(LightconeHttp("https://example.com"))

    with pytest.raises(SdkError, match="num_outcomes is required"):
        builder(client).build_ix()


def test_withdraw_from_position_validates_against_num_outcomes() -> None:
    client = LightconeClient(LightconeHttp("https://example.com"))

    with pytest.raises(InvalidOutcomeIndexError):
        builder(client).num_outcomes(2).build_ix()
