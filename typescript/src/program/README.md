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
| `client.admin()` | Admin operations | initialize, createMarket, addDepositMint, activateMarket, settleMarket, setPaused, setOperator, setAuthority, whitelistDepositToken, createOrderbook, matchOrdersMulti, depositAndSwap |
| `client.orders()` | Order management | cancelOrder, incrementNonce, createBidOrder, createAskOrder, signOrder, getStatus, getNonce |
| `client.markets()` | Market queries | mintCompleteSet, mergeCompleteSet, deriveConditionId, getConditionalMints, getOnchain |
| `client.positions()` | Position management | redeemWinnings, withdrawFromPosition, initPositionTokens, extendPositionTokens, depositToGlobal, globalToMarketDeposit, getOnchain |
| `client.orderbooks()` | Orderbook data | getOnchain |
| `client.rpc()` | RPC utilities | getExchange, getGlobalDepositToken, getLatestBlockhash |

### Transaction builders return `TransactionInstruction`

All `*Ix()` methods are synchronous and return a `TransactionInstruction`. The caller composes transactions:

```typescript
import { Transaction } from "@solana/web3.js";

const ix = client.admin().initializeIx(authority);
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
  Market,
  Position,
  OrderStatus,
  UserNonce,
} from "@lightconexyz/lightcone-sdk";
```

#### Exchange

| Field | Type | Description |
|-------|------|-------------|
| `discriminator` | Buffer | 8-byte discriminator |
| `authority` | PublicKey | Admin authority |
| `operator` | PublicKey | Order matching operator |
| `marketCount` | bigint | Number of markets created |
| `paused` | boolean | Trading paused |
| `bump` | number | PDA bump seed |

#### Market

| Field | Type | Description |
|-------|------|-------------|
| `discriminator` | Buffer | 8-byte discriminator |
| `marketId` | bigint | Sequential market ID |
| `numOutcomes` | number | Number of outcomes (2-6) |
| `status` | MarketStatus | Current status |
| `winningOutcome` | number | Winner (if settled) |
| `hasWinningOutcome` | boolean | Is settled |
| `bump` | number | PDA bump seed |
| `oracle` | PublicKey | Oracle authority |
| `questionId` | Buffer | Question identifier (32 bytes) |
| `conditionId` | Buffer | Computed condition ID (32 bytes) |

#### SignedOrder (225 bytes)

| Field | Type | Description |
|-------|------|-------------|
| `nonce` | number | Order nonce |
| `maker` | PublicKey | Maker public key |
| `market` | PublicKey | Market address |
| `baseMint` | PublicKey | Base token mint |
| `quoteMint` | PublicKey | Quote token mint |
| `side` | OrderSide | BID or ASK |
| `amountIn` | bigint | Amount maker gives |
| `amountOut` | bigint | Amount maker receives |
| `expiration` | bigint | Expiration timestamp (0 = no expiration) |
| `signature` | Buffer | Ed25519 signature (64 bytes) |

#### Order (29 bytes)

Compact order payload without `maker`, `market`, `baseMint`, or `quoteMint`.

---

## Constants

### Program IDs

The Lightcone program ID is derived from `LightconeEnv` and accessed via `programId(env)` or `client.programId`. `PROGRAM_ID` is re-exported as a convenience default (production). When targeting staging or local, always pass `programId` explicitly.

```typescript
import { PROGRAM_ID, TOKEN_PROGRAM_ID, TOKEN_2022_PROGRAM_ID } from "@lightconexyz/lightcone-sdk";
```

| Constant | Value |
|----------|-------|
| `PROGRAM_ID` | Production default, derived from `LightconeEnv.Prod` |
| `TOKEN_PROGRAM_ID` | `TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA` |
| `TOKEN_2022_PROGRAM_ID` | `TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb` |

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

  // Build match instruction via admin sub-client
  const matchIx = client.admin().matchOrdersMultiIx({
    operator: operatorPubkey,
    market: marketPda,
    baseMint: yesMint,
    quoteMint: noMint,
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
  // PDA functions
  getExchangePda, getMarketPda, getOrderStatusPda,
  // Account deserialization
  deserializeExchange, deserializeMarket,
  // Order utilities
  hashOrder, signOrder, createBidOrder, createAskOrder,
  // Constants
  PROGRAM_ID, INSTRUCTION, DISCRIMINATOR,
} from "@lightconexyz/lightcone-sdk";
```
