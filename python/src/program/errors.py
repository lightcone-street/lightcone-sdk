"""Custom exceptions for the Lightcone program module."""


class LightconeError(Exception):
    """Base exception for all Lightcone SDK errors."""

    pass


class InvalidDiscriminatorError(LightconeError):
    """Raised when account data has an invalid discriminator."""

    def __init__(self, expected: bytes, actual: bytes):
        self.expected = expected
        self.actual = actual
        super().__init__(
            f"Invalid discriminator: expected {expected!r}, got {actual!r}"
        )


class AccountNotFoundError(LightconeError):
    """Raised when an account is not found on-chain."""

    def __init__(self, address: str):
        self.address = address
        super().__init__(f"Account not found: {address}")


class InvalidAccountDataError(LightconeError):
    """Raised when account data cannot be deserialized."""

    def __init__(self, message: str):
        super().__init__(f"Invalid account data: {message}")


class InvalidOrderError(LightconeError):
    """Raised when an order is invalid."""

    def __init__(self, message: str):
        super().__init__(f"Invalid order: {message}")


class InvalidSignatureError(LightconeError):
    """Raised when a signature is invalid."""

    def __init__(self, message: str = "Invalid signature"):
        super().__init__(message)


class OrderExpiredError(LightconeError):
    """Raised when an order has expired."""

    def __init__(self, expiration: int, current_time: int):
        self.expiration = expiration
        self.current_time = current_time
        super().__init__(
            f"Order expired: expiration={expiration}, current_time={current_time}"
        )


class InsufficientBalanceError(LightconeError):
    """Raised when there is insufficient balance for an operation."""

    def __init__(self, required: int, available: int):
        self.required = required
        self.available = available
        super().__init__(
            f"Insufficient balance: required={required}, available={available}"
        )


class MarketNotActiveError(LightconeError):
    """Raised when trying to operate on a market that is not active."""

    def __init__(self, market_id: int, status: str):
        self.market_id = market_id
        self.status = status
        super().__init__(f"Market {market_id} is not active (status: {status})")


class ExchangePausedError(LightconeError):
    """Raised when the exchange is paused."""

    def __init__(self):
        super().__init__("Exchange is currently paused")


class InvalidOutcomeError(LightconeError):
    """Raised when an invalid outcome is specified."""

    def __init__(self, outcome: int, num_outcomes: int):
        self.outcome = outcome
        self.num_outcomes = num_outcomes
        super().__init__(
            f"Invalid outcome: {outcome} (valid range: 0-{num_outcomes - 1})"
        )


class TooManyMakersError(LightconeError):
    """Raised when too many makers are specified for a match."""

    def __init__(self, count: int, max_count: int):
        self.count = count
        self.max_count = max_count
        super().__init__(
            f"Too many makers: {count} (maximum: {max_count})"
        )


class OrdersDoNotCrossError(LightconeError):
    """Raised when orders do not cross (prices don't match)."""

    def __init__(self):
        super().__init__("Orders do not cross: buyer price < seller price")
