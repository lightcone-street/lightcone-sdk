"""Safety-gate tests for the canonical WSOL conversion example."""

import runpy
from collections.abc import Awaitable, Callable
from pathlib import Path
from typing import cast

import pytest
from lightcone_sdk import SolBalanceComponents, SolComponentDelta


def _example_namespace(monkeypatch: pytest.MonkeyPatch) -> dict[str, object]:
    """Load the guarded example without running its fund-moving entrypoint."""
    examples = Path(__file__).parents[1] / "examples"
    monkeypatch.syspath_prepend(str(examples))
    return runpy.run_path(str(examples / "wsol_conversion.py"))


def _safety_gate(monkeypatch: pytest.MonkeyPatch) -> Callable[[], None]:
    """Return the example's pre-side-effect environment guard."""
    return cast(
        Callable[[], None],
        _example_namespace(monkeypatch)["require_non_production"],
    )


def test_wsol_conversion_example_refuses_default_production(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Fail before client construction when no explicit safe environment exists."""
    monkeypatch.delenv("LIGHTCONE_ENV", raising=False)
    with pytest.raises(RuntimeError, match="disabled in production"):
        _safety_gate(monkeypatch)()

    monkeypatch.setenv("CI", "true")
    monkeypatch.setenv("LIGHTCONE_ENV", "prod")
    monkeypatch.setenv("SDK_RPC_URL", "ci-rpc")
    with pytest.raises(RuntimeError, match="disabled in production"):
        _safety_gate(monkeypatch)()


@pytest.mark.parametrize(
    "override",
    ["SDK_API_URL", "SDK_WS_URL", "SDK_RPC_URL", "SDK_PROGRAM_ID"],
)
def test_wsol_conversion_example_refuses_endpoint_overrides(
    monkeypatch: pytest.MonkeyPatch, override: str
) -> None:
    """Do not trust a non-production label when an origin can be repointed."""
    monkeypatch.setenv("LIGHTCONE_ENV", "staging")
    for name in ("SDK_API_URL", "SDK_WS_URL", "SDK_RPC_URL", "SDK_PROGRAM_ID"):
        monkeypatch.delenv(name, raising=False)
    monkeypatch.setenv(override, "unsafe-override")
    with pytest.raises(RuntimeError, match=f"unset {override}"):
        _safety_gate(monkeypatch)()


def test_wsol_conversion_example_accepts_builtin_non_production(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Permit an explicit built-in local environment without starting the flow."""
    monkeypatch.setenv("LIGHTCONE_ENV", "local")
    for name in ("SDK_API_URL", "SDK_WS_URL", "SDK_RPC_URL", "SDK_PROGRAM_ID"):
        monkeypatch.delenv(name, raising=False)
    _safety_gate(monkeypatch)()


def test_wsol_conversion_example_accepts_paid_local_rpc_only(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Allow local rate-limit avoidance without redirecting app or program endpoints."""
    monkeypatch.setenv("LIGHTCONE_ENV", "local")
    for name in ("SDK_API_URL", "SDK_WS_URL", "SDK_PROGRAM_ID"):
        monkeypatch.delenv(name, raising=False)
    monkeypatch.setenv("SDK_RPC_URL", "https://example.invalid")
    _safety_gate(monkeypatch)()

    for override in ("SDK_API_URL", "SDK_WS_URL", "SDK_PROGRAM_ID"):
        monkeypatch.setenv(override, "unsafe-override")
        with pytest.raises(RuntimeError, match=f"unset {override}"):
            _safety_gate(monkeypatch)()
        monkeypatch.delenv(override)


def test_wsol_conversion_example_accepts_ci_endpoints_but_not_program_override(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Permit workflow endpoints without permitting a different program ID."""
    monkeypatch.setenv("CI", "true")
    monkeypatch.setenv("LIGHTCONE_ENV", "staging")
    monkeypatch.setenv("SDK_API_URL", "https://api.dev.lightcone.xyz")
    monkeypatch.setenv("SDK_WS_URL", "wss://ws.dev.lightcone.xyz/ws")
    monkeypatch.setenv("SDK_RPC_URL", "ci-rpc")
    monkeypatch.delenv("SDK_PROGRAM_ID", raising=False)
    _safety_gate(monkeypatch)()

    monkeypatch.setenv("SDK_PROGRAM_ID", "unsafe-program")
    with pytest.raises(RuntimeError, match="unset SDK_PROGRAM_ID"):
        _safety_gate(monkeypatch)()


@pytest.mark.asyncio
async def test_wsol_conversion_example_does_not_retry_uncertain_submission(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Propagate one failed submission without a second attempt."""
    submit_once = cast(
        Callable[[object, Callable[[object], Awaitable[object]]], Awaitable[object]],
        _example_namespace(monkeypatch)["submit_prepared_once"],
    )
    transaction = object()
    attempts = 0

    async def fail_uncertain(submitted: object) -> object:
        nonlocal attempts
        attempts += 1
        assert submitted is transaction
        raise RuntimeError("uncertain confirmation")

    with pytest.raises(RuntimeError, match="uncertain confirmation"):
        await submit_once(transaction, fail_uncertain)
    assert attempts == 1


def test_wsol_conversion_example_requires_covering_snapshot(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Restore action authority only at or beyond the confirmed slot."""
    validate = cast(
        Callable[[int, int], None],
        _example_namespace(monkeypatch)["validate_covering_snapshot_slot"],
    )

    validate(10, 10)
    validate(11, 10)
    with pytest.raises(RuntimeError, match="did not cover"):
        validate(9, 10)


def test_wsol_conversion_example_rejects_negative_projection(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    """Keep the frozen example projection inside unsigned Solana balance units."""
    project = cast(
        Callable[[SolBalanceComponents, SolComponentDelta], SolBalanceComponents],
        _example_namespace(monkeypatch)["project_components"],
    )

    with pytest.raises(ValueError, match="negative frozen SOL projection"):
        project(SolBalanceComponents(1, 0), SolComponentDelta(-2, 0))
