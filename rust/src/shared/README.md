# Shared Module Reference

Shared utilities used across API and WebSocket modules.

## Resolution Enum

Candle resolution for price history queries.

```rust
use lightcone_sdk::shared::Resolution;

let res = Resolution::OneHour;
```

### Variants

| Variant | String | Description |
|---------|--------|-------------|
| `OneMinute` | `"1m"` | 1-minute candles |
| `FiveMinutes` | `"5m"` | 5-minute candles |
| `FifteenMinutes` | `"15m"` | 15-minute candles |
| `OneHour` | `"1h"` | 1-hour candles |
| `FourHours` | `"4h"` | 4-hour candles |
| `OneDay` | `"1d"` | Daily candles |

### Default

```rust
let res = Resolution::default();  // Resolution::OneMinute
```

### Methods

```rust
// Convert to string
let s = res.as_str();      // "1h"
let s = res.to_string();   // "1h"

// Display trait
println!("{}", res);       // "1h"
```

### Traits

- `Debug`, `Clone`, `Copy`
- `Default` (OneMinute)
- `PartialEq`, `Eq`, `Hash`
- `Display`
- `Serialize`, `Deserialize`

### Usage

**REST API:**
```rust
use lightcone_sdk::api::PriceHistoryParams;
use lightcone_sdk::shared::Resolution;

let params = PriceHistoryParams::new("orderbook_id")
    .with_resolution(Resolution::OneHour);

let response = client.get_price_history(params).await?;
```

**WebSocket:**
```rust
// Using string directly
client.subscribe_price_history(
    "orderbook_id".to_string(),
    "1h".to_string(),
    true,
).await?;

// Or using Resolution
client.subscribe_price_history(
    "orderbook_id".to_string(),
    Resolution::OneHour.to_string(),
    true,
).await?;
```

## Decimal Helpers

Functions for converting between string decimals and numeric types.

### parse_decimal

Parses a decimal string to `f64`. Returns `Result<f64, std::num::ParseFloatError>`.

```rust
use lightcone_sdk::shared::parse_decimal;

let price: f64 = parse_decimal("0.500000")?;
let size: f64 = parse_decimal("1000.123456")?;

// Error on invalid input
let result = parse_decimal("invalid");  // Err(ParseFloatError)
```

### format_decimal

Formats a floating point number to a string with specified precision.

```rust
use lightcone_sdk::shared::format_decimal;

let s = format_decimal(0.5, 6);       // "0.500000"
let s = format_decimal(1000.1, 2);    // "1000.10"
let s = format_decimal(0.123456789, 8); // "0.12345679" (rounded)
```

### Example Usage

```rust
use lightcone_sdk::shared::{parse_decimal, format_decimal};

// Calculate position value
let price = parse_decimal(&price_level.price)?;  // "0.650000" -> 0.65
let size = parse_decimal(&price_level.size)?;    // "100.000000" -> 100.0
let value = price * size;                         // 65.0

// Format for display
let display = format_decimal(value, 6);          // "65.000000"
```

## Design Rationale

### Why String Types for Prices?

The API returns decimal values as strings (e.g., `"0.500000"`) rather than floats.

**Reasons:**

1. **Precision Preservation**
   - Floating point has rounding errors (`0.1 + 0.2 ≠ 0.3`)
   - Strings preserve exact server representation
   - No precision loss during JSON serialization

2. **Variable Decimal Places**
   - USDC: 6 decimals
   - SOL: 9 decimals
   - BTC: 8 decimals
   - String representation is agnostic to decimal places

3. **Server Round-Trip**
   - Values returned by the server can be sent back exactly
   - No accumulation of rounding errors

**Best Practices:**

```rust
// Store as strings (as returned by API/WebSocket)
struct OrderbookLevel {
    price: String,  // "0.500000"
    size: String,   // "100.000000"
}

// Parse only when calculations are needed
fn calculate_value(level: &OrderbookLevel) -> Result<f64, Error> {
    let price = parse_decimal(&level.price)?;
    let size = parse_decimal(&level.size)?;
    Ok(price * size)
}

// Format back to string for API submission
fn format_order_price(price: f64, decimals: u8) -> String {
    format_decimal(price, decimals as usize)
}
```

### Token Decimal Reference

| Token | Decimals | Example Value | Raw Units |
|-------|----------|---------------|-----------|
| USDC | 6 | `"1.000000"` | 1_000_000 |
| USDT | 6 | `"1.000000"` | 1_000_000 |
| SOL | 9 | `"1.000000000"` | 1_000_000_000 |
| BTC | 8 | `"1.00000000"` | 100_000_000 |

### Raw Units vs Decimal Strings

The on-chain program uses raw `u64` units (no decimals). The API returns human-readable decimal strings.

**Note:** The SDK does not provide raw unit conversion functions. You must implement conversion based on the token's decimal places.

```rust
// Example: Converting between decimal strings and raw units
// This is application code - you must implement based on your token's decimals

// API response: "1.500000" USDC (6 decimals)
let price_string = "1.500000";

// Convert to raw units for on-chain (user implementation)
let price_decimal = parse_decimal(price_string)?;  // 1.5
let decimals = 6;  // USDC has 6 decimals
let multiplier = 10_u64.pow(decimals);
let raw_units = (price_decimal * multiplier as f64) as u64;  // 1_500_000

// Convert raw units back to string (user implementation)
let back_to_string = format_decimal(raw_units as f64 / multiplier as f64, decimals as usize);
// "1.500000"
```
