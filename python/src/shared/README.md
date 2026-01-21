# Shared Module Reference

Minimal shared utilities used across API and WebSocket modules.

> **Note:** Program-specific code (types, constants, errors, utilities) has been moved to the `program` module. This module now contains only items shared between the API and WebSocket modules.

## Contents

This module exports:
- `Resolution` - Price history candle intervals
- `parse_decimal()` - Parse decimal strings
- `format_decimal()` - Format decimal strings

## Resolution

Enum for price history candle intervals. Used by both REST API and WebSocket for price history queries.

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

## Price Utilities

```python
from lightcone_sdk.shared import parse_decimal, format_decimal

# Parse a decimal string to float
price = parse_decimal("0.542500")  # Returns 0.5425

# Format a float as decimal string
formatted = format_decimal(0.5425, precision=6)  # Returns "0.542500"
```

## For Program-Specific Code

All program-specific code has been moved to the `program` module:

```python
# Types, constants, errors, and utilities are now in program module
from lightcone_sdk.program import (
    # Types
    MarketStatus,
    OrderSide,
    Exchange,
    Market,
    FullOrder,

    # Constants
    PROGRAM_ID,
    MAX_OUTCOMES,
    FULL_ORDER_SIZE,

    # Errors
    LightconeError,
    InvalidOrderError,

    # Utilities
    keccak256,
    derive_condition_id,
    get_associated_token_address,
)
```

See the [Program Module README](../program/README.md) for complete documentation.
