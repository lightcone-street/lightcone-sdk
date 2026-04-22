"""Testnet faucet types — used by ``LightconeClient.claim`` to request testnet
SOL and whitelisted deposit tokens.

There is no faucet sub-client; the single entry point is the ``claim()``
method on the root client.
"""

from __future__ import annotations

from dataclasses import dataclass, field


@dataclass
class FaucetRequest:
    """Request payload for ``POST /api/claim``."""

    wallet_address: str

    def to_dict(self) -> dict:
        return {"wallet_address": self.wallet_address}


@dataclass
class FaucetToken:
    """A single token minted to the wallet by the faucet."""

    symbol: str = ""
    amount: int = 0

    @staticmethod
    def from_dict(d: dict) -> "FaucetToken":
        return FaucetToken(
            symbol=d.get("symbol", ""),
            amount=int(d.get("amount", 0)),
        )


@dataclass
class FaucetResponse:
    """Response from ``POST /api/claim``."""

    signature: str = ""
    sol: float = 0.0
    tokens: list[FaucetToken] = field(default_factory=list)

    @staticmethod
    def from_dict(d: dict) -> "FaucetResponse":
        return FaucetResponse(
            signature=d.get("signature", ""),
            sol=float(d.get("sol", 0.0)),
            tokens=[FaucetToken.from_dict(t) for t in d.get("tokens", [])],
        )


__all__ = ["FaucetRequest", "FaucetResponse", "FaucetToken"]
