# Program Module Reference

On-chain Solana program interaction for the Lightcone protocol. This module contains all program-specific types, constants, utilities, and instruction builders.

## Architecture

On-chain operations are accessed through `LightconeClient`'s domain sub-clients:

```typescript
import { LightconeClient } from "@lightconexyz/lightcone-sdk";

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
| `client.positions()` | Position management | redeemWinnings, withdrawConditionalFromPosition, withdrawFromPosition compatibility wrapper, initPositionTokens, extendPositionTokens, depositToGlobal, globalToMarketDeposit, closePositionAlt, closePositionTokenAccounts, getOnchain |
| `client.orderbooks()` | Orderbook data | closeOrderbookAlt, closeOrderbook, getOnchain |
| `client.rpc()` | RPC utilities | getExchange, getGlobalDepositToken, getLatestBlockhash |

### Transaction builders return `TransactionInstruction`

All `*Ix()` methods are synchronous and return a `TransactionInstruction`. The caller composes transactions:

```typescript
import { Transaction } from "@solana/web3.js";
import { buildInitializeIx } from "@lightconexyz/lightcone-sdk";

const ix = buildInitializeIx({ authority });
const tx = new Transaction().add(ix);
tx.feePayer = authority;
tx.recentBlockhash = (await client.rpc().getLatestBlockhash()).blockhash;
```

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

#### GlobalDepositToken

| Field | Type | Description |
|-------|------|-------------|
| `discriminator` | Buffer | 8-byte discriminator |
| `mint` | PublicKey | Whitelisted deposit mint |
| `bump` | number | PDA bump seed |
| `index` | number | Deposit token ordering index |
| `active` | boolean | Backend-visible status flag |

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

- `Exchange` and `Market` accounts are 216 bytes.
- `GlobalDepositToken` accounts are 47 bytes, with `bump` at offset 40, `index` at 41..43, and `active` at offset 43.
- `setAuthority`, `setManager`, and `setOperator` now propose role transfers. The corresponding `acceptAuthority`, `acceptManager`, or `acceptOperator` instruction performs the effective role change.
- `matchOrdersMulti` and `depositAndSwap` include the fee receiver and associated token program in their fixed account lists.
- `setFeeReceiverWithAtas` can append quote mint / fee receiver ATA pairs for idempotent ATA creation.
- `refreshOrderbookAlt` appends the current fee receiver quote ATA when missing, but does not fully reshape older orderbook ALTs.
- Instruction discriminators are current through `SetDepositTokenStatus = 38`.

### Limits

```typescript
import { MAX_OUTCOMES, MIN_OUTCOMES, MAX_MAKERS } from "@lightconexyz/lightcone-sdk";

MAX_OUTCOMES  // 6
MIN_OUTCOMES  // 2
MAX_MAKERS    // 5
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

  // Get conditional mints
  const mints = client.markets().getConditionalMints(marketPda, usdcMint, 2);
  const [yesMint, noMint] = mints;

  // Create and sign orders via orders sub-client
  const orders = client.orders();
  const nonce = await orders.currentNonce(maker.publicKey);

  const signedOrder = orders.createSignedBidOrder(
    {
      nonce,
      maker: maker.publicKey,
      market: marketPda,
      baseMint: yesMint,
      quoteMint: noMint,
      amountIn: 500_000n,
      amountOut: 500_000n,
    },
    maker
  );

  // Build match instruction via the raw encoder (operator-only on-chain)
  const matchIx = buildMatchOrdersMultiIx({
    operator: operatorPubkey,
    market: marketPda,
    baseMint: yesMint,
    quoteMint: noMint,
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

```typescript
import {
  // Instruction builders
  buildInitializeIx, buildCreateMarketIx, buildMatchOrdersMultiIx,
  buildRefreshOrderbookAltIx, buildAcceptAuthorityIx, buildSetOracleIx,
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
