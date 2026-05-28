"""Environment configuration for the Lightcone SDK.

The :class:`LightconeEnv` enum determines which Lightcone deployment the SDK
connects to. Each variant maps to a specific API URL, WebSocket URL,
Solana RPC URL, and on-chain program ID.
"""

import os
from enum import Enum

from solders.pubkey import Pubkey


class LightconeEnv(Enum):
    """Lightcone deployment environment.

    Pass to :meth:`LightconeClientBuilder.env` to configure the client for a
    specific deployment.  Defaults to :attr:`PROD` when not specified.

    Example::

        client = LightconeClientBuilder().env(LightconeEnv.STAGING).build()
    """

    LOCAL = "local"
    STAGING = "staging"
    PROD = "prod"

    @property
    def api_url(self) -> str:
        """REST API base URL for this environment."""
        return {
            LightconeEnv.LOCAL: "https://api.local.lightcone.xyz",
            LightconeEnv.STAGING: "https://api.staging.lightcone.xyz",
            LightconeEnv.PROD: "https://api.lightcone.xyz",
        }[self]

    @property
    def ws_url(self) -> str:
        """WebSocket URL for this environment."""
        return {
            LightconeEnv.LOCAL: "wss://ws.local.lightcone.xyz/ws",
            LightconeEnv.STAGING: "wss://ws.staging.lightcone.xyz/ws",
            LightconeEnv.PROD: "wss://ws.lightcone.xyz/ws",
        }[self]

    @property
    def rpc_url(self) -> str:
        """Solana RPC URL for this environment."""
        return {
            LightconeEnv.LOCAL: "https://api.devnet.solana.com",
            LightconeEnv.STAGING: "https://api.devnet.solana.com",
            LightconeEnv.PROD: "https://api.devnet.solana.com",
        }[self]

    @property
    def program_id(self) -> Pubkey:
        """On-chain Lightcone program ID for this environment.

        If the ``SDK_PROGRAM_ID`` environment variable is set, its value is
        used regardless of the selected environment.
        """
        override_id = os.environ.get("SDK_PROGRAM_ID")
        if override_id:
            return Pubkey.from_string(override_id)
        if self in (LightconeEnv.LOCAL, LightconeEnv.STAGING):
            return Pubkey.from_string(
                "FAq4NbwPVWNzoaNjcJGhWz4VFT5CbdysLPo7ZWWiWuuE"
            )
        return Pubkey.from_string(
            "8nzsoyHZFYig3uN3M717Q47MtLqzx2V2UAKaPTqDy5rV"
        )

    def __str__(self) -> str:
        return self.value


# Default program ID (production). Used as the default argument in PDA and
# instruction helper functions. When targeting a non-production environment,
# always pass ``program_id`` explicitly via ``LightconeClient.program_id``
# or ``LightconeEnv.<env>.program_id``.
PROGRAM_ID = LightconeEnv.PROD.program_id
