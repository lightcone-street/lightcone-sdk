# `Decimal` — bindings to `decimal.js`

ReScript bindings to [`decimal.js`](https://github.com/MikeMcl/decimal.js), used for
**arbitrary-precision decimal arithmetic** — price/size scaling and display formatting
that must not drift into binary float error. This mirrors the Rust SDK's `rust_decimal`
usage (see `program/Scaling.res`, which scales human prices/sizes to on-chain lamports).

Written for someone who knows `decimal.js` but is new to ReScript bindings. We bind only
the constructors and operations the SDK needs; the upstream `Decimal` class has much more.

> Compatibility: `decimal.js` **v10**. The default export is the `Decimal` class; every
> binding here is either a constructor (`new Decimal(value)`) or an instance method.
> `Decimal.Value` accepts `string | number | bigint | Decimal` — we surface the three
> source types the SDK uses via `fromString` / `fromInt` / `fromFloat`.
>
> See [`tests/`](./tests) for the runnable suite and [`tests/README.md`](./tests/README.md)
> for the coverage matrix.

## Setup

The module is part of the SDK package (no extra `rescript.json` dependency). `decimal.js`
must be installed (it is an SDK dependency). Reference it directly — `Decimal.fromString`,
`Decimal.times`, etc. No `open` is needed (and there is no namespace prefix).

None of the bindings take an optional last argument, so there is **no trailing-`()`**
calling convention to worry about here — every function takes its arguments positionally
and you pipe handles through with `->`.

## How to read returned values

`Decimal.t` is an **opaque handle** — a wrapped `Decimal` instance. You never read it
directly. You either **chain another operation** (every arithmetic/rounding binding takes
a `t` and returns a new `t`) or **convert to a primitive with a terminal accessor**:

- **`t` → `string`**: `->Decimal.toString` (canonical), or `->Decimal.toFixed(dp)` for a
  fixed number of decimal places (always emits exactly `dp` digits, e.g. `"3.0"`).
- **`t` → `float`**: `->Decimal.toNumber` (may lose precision — only for display/JS interop,
  never for further exact math).
- **`t` → `t` (rounded)**: `toSignificantDigits(n)`, `toDecimalPlaces(dp, roundingMode)`,
  `floor`/`ceil`/`round`/`abs` — still opaque; chain or convert afterwards.
- **comparison → `int`**: `cmp` returns `-1 | 0 | 1` (less / equal / greater).
- **predicate → `bool`**: `eq` / `gt` / `gte` / `lt` / `lte` / `isZero` / `isNeg`.

So the rule of thumb: build and combine with handle-returning bindings, then call exactly
one terminal accessor (`toString` / `toFixed` / `toNumber`) or predicate at the end.

### Rounding modes

`toDecimalPlaces(value, dp, roundingMode)` takes a `decimal.js` rounding-mode integer. The
binding exports the one the SDK uses:

- `Decimal.roundDown` (`= 1`, `ROUND_DOWN`) — truncates toward zero (e.g. `1.2399 → 1.23`).

For other modes pass the raw `decimal.js` constant (see Escape hatches).

## Quick start

```rescript
// 1.5 * 2 = 3, formatted to one decimal place
let formatted = Decimal.fromString("1.5")->Decimal.times(Decimal.fromInt(2))->Decimal.toFixed(1)
// formatted == "3.0"

// exact decimal addition (no binary float drift)
let exact = Decimal.fromString("0.1")->Decimal.plus(Decimal.fromString("0.2"))->Decimal.toFixed(1)
// exact == "0.3"
```

## Reference

### Construction — `string | int | float` → `Decimal.t`

| Binding | Signature | Notes |
|---|---|---|
| `fromString` | `string => t` | `new Decimal("1.5")`. The exact-precision path — prefer this for user input and on-chain values. |
| `fromInt` | `int => t` | `new Decimal(42)`. |
| `fromFloat` | `float => t` | `new Decimal(2.5)`. Constructs from a JS number; only as exact as the float you pass in. |

### Arithmetic — `(t, t) => t` (chainable)

| Binding | Signature | Notes |
|---|---|---|
| `plus` | `(t, t) => t` | `a.plus(b)`. |
| `minus` | `(t, t) => t` | `a.minus(b)`. |
| `times` | `(t, t) => t` | `a.times(b)`. |
| `div` | `(t, t) => t` | `a.div(b)`. |
| `pow` | `(t, t) => t` | `a.pow(b)` — base and exponent both `t`. |
| `powInt` | `(t, int) => t` | Same JS method (`pow`), `int` exponent convenience — e.g. `10 ^ tokenDecimals`. |

### Rounding / sign — `t => t` (chainable)

| Binding | Signature | Notes |
|---|---|---|
| `abs` | `t => t` | Absolute value. |
| `floor` | `t => t` | Round toward −∞. |
| `ceil` | `t => t` | Round toward +∞. |
| `round` | `t => t` | Round to integer using `decimal.js`'s default mode (`ROUND_HALF_UP`): `2.5 → 3`, `2.4 → 2`. |
| `toSignificantDigits` | `(t, int) => t` | Keep `n` significant digits: `123.456` at `2` → `120`. |
| `toDecimalPlaces` | `(t, int, int) => t` | `(value, dp, roundingMode)`. With `Decimal.roundDown`, `1.2399` at `2` → `1.23` (truncates). |

### Comparison / predicates — `=> int` / `=> bool`

| Binding | Signature | Notes |
|---|---|---|
| `cmp` | `(t, t) => int` | Returns `-1 | 0 | 1` (`comparedTo`). |
| `eq` | `(t, t) => bool` | `1.0` eq `1` → `true`. |
| `gt` | `(t, t) => bool` | strictly greater. |
| `gte` | `(t, t) => bool` | greater-or-equal. |
| `lt` | `(t, t) => bool` | strictly less. |
| `lte` | `(t, t) => bool` | less-or-equal. |
| `isZero` | `t => bool` | |
| `isNeg` | `t => bool` | `isNegative`. |

### Terminal accessors — `=> string` / `=> float`

| Binding | Signature | Notes |
|---|---|---|
| `toFixed` | `(t, int) => string` | Exactly `dp` decimal places — `1.5` at `3` → `"1.500"`. Use for display. |
| `toString` | `t => string` | Canonical string — `42` → `"42"`, `2.5` → `"2.5"`. |
| `toNumber` | `t => float` | JS number — lossy; never round-trip exact math through it. |

### Exported constants

| Name | Value | Notes |
|---|---|---|
| `roundDown` | `1` | `decimal.js` `ROUND_DOWN` — truncate toward zero. Pass to `toDecimalPlaces`. |

## Escape hatches

- **Other instance methods** (`sqrt`, `mod`, `log`, `exp`, …): add an ad-hoc
  `@send external` next to the existing ones, mirroring their shape — e.g.
  `@send external sqrt: t => t = "sqrt"`.
- **Other rounding modes**: `toDecimalPlaces` takes a raw `int`. The `decimal.js` constants
  are `0`=`ROUND_UP`, `1`=`ROUND_DOWN`, `2`=`ROUND_CEIL`, `3`=`ROUND_FLOOR`,
  `4`=`ROUND_HALF_UP`, … — pass the integer directly, or bind a named `let` like `roundDown`.
- **`bigint` conversion** (no direct binding): go through a string —
  `%raw("(decimal) => BigInt(decimal.toFixed(0))")`, as `program/Scaling.res` does.
