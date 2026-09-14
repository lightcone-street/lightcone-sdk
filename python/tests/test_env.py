import pytest

from lightcone_sdk.env import LightconeEnv


@pytest.mark.unit
@pytest.mark.parametrize(
    "environment,api_url,ws_url",
    [
        (
            LightconeEnv.LOCAL,
            "https://api.local.internalcone.com",
            "wss://ws.local.internalcone.com/ws",
        ),
        (
            LightconeEnv.STAGING,
            "https://api.staging.internalcone.com",
            "wss://ws.staging.internalcone.com/ws",
        ),
        (
            LightconeEnv.PROD,
            "https://api.lightcone.xyz",
            "wss://ws.lightcone.xyz/ws",
        ),
    ],
)
def test_environment_defaults_target_expected_backends(
    monkeypatch, environment, api_url, ws_url
):
    monkeypatch.delenv("SDK_API_URL", raising=False)
    monkeypatch.delenv("SDK_WS_URL", raising=False)

    assert environment.api_url == api_url
    assert environment.ws_url == ws_url
