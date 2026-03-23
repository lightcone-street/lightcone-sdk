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


class TooManyMakersError(LightconeError):
    """Raised when too many makers are specified for a match."""

    def __init__(self, count: int, max_count: int):
        self.count = count
        self.max_count = max_count
        super().__init__(
            f"Too many makers: {count} (maximum: {max_count})"
        )


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


class UnsignedOrderError(LightconeError):
    """Raised when an operation requires a signed order but receives unsigned."""

    def __init__(self):
        super().__init__("Order is not signed")
