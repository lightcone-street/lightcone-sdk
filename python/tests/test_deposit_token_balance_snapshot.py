import pytest

from lightcone_sdk.domain.position import DepositTokenBalancesSnapshot
from lightcone_sdk.domain.position.client import Positions


def test_snapshot_parses_context_slot_and_nested_balances() -> None:
    snapshot = DepositTokenBalancesSnapshot.from_dict(
        {
            "context_slot": 1234,
            "balances": {
                "MintA": {
                    "mint": "MintA",
                    "idle": "1.25",
                    "symbol": "USDC",
                    "name": "USD Coin",
                }
            },
        }
    )

    assert snapshot.context_slot == 1234
    assert snapshot.balances["MintA"].idle == "1.25"


def test_snapshot_rejects_malformed_balance_entries() -> None:
    with pytest.raises(
        TypeError, match="deposit-token snapshot balance entries must be objects"
    ):
        DepositTokenBalancesSnapshot.from_dict(
            {
                "context_slot": 1234,
                "balances": {"MintA": None},
            }
        )


class FakeHttp:
    def __init__(self) -> None:
        self.requests: list[tuple[str, dict[str, str] | None, str | None]] = []

    async def get(
        self, path: str, *, params: dict[str, str] | None = None
    ) -> dict[str, object]:
        self.requests.append((path, params, None))
        return {"context_slot": 1234, "balances": {}}

    async def get_with_cookies(
        self,
        path: str,
        *,
        cookie_header: str,
        params: dict[str, str] | None = None,
    ) -> dict[str, object]:
        self.requests.append((path, params, cookie_header))
        return {"context_slot": 1234, "balances": {}}


class FakeClient:
    def __init__(self, http: FakeHttp) -> None:
        self._http = http


@pytest.mark.asyncio
async def test_deposit_balance_methods_forward_minimum_slot_and_cookie() -> None:
    http = FakeHttp()
    positions = Positions(FakeClient(http))  # type: ignore[arg-type]

    snapshot = await positions.deposit_token_balances(1234)
    cookie_snapshot = await positions.deposit_token_balances_with_cookies(
        None, "lightcone-token=test"
    )

    assert snapshot.context_slot == 1234
    assert cookie_snapshot.context_slot == 1234
    assert http.requests == [
        (
            "/api/users/deposit-token-balances",
            {"min_context_slot": "1234"},
            None,
        ),
        (
            "/api/users/deposit-token-balances",
            None,
            "lightcone-token=test",
        ),
    ]
