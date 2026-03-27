"""Environment configuration for the Lightcone SDK.

The :class:`LightconeEnv` enum determines which Lightcone deployment the SDK
connects to. Each variant maps to a specific API URL, WebSocket URL,
Solana RPC URL, and on-chain program ID.
"""

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
            LightconeEnv.LOCAL: "https://local-api.lightcone.xyz",
            LightconeEnv.STAGING: "https://tapi2.lightcone.xyz",
            LightconeEnv.PROD: "https://tapi.lightcone.xyz",
        }[self]

    @property
    def ws_url(self) -> str:
        """WebSocket URL for this environment."""
        return {
            LightconeEnv.LOCAL: "wss://local-ws.lightcone.xyz/ws",
            LightconeEnv.STAGING: "wss://tws2.lightcone.xyz/ws",
            LightconeEnv.PROD: "wss://tws.lightcone.xyz/ws",
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
        """On-chain Lightcone program ID for this environment."""
        if self in (LightconeEnv.LOCAL, LightconeEnv.STAGING):
            return Pubkey.from_string(
                "H3qkHTWUDUUw4ZvGNPdwdU4CYqks69bijo1CzVR12mq"
            )
        return Pubkey.from_string(
            "8nzsoyHZFYig3uN3M717Q47MtLqzx2V2UAKaPTqDy5rV"
        )

    def __str__(self) -> str:
        return self.value
