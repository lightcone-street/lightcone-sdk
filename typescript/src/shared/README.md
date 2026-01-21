# Shared Module Reference

Minimal shared utilities used across API and WebSocket modules.

> **Note:** Program-specific code (types, constants, errors, utilities) has been moved to the `program` module. This module now contains only items shared between the API and WebSocket modules.

## Contents

This module exports:
- `Resolution` - Price history candle intervals
- `parseDecimal()` - Parse decimal strings
- `formatDecimal()` - Format decimal strings

## Resolution

Enum for price history candle intervals. Used by both REST API and WebSocket for price history queries.

```typescript
import { Resolution } from "@lightcone/sdk";

// Values
Resolution.OneMinute      // "1m"
Resolution.FiveMinutes    // "5m"
Resolution.FifteenMinutes // "15m"
Resolution.OneHour        // "1h"
Resolution.FourHours      // "4h"
Resolution.OneDay         // "1d"
```

### Usage

```typescript
import { Resolution } from "@lightcone/sdk";

// Use in API calls
const response = await client.getPriceHistory({
  orderbook_id: "orderbook_id",
  resolution: Resolution.OneHour,
});

// Use in WebSocket subscriptions
client.subscribePriceHistory("orderbook_id", Resolution.OneHour, true);
```

### Resolution Mapping

| Enum | String | Description |
|------|--------|-------------|
| `OneMinute` | "1m" | 1-minute candles |
| `FiveMinutes` | "5m" | 5-minute candles |
| `FifteenMinutes` | "15m" | 15-minute candles |
| `OneHour` | "1h" | 1-hour candles |
| `FourHours` | "4h" | 4-hour candles |
| `OneDay` | "1d" | Daily candles |

## Price Utilities

```typescript
import { parseDecimal, formatDecimal } from "@lightcone/sdk";

// Parse decimal string to number
const price = parseDecimal("0.500000");  // 0.5

// Format number as decimal string
const priceStr = formatDecimal(0.5, 6);  // "0.500000"
```

## For Program-Specific Code

All program-specific code has been moved to the `program` module:

```typescript
// Types, constants, and utilities are now in program module
import {
  // Types
  MarketStatus,
  OrderSide,
  Exchange,
  Market,
  FullOrder,

  // Constants
  PROGRAM_ID,
  MAX_OUTCOMES,
  ORDER_SIZE,

  // Utilities
  keccak256,
  deriveConditionId,
  getAssociatedTokenAddress,
} from "@lightcone/sdk";
```

See the [Program Module README](../program/README.md) for complete documentation.
