"""Wire-compatible jurisdiction response models."""

from __future__ import annotations

from dataclasses import dataclass


def _required_bool(data: dict, field: str) -> bool:
    value = data[field]
    if type(value) is not bool:
        raise ValueError(f"{field} must be a boolean")
    return value


@dataclass(frozen=True)
class JurisdictionCapabilities:
    mode: str
    can_authenticate: bool
    can_mutate_account: bool
    can_submit_orders: bool
    can_cancel_orders: bool

    @classmethod
    def from_dict(cls, data: dict) -> JurisdictionCapabilities:
        return cls(
            mode=data["mode"],
            can_authenticate=_required_bool(data, "can_authenticate"),
            can_mutate_account=_required_bool(data, "can_mutate_account"),
            can_submit_orders=_required_bool(data, "can_submit_orders"),
            can_cancel_orders=_required_bool(data, "can_cancel_orders"),
        )


@dataclass(frozen=True)
class JurisdictionResponse:
    country: str | None
    geoblocked: bool
    tier: str | None
    policy_version: str
    frontend: JurisdictionCapabilities
    api: JurisdictionCapabilities
    relayed: bool

    @classmethod
    def from_dict(cls, data: dict) -> JurisdictionResponse:
        return cls(
            country=data["country"],
            geoblocked=_required_bool(data, "geoblocked"),
            tier=data["tier"],
            policy_version=data["policy_version"],
            frontend=JurisdictionCapabilities.from_dict(data["frontend"]),
            api=JurisdictionCapabilities.from_dict(data["api"]),
            relayed=_required_bool(data, "relayed"),
        )
