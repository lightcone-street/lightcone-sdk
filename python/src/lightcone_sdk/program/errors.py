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


# =========================================================================
# Additional error types matching Rust SdkError variants
# =========================================================================


class InvalidDataLengthError(LightconeError):
    """Raised when account data has an invalid length."""

    def __init__(self, expected: int, actual: int):
        self.expected = expected
        self.actual = actual
        super().__init__(f"Invalid data length: expected {expected}, got {actual}")


class InvalidOutcomeCountError(LightconeError):
    """Raised when the number of outcomes is invalid."""

    def __init__(self, count: int):
        self.count = count
        super().__init__(f"Invalid outcome count: {count}")


class InvalidOutcomeIndexError(LightconeError):
    """Raised when an outcome index is out of range."""

    def __init__(self, index: int, max_index: int):
        self.index = index
        self.max_index = max_index
        super().__init__(f"Invalid outcome index {index}, max {max_index}")


class SignatureVerificationFailedError(LightconeError):
    """Raised when Ed25519 signature verification fails."""

    def __init__(self):
        super().__init__("Signature verification failed")


class SerializationError(LightconeError):
    """Raised when serialization or deserialization fails."""

    def __init__(self, message: str):
        super().__init__(f"Serialization error: {message}")


class InvalidSideError(LightconeError):
    """Raised when an invalid order side value is encountered."""

    def __init__(self, side: int):
        self.side = side
        super().__init__(f"Invalid order side: {side}")


class InvalidMarketStatusError(LightconeError):
    """Raised when an invalid market status value is encountered."""

    def __init__(self, status: int):
        self.status = status
        super().__init__(f"Invalid market status: {status}")


class MissingFieldError(LightconeError):
    """Raised when a required field is missing."""

    def __init__(self, field: str):
        self.field = field
        super().__init__(f"Missing required field: {field}")


class ArithmeticOverflowError(LightconeError):
    """Raised when an arithmetic operation overflows."""

    def __init__(self):
        super().__init__("Arithmetic overflow")


class InvalidMintOrderError(LightconeError):
    """Raised when mints are in invalid order."""

    def __init__(self):
        super().__init__("Invalid mint order")


class OrderbookExistsError(LightconeError):
    """Raised when trying to create an orderbook that already exists."""

    def __init__(self):
        super().__init__("Orderbook already exists")


class InvalidMarketError(LightconeError):
    """Raised when a market is invalid."""

    def __init__(self):
        super().__init__("Invalid market")


class MarketSettledError(LightconeError):
    """Raised when a market is already settled."""

    def __init__(self):
        super().__init__("Market is already settled")


class InvalidProgramIdError(LightconeError):
    """Raised when the program ID is invalid."""

    def __init__(self):
        super().__init__("Invalid program ID")


class InvalidManagerError(LightconeError):
    """Raised when a signer is not the exchange manager."""

    def __init__(self):
        super().__init__("Invalid manager")


class InvalidOrderbookError(LightconeError):
    """Raised when an orderbook is invalid."""

    def __init__(self):
        super().__init__("Invalid orderbook")


class FullFillRequiredError(LightconeError):
    """Raised when a full fill is required but not provided."""

    def __init__(self):
        super().__init__("Full fill required")


class DivisionByZeroError(LightconeError):
    """Raised when a division by zero is attempted."""

    def __init__(self):
        super().__init__("Division by zero")


class DepositTokenNotActiveError(LightconeError):
    """Raised when a deposit token is not active."""

    def __init__(self):
        super().__init__("Deposit token not active")


class InvalidPubkeyError(LightconeError):
    """Raised when a public key is invalid."""

    def __init__(self, pubkey: str):
        self.pubkey = pubkey
        super().__init__(f"Invalid pubkey: {pubkey}")


class ScalingError(LightconeError):
    """Raised when price/size scaling fails."""

    def __init__(self, message: str):
        super().__init__(f"Scaling error: {message}")


class UnsignedOrderError(LightconeError):
    """Raised when an operation requires a signed order but receives unsigned."""

    def __init__(self):
        super().__init__("Order is not signed")
