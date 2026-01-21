# Shared Module Reference

Common utilities, types, and constants used across all SDK modules.

## Resolution

Enum for price history candle intervals.

```python
from lightcone_sdk.shared import Resolution

# Values
Resolution.ONE_MINUTE      # "1m"
Resolution.FIVE_MINUTES    # "5m"
Resolution.FIFTEEN_MINUTES # "15m"
Resolution.ONE_HOUR        # "1h"
Resolution.FOUR_HOURS      # "4h"
Resolution.ONE_DAY         # "1d"
```

### Usage

```python
from lightcone_sdk.shared import Resolution

# Get string representation
res = Resolution.ONE_HOUR
print(res.as_str())  # "1h"
print(str(res))      # "1h"

# Parse from string
res = Resolution.from_str("5m")
assert res == Resolution.FIVE_MINUTES

# Use in API calls
from lightcone_sdk.api import PriceHistoryParams

params = PriceHistoryParams.new("orderbook_id").with_resolution(Resolution.ONE_HOUR)
```

### Resolution Mapping

| Enum | String | Description |
|------|--------|-------------|
| `ONE_MINUTE` | "1m" | 1-minute candles |
| `FIVE_MINUTES` | "5m" | 5-minute candles |
| `FIFTEEN_MINUTES` | "15m" | 15-minute candles |
| `ONE_HOUR` | "1h" | 1-hour candles |
| `FOUR_HOURS` | "4h" | 4-hour candles |
| `ONE_DAY` | "1d" | Daily candles |

## Constants

### Program IDs

```python
from lightcone_sdk.shared import (
    PROGRAM_ID,
    TOKEN_PROGRAM_ID,
    TOKEN_2022_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID,
    SYSTEM_PROGRAM_ID,
    RENT_SYSVAR_ID,
    INSTRUCTIONS_SYSVAR_ID,
    ED25519_PROGRAM_ID,
)
```

| Constant | Value | Description |
|----------|-------|-------------|
| `PROGRAM_ID` | `Aumw7EC9nnxDjQFzr1fhvXvnG3Rn3Bb5E3kbcbLrBdEk` | Lightcone program |
| `TOKEN_PROGRAM_ID` | `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA` | SPL Token program |
| `TOKEN_2022_PROGRAM_ID` | `TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb` | Token-2022 program |
| `ASSOCIATED_TOKEN_PROGRAM_ID` | `ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL` | ATA program |
| `SYSTEM_PROGRAM_ID` | `11111111111111111111111111111111` | System program |
| `RENT_SYSVAR_ID` | `SysvarRent111111111111111111111111111111111` | Rent sysvar |
| `INSTRUCTIONS_SYSVAR_ID` | `Sysvar1nstructions1111111111111111111111111` | Instructions sysvar |
| `ED25519_PROGRAM_ID` | `Ed25519SigVerify111111111111111111111111111` | Ed25519 verify program |

### PDA Seeds

```python
from lightcone_sdk.shared import (
    SEED_CENTRAL_STATE,
    SEED_MARKET,
    SEED_VAULT,
    SEED_MINT_AUTHORITY,
    SEED_CONDITIONAL_MINT,
    SEED_ORDER_STATUS,
    SEED_USER_NONCE,
    SEED_POSITION,
)
```

| Constant | Value | Description |
|----------|-------|-------------|
| `SEED_CENTRAL_STATE` | `b"central_state"` | Exchange PDA seed |
| `SEED_MARKET` | `b"market"` | Market PDA seed |
| `SEED_VAULT` | `b"market_deposit_token_account"` | Vault PDA seed |
| `SEED_MINT_AUTHORITY` | `b"market_mint_authority"` | Mint authority PDA seed |
| `SEED_CONDITIONAL_MINT` | `b"conditional_mint"` | Conditional token mint seed |
| `SEED_ORDER_STATUS` | `b"order_status"` | Order status PDA seed |
| `SEED_USER_NONCE` | `b"user_nonce"` | User nonce PDA seed |
| `SEED_POSITION` | `b"position"` | Position PDA seed |

### Account Discriminators

```python
from lightcone_sdk.shared import (
    EXCHANGE_DISCRIMINATOR,
    MARKET_DISCRIMINATOR,
    ORDER_STATUS_DISCRIMINATOR,
    POSITION_DISCRIMINATOR,
    USER_NONCE_DISCRIMINATOR,
)
```

| Constant | Value | Length |
|----------|-------|--------|
| `EXCHANGE_DISCRIMINATOR` | `b"exchange"` | 8 bytes |
| `MARKET_DISCRIMINATOR` | `b"market\x00\x00"` | 8 bytes |
| `ORDER_STATUS_DISCRIMINATOR` | `b"ordstat\x00"` | 8 bytes |
| `POSITION_DISCRIMINATOR` | `b"position"` | 8 bytes |
| `USER_NONCE_DISCRIMINATOR` | `b"usrnonce"` | 8 bytes |

### Account Sizes

```python
from lightcone_sdk.shared import (
    EXCHANGE_SIZE,
    MARKET_SIZE,
    ORDER_STATUS_SIZE,
    USER_NONCE_SIZE,
    POSITION_SIZE,
)
```

| Constant | Value | Description |
|----------|-------|-------------|
| `EXCHANGE_SIZE` | 88 | Exchange account size |
| `MARKET_SIZE` | 120 | Market account size |
| `ORDER_STATUS_SIZE` | 24 | OrderStatus account size |
| `USER_NONCE_SIZE` | 16 | UserNonce account size |
| `POSITION_SIZE` | 80 | Position account size |

### Order Sizes

```python
from lightcone_sdk.shared import (
    FULL_ORDER_SIZE,
    COMPACT_ORDER_SIZE,
    SIGNATURE_SIZE,
    ORDER_HASH_SIZE,
)
```

| Constant | Value | Description |
|----------|-------|-------------|
| `FULL_ORDER_SIZE` | 225 | Full order with signature |
| `COMPACT_ORDER_SIZE` | 65 | Compact order (no market/mints) |
| `SIGNATURE_SIZE` | 64 | Ed25519 signature size |
| `ORDER_HASH_SIZE` | 32 | Keccak256 hash size |

### Limits

```python
from lightcone_sdk.shared import (
    MAX_OUTCOMES,
    MIN_OUTCOMES,
    MAX_MAKERS,
)
```

| Constant | Value | Description |
|----------|-------|-------------|
| `MAX_OUTCOMES` | 6 | Maximum outcomes per market |
| `MIN_OUTCOMES` | 2 | Minimum outcomes per market |
| `MAX_MAKERS` | 5 | Maximum makers per match |

## Types

### MarketStatus

```python
from lightcone_sdk.shared import MarketStatus

MarketStatus.PENDING    # 0 - Not yet active
MarketStatus.ACTIVE     # 1 - Trading enabled
MarketStatus.RESOLVED   # 2 - Market settled
MarketStatus.CANCELLED  # 3 - Market cancelled
```

### OrderSide

```python
from lightcone_sdk.shared import OrderSide

OrderSide.BID  # 0 - Buyer gives quote, receives base
OrderSide.ASK  # 1 - Seller gives base, receives quote
```

### Account Types

```python
from lightcone_sdk.shared import (
    Exchange,
    Market,
    Position,
    OrderStatus,
    UserNonce,
)
```

#### Exchange

| Field | Type | Description |
|-------|------|-------------|
| `authority` | Pubkey | Admin authority |
| `operator` | Pubkey | Order matching operator |
| `market_count` | int | Number of markets created |
| `paused` | bool | Trading paused |
| `bump` | int | PDA bump seed |

#### Market

| Field | Type | Description |
|-------|------|-------------|
| `market_id` | int | Sequential market ID |
| `num_outcomes` | int | Number of outcomes |
| `status` | MarketStatus | Current status |
| `winning_outcome` | Optional[int] | Winner (if settled) |
| `bump` | int | PDA bump seed |
| `oracle` | Pubkey | Oracle authority |
| `question_id` | bytes | Question identifier |
| `condition_id` | bytes | Computed condition ID |

#### Position

| Field | Type | Description |
|-------|------|-------------|
| `owner` | Pubkey | Position owner |
| `market` | Pubkey | Market address |
| `bump` | int | PDA bump seed |

#### OrderStatus

| Field | Type | Description |
|-------|------|-------------|
| `remaining` | int | Remaining order amount |
| `is_cancelled` | bool | Cancelled flag |

#### UserNonce

| Field | Type | Description |
|-------|------|-------------|
| `nonce` | int | Current nonce value |

### Order Types

```python
from lightcone_sdk.shared import FullOrder, CompactOrder, MakerFill
```

#### FullOrder (225 bytes)

| Field | Type | Description |
|-------|------|-------------|
| `nonce` | int | Order nonce |
| `maker` | Pubkey | Maker public key |
| `market` | Pubkey | Market address |
| `base_mint` | Pubkey | Base token mint |
| `quote_mint` | Pubkey | Quote token mint |
| `side` | OrderSide | BID or ASK |
| `maker_amount` | int | Amount maker gives |
| `taker_amount` | int | Amount maker receives |
| `expiration` | int | Expiration timestamp |
| `signature` | bytes | Ed25519 signature (64 bytes) |

#### CompactOrder (65 bytes)

Same as FullOrder but without `market`, `base_mint`, `quote_mint` (derived from instruction context).

| Field | Type | Description |
|-------|------|-------------|
| `nonce` | int | Order nonce |
| `maker` | Pubkey | Maker public key |
| `side` | OrderSide | BID or ASK |
| `maker_amount` | int | Amount maker gives |
| `taker_amount` | int | Amount maker receives |
| `expiration` | int | Expiration timestamp |

#### MakerFill

| Field | Type | Description |
|-------|------|-------------|
| `order` | FullOrder | The maker order |
| `fill_amount` | int | Amount to fill |

### Parameter Types

```python
from lightcone_sdk.shared import (
    InitializeParams,
    CreateMarketParams,
    AddDepositMintParams,
    MintCompleteSetParams,
    MergeCompleteSetParams,
    SettleMarketParams,
    RedeemWinningsParams,
    WithdrawFromPositionParams,
    ActivateMarketParams,
    MatchOrdersMultiParams,
    BidOrderParams,
    AskOrderParams,
    OutcomeMetadata,
)
```

## Utilities

### Hashing

```python
from lightcone_sdk.shared import keccak256

# Hash arbitrary data
hash_bytes = keccak256(b"data to hash")  # Returns 32 bytes
```

### Condition ID

```python
from lightcone_sdk.shared import derive_condition_id

condition_id = derive_condition_id(
    oracle_pubkey,     # Pubkey
    question_id,       # bytes (32)
    num_outcomes,      # int
)
```

### Associated Token Addresses

```python
from lightcone_sdk.shared import (
    get_associated_token_address,
    get_associated_token_address_2022,
)

# SPL Token
ata = get_associated_token_address(owner_pubkey, mint_pubkey)

# Token-2022
ata = get_associated_token_address_2022(owner_pubkey, mint_pubkey)
```

### Order Crossing

```python
from lightcone_sdk.shared import orders_cross

# Check if a bid and ask order would match
crosses = orders_cross(bid_order, ask_order)  # Returns bool
```

## Errors

```python
from lightcone_sdk.shared import (
    LightconeError,
    InvalidDiscriminatorError,
    AccountNotFoundError,
    InvalidAccountDataError,
    InvalidOrderError,
    InvalidSignatureError,
    OrderExpiredError,
    InsufficientBalanceError,
    MarketNotActiveError,
    ExchangePausedError,
    InvalidOutcomeError,
    TooManyMakersError,
    OrdersDoNotCrossError,
)
```

| Error | Description |
|-------|-------------|
| `LightconeError` | Base error class |
| `InvalidDiscriminatorError` | Account discriminator mismatch |
| `AccountNotFoundError` | Account does not exist |
| `InvalidAccountDataError` | Account data malformed |
| `InvalidOrderError` | Order validation failed |
| `InvalidSignatureError` | Signature verification failed |
| `OrderExpiredError` | Order has expired |
| `InsufficientBalanceError` | Insufficient funds |
| `MarketNotActiveError` | Market not in active state |
| `ExchangePausedError` | Exchange is paused |
| `InvalidOutcomeError` | Invalid outcome index |
| `TooManyMakersError` | Exceeds MAX_MAKERS |
| `OrdersDoNotCrossError` | Orders don't match |

## Design Rationale

### Decimal Strings

All prices, sizes, and amounts are represented as **decimal strings** rather than floats or integers. This matches the Lightcone API format and preserves exact precision for financial calculations.

```python
# Prices/sizes from API/WebSocket
price = "0.542500"
size = "1000.000000"

# Convert to float/Decimal when needed
from decimal import Decimal
price_decimal = Decimal(price)
```

### String Keys

Orderbook IDs and public keys are consistently represented as strings (base58 for pubkeys) for JSON serialization compatibility.

### Immutable Data Classes

Account types are `@dataclass` instances with immutable semantics. State updates create new instances rather than mutating existing ones.
