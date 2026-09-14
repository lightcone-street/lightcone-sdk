# Program Module Reference

On-chain Solana program interaction for the Lightcone protocol. This module contains all program-specific types, constants, utilities, and instruction builders.

## Architecture

On-chain operations are accessed through `LightconeClient`'s domain sub-clients:

```typescript
import { LightconeClient, OrderBuilder } from "@lightconexyz/lightcone-sdk";

// HTTP-only client (no Connection required for instruction building)
const client = LightconeClient.builder().build();

// With Solana RPC for on-chain reads
const client = LightconeClient.builder()
  .rpcUrl("https://api.devnet.solana.com")
  .build();

// With custom program ID
const client = LightconeClient.builder()
  .rpcUrl("https://api.devnet.solana.com")
  .programId(customProgramId)
  .build();
```

### Sub-client Organization

| Sub-client | Access | On-chain capabilities |
|------------|--------|----------------------|
| `client.orders()` | Order management | cancelOrder, incrementNonce, closeOrderStatus, createBidOrder, createAskOrder, signOrder, getStatus, getNonce |
| `client.markets()` | Market queries | mintCompleteSet, mergeCompleteSet, deriveConditionId, getConditionalMints, getOnchain |
| `client.positions()` | Position management | redeemWinnings, withdrawConditionalFromPosition, withdrawFromPosition compatibility wrapper, initPositionTokens, depositToGlobal, globalToMarketDeposit, closePositionTokenAccounts, getOnchain |
| `client.orderbooks()` | Orderbook data | closeOrderbook, getOnchain |
| `client.rpc()` | RPC utilities | getExchange, getGlobalDepositToken, getLatestBlockhash |

### Instruction and v1 transaction builders

All `*Ix()` methods return web3.js `TransactionInstruction`. The `*Tx()` helpers
compile immutable `V1Transaction` values with an explicit blockhash/resource
context. Direct helpers take the context before their optional program ID.

```typescript
import { V1Transaction, buildInitializeIx } from "@lightconexyz/lightcone-sdk";

const context = await client.transactionContext();
const ix = buildInitializeIx({ authority });
const tx = V1Transaction.compile([ix], authority, context);
const signed = tx.sign([authorityKeypair]);
const signature = await client.rpc().submitSignedTransaction(signed);
await client.rpc().confirmSignature(signature, context.lastValidBlockHeight);
```

See [Solana v1 transactions](../../README.md#solana-v1-transactions) for explicit
resource units, wallet requirements, and one-shot submission behavior.

---

## Types

### MarketStatus

```typescript
import { MarketStatus } from "@lightconexyz/lightcone-sdk";

MarketStatus.Pending    // 0 - Not yet active
MarketStatus.Active     // 1 - Trading enabled
MarketStatus.Resolved   // 2 - Market settled
MarketStatus.Cancelled  // 3 - Market cancelled
```

### OrderSide

```typescript
import { OrderSide } from "@lightconexyz/lightcone-sdk";

OrderSide.BID  // 0 - Buyer gives quote, receives base
OrderSide.ASK  // 1 - Seller gives base, receives quote
```

### Account Types

```typescript
import type {
  Exchange,
  GlobalDepositToken,
  Market,
  Position,
  OrderStatus,
  UserNonce,
  PendingRoleKind,
} from "@lightconexyz/lightcone-sdk";
```

#### Exchange

| Field | Type | Description |
|-------|------|-------------|
| `discriminator` | Buffer | 8-byte discriminator |
| `authority` | PublicKey | Admin authority |
| `operator` | PublicKey | Order matching operator |
| `manager` | PublicKey | Market and orderbook setup manager |
| `marketCount` | bigint | Number of markets created |
| `paused` | boolean | Trading paused |
| `bump` | number | PDA bump seed |
| `depositTokenCount` | number | Number of whitelisted deposit tokens |
| `feeReceiver` | PublicKey | Current protocol fee receiver |
| `pendingRole` | PublicKey | Pending privileged-role recipient |
| `pendingRoleKind` | PendingRoleKind | Pending role kind: none, authority, manager, or operator |

#### Market

| Field | Type | Description |
|-------|------|-------------|
| `discriminator` | Buffer | 8-byte discriminator |
| `marketId` | bigint | Sequential market ID |
| `numOutcomes` | number | Number of outcomes (2-6) |
| `status` | MarketStatus | Current status |
| `bump` | number | PDA bump seed |
| `makerFeeBps` | number | Maker fee in basis points |
| `takerFeeBps` | number | Taker fee in basis points |
| `oracle` | PublicKey | Oracle authority |
| `questionId` | Buffer | Question identifier (32 bytes) |
| `conditionId` | Buffer | Computed condition ID (32 bytes) |
| `payoutNumerators` | [number, number, number, number, number, number] | Resolution vector; first `numOutcomes` entries are meaningful |
| `payoutDenominator` | number | Sum of meaningful payout numerators |
| `depositMintCount` | number | Deposit mints registered through `addDepositMint` (byte 148; capped at `MAX_DEPOSIT_MINTS_PER_MARKET`) |

#### GlobalDepositToken

| Field | Type | Description |
|-------|------|-------------|
| `discriminator` | Buffer | 8-byte discriminator |
| `mint` | PublicKey | Whitelisted deposit mint |
| `bump` | number | PDA bump seed |
| `index` | number | Deposit token ordering index |
| `active` | boolean | Whether the collateral may back an executed trade; inactivity leaves deposit, preparation, split, merge, and exit rules unchanged |

#### SignedOrder (233 bytes)

| Field | Type | Description |
|-------|------|-------------|
| `nonce` | number | Order nonce |
| `salt` | bigint | Random salt for order uniqueness |
| `maker` | PublicKey | Maker public key |
| `market` | PublicKey | Market address |
| `baseMint` | PublicKey | Base token mint |
| `quoteMint` | PublicKey | Quote token mint |
| `side` | OrderSide | BID or ASK |
| `amountIn` | bigint | Amount maker gives |
| `amountOut` | bigint | Amount maker receives |
| `expiration` | bigint | Expiration timestamp (0 = no expiration) |
| `signature` | Buffer | Ed25519 signature (64 bytes) |

#### Order (37 bytes)

Compact order payload without `maker`, `market`, `baseMint`, or `quoteMint`.

---

## Constants

### Program IDs

The Lightcone program ID is derived from `LightconeEnv` and accessed via `programId(env)` or `client.programId`. `PROGRAM_ID` is re-exported as a convenience default (production). When targeting staging or local, always pass `programId` explicitly.

```typescript
import { PROGRAM_ID, TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID } from "@lightconexyz/lightcone-sdk";
```

| Constant | Value |
|----------|-------|
| `PROGRAM_ID` | Production default, derived from `LightconeEnv.Prod` |
| `TOKEN_PROGRAM_ID` | `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA` |
| `ASSOCIATED_TOKEN_PROGRAM_ID` | SPL Associated Token Account program |
| `INITIALIZE_AUTHORITY` | Pubkey allowed by the program to initialize the exchange |

### Current Program Alignment Notes

- `Orderbook` requires exactly 176 bytes and exposes `depositMintA`, `depositMintB`, and the shared `outcomeIndex`. Both conditional mints belong to that market outcome and have distinct collateral.
- `createOrderbook` accepts one `outcomeIndex`. It sorts supplied mints with their collateral identities and preserves the requested base orientation.
- `matchOrdersMulti` and `depositAndSwap` require `baseDepositMint` and `quoteDepositMint`. Fixed accounts 4 and 5 are the canonical collateral GDTs. Both live activity flags must permit trading, including with zero funding.
- Matching accepts 1..11 makers. Masks are unsigned 16-bit little-endian values. Bits 0..10 select makers, bit 15 selects the taker, and bits 11..14 are reserved. For nine makers, `0x8100` selects maker index 8 and the taker.
- A selected funding mint must back the participant's signed give side: quote collateral for BUY, base collateral for SELL. Complete-set funding still covers every market outcome.
- `initPositionTokens` prepares new, partial, repeated, and additional collateral groups with any signing payer. Supply 1..8 distinct mints in increasing global registration-index order. No slot is required. The synchronous builder preserves the supplied order.
- `depositToGlobal` emits the exact eight-account deposit interface. `closeOrderbook` directly closes a resolved book with four business accounts. Both counts exclude the two event trailers.
- Instruction values 21, 23, 26, and 34 are unassigned. ALT instructions, helpers, options, and exports have been removed. Use `initPositionTokens` for additional preparation groups.
- `GlobalDepositToken` requires exactly 47 bytes and an activity byte of 0 or 1. Inactivity gates trading, while deposits, preparation, splits, merges, and exits retain their existing rules.
- `Exchange` and `Market` remain 216 bytes. Role transfers, fee updates, order signing, and backend wire fields retain their existing contracts.

Transaction helpers return only validated Solana v1 envelopes, with 4,096-byte signed size and 64-address limits. Eleven-maker instructions retain the upgraded program ABI. Legacy/v0 imports, ComputeBudget instructions, and address lookup tables are rejected.

### Event Transport Trailer

Every `build*Ix` appends the event-authority PDA and executable program account
as read-only, non-signer accounts, in that order. These are always the last two
accounts; callers must not append another trailer.

See the [program integration contract](https://github.com/lightcone-street/docs/blob/0886e2356c69e8d59b2dca953331f63d7ecd9619/api-reference/program-integration.mdx) for invocation rules and the governance CPI allowlist. The [program source at `db552338`](https://github.com/lightcone-street/lightcone-pinnochio/tree/db552338404263b17b6af5e39a99477ee16a1934/src) defines the current binary interfaces, preparation behavior, limits, and errors.

The program emits authenticated event schema 2. This is separate from the outer Solana transaction version.
The SDK neither builds nor decodes event batches. `INSTRUCTION.EVENT_BATCH = 255` is reserved. Builders do not add a compute-budget instruction. Callers must include
the program's final self-CPI when estimating transaction compute.

`buildInitPositionTokensIx` throws
`ProgramSdkError` with variant `InvalidPubkey` for zero or off-curve beneficiaries,
`MissingField` for empty mint lists, and `TooManyDepositMints` for lists exceeding
`MAX_DEPOSIT_MINTS_PER_IX`. Market creation and oracle rotation builders throw
`InvalidOracle` for zero or off-curve oracle keys.

```typescript
import { program } from "@lightconexyz/lightcone-sdk";

const [eventAuthority, bump] = program.getEventAuthorityPda(program.PROGRAM_ID);
```

### Limits

```typescript
import { program } from "@lightconexyz/lightcone-sdk";

program.MAX_OUTCOMES                  // 6
program.MIN_OUTCOMES                  // 2
program.MAX_MAKERS                    // 11 (parser ceiling)
program.MAX_DEPOSIT_MINTS_PER_MARKET   // 8
program.MAX_DEPOSIT_MINTS_PER_IX       // 8
```

---

## Complete Example

```typescript
import { Keypair } from "@solana/web3.js";
import { LightconeClient } from "@lightconexyz/lightcone-sdk";

async function main() {
  const client = LightconeClient.builder()
    .rpcUrl("https://api.devnet.solana.com")
    .build();

  // Fetch exchange state
  const exchange = await client.rpc().getExchange();
  console.log(`Markets: ${exchange.marketCount}`);

  // Get market PDA
  const marketPda = client.markets().pda(0n);

  // Trade the same outcome backed by two different collateral assets.
  const [baseMint] = client.markets().getConditionalMints(marketPda, btcMint, 2);
  const [quoteMint] = client.markets().getConditionalMints(marketPda, usdcMint, 2);

  // Construct and sign an exact order with immutable rules
  const orders = client.orders();
  const nonce = await orders.currentNonce(maker.publicKey);
  const rules = await client.orderbooks().decimals(orderbookId);

  const signedOrder = new OrderBuilder()
    .nonce(nonce)
    .maker(maker.publicKey)
    .market(marketPda)
    .baseMint(baseMint)
    .quoteMint(quoteMint)
    .bid()
    .price("0.5", "1", rules)
    .buildAndSign(maker, rules);

  // Build match instruction via the raw encoder (operator-only on-chain)
  const matchIx = buildMatchOrdersMultiIx({
    operator: operatorPubkey,
    market: marketPda,
    baseMint,
    quoteMint,
    baseDepositMint: btcMint,
    quoteDepositMint: usdcMint,
    feeReceiver: exchange.feeReceiver,
    takerOrder: signedTakerOrder,
    makerOrders: [signedOrder],
    makerFillAmounts: [500_000n],
    takerFillAmounts: [500_000n],
    fullFillBitmask: 0,
  });
}
```

## Low-Level Building Blocks

The `program` module also exports all building blocks directly for advanced usage:

Raw signing helpers preserve the wire/signature contract and require fetched
`OrderbookRules`; they validate the executable ratio, size quantum, and signed
field ranges before signing. `LimitOrderEnvelope.submit()` fetches and caches
those rules automatically.

```typescript
import {
  // Instruction builders
  buildInitializeIx, buildCreateMarketIx, buildMatchOrdersMultiIx,
  buildAcceptAuthorityIx, buildSetOracleIx,
  // PDA functions
  getExchangePda, getMarketPda, getOrderStatusPda,
  // Account deserialization
  deserializeExchange, deserializeMarket,
  // Resolution helpers
  winnerTakesAllPayoutNumerators, scalarToPayoutNumerators,
  // Order utilities
  hashOrder, signOrder, createBidOrder, createAskOrder,
  // Constants
  PROGRAM_ID, INSTRUCTION, DISCRIMINATOR,
} from "@lightconexyz/lightcone-sdk";
```

Settle instructions now submit payout numerators directly. Binary markets can use
`winnerTakesAllPayoutNumerators(winningOutcome, numOutcomes)`, while scalar
markets should use integer fixed-point `scalarToPayoutNumerators(...)` and pass
the returned vector as `SettleMarketParams.payoutNumerators`.
