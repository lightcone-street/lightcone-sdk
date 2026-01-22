"""REST API client module for Lightcone.

This module provides HTTP client functionality for interacting with
the Lightcone REST API for orderbook data, market info, and more.

Example:
    ```python
    from lightcone_sdk.api import LightconeApiClient

    async with LightconeApiClient("https://api.lightcone.xyz") as client:
        markets = await client.get_markets()
        print(f"Found {markets.total} markets")
    ```
"""

from .client import LightconeApiClient

from .error import (
    ApiError,
    HttpError,
    NotFoundError,
    BadRequestError,
    ForbiddenError,
    UnauthorizedError,
    RateLimitedError,
    ConflictError,
    ServerError,
    DeserializeError,
    InvalidParameterError,
    UnexpectedStatusError,
    ErrorResponse,
)

from .validation import MAX_PAGINATION_LIMIT

from .retry import RetryConfig

from .types import (
    # Market types
    ApiMarketStatus,
    Outcome,
    OrderbookSummary,
    ConditionalToken,
    DepositAsset,
    Market,
    MarketsResponse,
    MarketInfoResponse,
    DepositAssetsResponse,
    # Orderbook types
    PriceLevel,
    OrderbookResponse,
    # Order types
    ApiOrderSide,
    OrderStatus,
    Fill,
    SubmitOrderRequest,
    OrderResponse,
    CancelOrderRequest,
    CancelResponse,
    CancelAllOrdersRequest,
    CancelAllResponse,
    UserOrder,
    UserOrderOutcomeBalance,
    UserBalance,
    GetUserOrdersRequest,
    UserOrdersResponse,
    # Position types
    OutcomeBalance,
    Position,
    PositionsResponse,
    MarketPositionsResponse,
    # Price history types
    PricePoint,
    PriceHistoryParams,
    PriceHistoryResponse,
    # Trade types
    ApiTradeSide,
    Trade,
    TradesParams,
    TradesResponse,
    # Admin types
    AdminResponse,
    CreateOrderbookRequest,
    CreateOrderbookResponse,
)

__all__ = [
    # Client
    "LightconeApiClient",
    # Errors
    "ApiError",
    "HttpError",
    "NotFoundError",
    "BadRequestError",
    "ForbiddenError",
    "UnauthorizedError",
    "RateLimitedError",
    "ConflictError",
    "ServerError",
    "DeserializeError",
    "InvalidParameterError",
    "UnexpectedStatusError",
    "ErrorResponse",
    # Constants
    "MAX_PAGINATION_LIMIT",
    # Retry
    "RetryConfig",
    # Market types
    "ApiMarketStatus",
    "Outcome",
    "OrderbookSummary",
    "ConditionalToken",
    "DepositAsset",
    "Market",
    "MarketsResponse",
    "MarketInfoResponse",
    "DepositAssetsResponse",
    # Orderbook types
    "PriceLevel",
    "OrderbookResponse",
    # Order types
    "ApiOrderSide",
    "OrderStatus",
    "Fill",
    "SubmitOrderRequest",
    "OrderResponse",
    "CancelOrderRequest",
    "CancelResponse",
    "CancelAllOrdersRequest",
    "CancelAllResponse",
    "UserOrder",
    "UserOrderOutcomeBalance",
    "UserBalance",
    "GetUserOrdersRequest",
    "UserOrdersResponse",
    # Position types
    "OutcomeBalance",
    "Position",
    "PositionsResponse",
    "MarketPositionsResponse",
    # Price history types
    "PricePoint",
    "PriceHistoryParams",
    "PriceHistoryResponse",
    # Trade types
    "ApiTradeSide",
    "Trade",
    "TradesParams",
    "TradesResponse",
    # Admin types
    "AdminResponse",
    "CreateOrderbookRequest",
    "CreateOrderbookResponse",
]
