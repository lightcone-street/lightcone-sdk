"""Direct jurisdiction response parity with the Rust SDK."""

from types import SimpleNamespace

import pytest

from lightcone_sdk.domain.jurisdiction import Jurisdiction, JurisdictionResponse


def response() -> dict:
    capabilities = {
        "mode": "full",
        "can_authenticate": True,
        "can_mutate_account": True,
        "can_submit_orders": True,
        "can_cancel_orders": True,
    }
    return {
        "country": "US",
        "geoblocked": False,
        "tier": "tier_3",
        "policy_version": "test",
        "frontend": capabilities,
        "api": capabilities,
        "relayed": False,
    }


@pytest.mark.asyncio
async def test_direct_geoblock_uses_api_transport_and_decodes_capabilities() -> None:
    class Http:
        async def get(self, path: str) -> dict:
            assert path == "/api/geoblock"
            return response()

    result = await Jurisdiction(SimpleNamespace(_http=Http())).geoblock()
    assert isinstance(result, JurisdictionResponse)
    assert result.country == "US"
    assert result.frontend.can_submit_orders
    assert result.api.can_cancel_orders
    assert result.relayed is False


def test_missing_or_wrong_type_relay_stamp_is_rejected() -> None:
    missing = response()
    del missing["relayed"]
    with pytest.raises(KeyError, match="relayed"):
        JurisdictionResponse.from_dict(missing)
    malformed = response()
    malformed["relayed"] = "false"
    with pytest.raises(ValueError, match="relayed"):
        JurisdictionResponse.from_dict(malformed)
