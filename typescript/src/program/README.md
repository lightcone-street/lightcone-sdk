# Program Module Reference

On-chain Solana program interaction for the Lightcone protocol. This module contains all program-specific types, constants, utilities, and the client for building transactions.

## Contents

- [Types](#types) - Enums, account types, order types, parameter types
- [Constants](#constants) - Program IDs, seeds, discriminators, sizes
- [Utilities](#utilities) - Byte operations, hashing, ATA derivation, validation
- [Client](#client) - Transaction building and account fetching
- [PDA Functions](#pda-functions) - Program Derived Address derivation
- [Order Operations](#order-creation-and-signing) - Order creation, signing, serialization
- [Ed25519 Verification](#ed25519-verification) - Signature verification strategies

---

## Types

### MarketStatus

```typescript
import { MarketStatus } from "@lightcone/sdk";

MarketStatus.Pending    // 0 - Not yet active
MarketStatus.Active     // 1 - Trading enabled
MarketStatus.Resolved   // 2 - Market settled
MarketStatus.Cancelled  // 3 - Market cancelled
```

### OrderSide

```typescript
import { OrderSide } from "@lightcone/sdk";

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
} from "@lightcone/sdk";
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

#### Position

| Field | Type | Description |
|-------|------|-------------|
| `discriminator` | Buffer | 8-byte discriminator |
| `owner` | PublicKey | Position owner |
| `market` | PublicKey | Market address |
| `bump` | number | PDA bump seed |

#### OrderStatus

| Field | Type | Description |
|-------|------|-------------|
| `discriminator` | Buffer | 8-byte discriminator |
| `remaining` | bigint | Remaining order amount |
| `isCancelled` | boolean | Cancelled flag |

#### UserNonce

| Field | Type | Description |
|-------|------|-------------|
| `discriminator` | Buffer | 8-byte discriminator |
| `nonce` | bigint | Current nonce value |

### Order Types

```typescript
import type { FullOrder, CompactOrder } from "@lightcone/sdk";
```

#### FullOrder (225 bytes)

| Field | Type | Description |
|-------|------|-------------|
| `nonce` | bigint | Order nonce |
| `maker` | PublicKey | Maker public key |
| `market` | PublicKey | Market address |
| `baseMint` | PublicKey | Base token mint |
| `quoteMint` | PublicKey | Quote token mint |
| `side` | OrderSide | BID or ASK |
| `makerAmount` | bigint | Amount maker gives |
| `takerAmount` | bigint | Amount maker receives |
| `expiration` | bigint | Expiration timestamp (0 = no expiration) |
| `signature` | Buffer | Ed25519 signature (64 bytes) |

#### CompactOrder (65 bytes)

Same as FullOrder but without `market`, `baseMint`, `quoteMint` (derived from instruction context).

| Field | Type | Description |
|-------|------|-------------|
| `nonce` | bigint | Order nonce |
| `maker` | PublicKey | Maker public key |
| `side` | OrderSide | BID or ASK |
| `makerAmount` | bigint | Amount maker gives |
| `takerAmount` | bigint | Amount maker receives |
| `expiration` | bigint | Expiration timestamp |

### Parameter Types

```typescript
import type {
  InitializeParams,
  CreateMarketParams,
  AddDepositMintParams,
  OutcomeMetadata,
  MintCompleteSetParams,
  MergeCompleteSetParams,
  CancelOrderParams,
  IncrementNonceParams,
  SettleMarketParams,
  RedeemWinningsParams,
  SetPausedParams,
  SetOperatorParams,
  WithdrawFromPositionParams,
  ActivateMarketParams,
  MatchOrdersMultiParams,
  BidOrderParams,
  AskOrderParams,
} from "@lightcone/sdk";
```

### Build Result Types

```typescript
import type {
  BuildResult,
  InitializeAccounts,
  CreateMarketAccounts,
  AddDepositMintAccounts,
  MintCompleteSetAccounts,
  MergeCompleteSetAccounts,
  CancelOrderAccounts,
  IncrementNonceAccounts,
  SettleMarketAccounts,
  RedeemWinningsAccounts,
  ActivateMarketAccounts,
  MatchOrdersMultiAccounts,
} from "@lightcone/sdk";
```

#### BuildResult<T>

| Field | Type | Description |
|-------|------|-------------|
| `transaction` | Transaction | Unsigned transaction ready for signing |
| `accounts` | T | Key accounts involved |
| `serialize` | () => string | Serialize to base64 |

---

## Constants

### Program IDs

```typescript
import {
  PROGRAM_ID,
  TOKEN_PROGRAM_ID,
  TOKEN_2022_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  SYSTEM_PROGRAM_ID,
  RENT_SYSVAR_ID,
  INSTRUCTIONS_SYSVAR_ID,
  ED25519_PROGRAM_ID,
} from "@lightcone/sdk";
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

```typescript
import { SEEDS } from "@lightcone/sdk";

SEEDS.CENTRAL_STATE              // "central_state"
SEEDS.MARKET                     // "market"
SEEDS.MARKET_DEPOSIT_TOKEN_ACCOUNT // "market_deposit_token_account"
SEEDS.MARKET_MINT_AUTHORITY      // "market_mint_authority"
SEEDS.CONDITIONAL_MINT           // "conditional_mint"
SEEDS.ORDER_STATUS               // "order_status"
SEEDS.USER_NONCE                 // "user_nonce"
SEEDS.POSITION                   // "position"
```

### Account Discriminators

```typescript
import { DISCRIMINATOR } from "@lightcone/sdk";

DISCRIMINATOR.EXCHANGE      // Buffer.from("exchange")
DISCRIMINATOR.MARKET        // Buffer.from("market\0\0")
DISCRIMINATOR.ORDER_STATUS  // Buffer.from("ordstat\0")
DISCRIMINATOR.USER_NONCE    // Buffer.from("usrnonce")
DISCRIMINATOR.POSITION      // Buffer.from("position")
```

### Account Sizes

```typescript
import { ACCOUNT_SIZE } from "@lightcone/sdk";

ACCOUNT_SIZE.EXCHANGE      // 88
ACCOUNT_SIZE.MARKET        // 120
ACCOUNT_SIZE.ORDER_STATUS  // 24
ACCOUNT_SIZE.USER_NONCE    // 16
ACCOUNT_SIZE.POSITION      // 80
```

### Order Sizes

```typescript
import { ORDER_SIZE } from "@lightcone/sdk";

ORDER_SIZE.FULL       // 225
ORDER_SIZE.COMPACT    // 65
ORDER_SIZE.SIGNATURE  // 64
```

### Instruction Discriminators

```typescript
import { INSTRUCTION } from "@lightcone/sdk";

INSTRUCTION.INITIALIZE           // 0
INSTRUCTION.CREATE_MARKET        // 1
INSTRUCTION.ADD_DEPOSIT_MINT     // 2
INSTRUCTION.MINT_COMPLETE_SET    // 3
INSTRUCTION.MERGE_COMPLETE_SET   // 4
INSTRUCTION.CANCEL_ORDER         // 5
INSTRUCTION.INCREMENT_NONCE      // 6
INSTRUCTION.SETTLE_MARKET        // 7
INSTRUCTION.REDEEM_WINNINGS      // 8
INSTRUCTION.SET_PAUSED           // 9
INSTRUCTION.SET_OPERATOR         // 10
INSTRUCTION.WITHDRAW_FROM_POSITION // 11
INSTRUCTION.ACTIVATE_MARKET      // 12
INSTRUCTION.MATCH_ORDERS_MULTI   // 13
```

### Limits

```typescript
import { MAX_OUTCOMES, MIN_OUTCOMES, MAX_MAKERS } from "@lightcone/sdk";

MAX_OUTCOMES  // 6
MIN_OUTCOMES  // 2
MAX_MAKERS    // 5
```

---

## Utilities

### Byte Utilities

```typescript
import {
  toLeBytes,
  fromLeBytes,
  toBeBytes,
  fromBeBytes,
  toU8,
  toU64Le,
  toI64Le,
  fromI64Le,
} from "@lightcone/sdk";

// Little-endian conversion
const bytes = toLeBytes(1000n, 8);  // bigint to 8-byte LE buffer
const value = fromLeBytes(bytes);   // buffer to bigint

// Big-endian conversion
const beBytes = toBeBytes(1000n, 8);
const beValue = fromBeBytes(beBytes);

// Type-specific conversions
const u8 = toU8(42);           // number to 1-byte buffer
const u64 = toU64Le(1000000n); // bigint to 8-byte LE buffer
const i64 = toI64Le(-1000n);   // signed bigint to 8-byte LE buffer
const signed = fromI64Le(i64); // 8-byte LE buffer to signed bigint
```

### Hashing

```typescript
import { keccak256 } from "@lightcone/sdk";

// Hash arbitrary data
const hash = keccak256(Buffer.from("data"));  // Returns 32-byte Buffer
```

### Condition ID

```typescript
import { deriveConditionId } from "@lightcone/sdk";
import { PublicKey } from "@solana/web3.js";

const conditionId = deriveConditionId(
  new PublicKey("oracle_pubkey"),  // Oracle
  Buffer.alloc(32),                // Question ID (32 bytes)
  2,                               // Number of outcomes
);
// Returns 32-byte Buffer: keccak256(oracle || questionId || numOutcomes)
```

### Associated Token Addresses

```typescript
import {
  getAssociatedTokenAddress,
  getConditionalTokenAta,
  getDepositTokenAta,
} from "@lightcone/sdk";
import { PublicKey } from "@solana/web3.js";

const owner = new PublicKey("owner_pubkey");
const mint = new PublicKey("mint_pubkey");

// SPL Token ATA
const ata = getAssociatedTokenAddress(mint, owner);

// Token-2022 ATA (for conditional tokens)
const ata2022 = getAssociatedTokenAddress(mint, owner, true);

// Convenience methods
const conditionalAta = getConditionalTokenAta(mint, owner);  // Token-2022
const depositAta = getDepositTokenAta(mint, owner);          // SPL Token
```

### String Serialization

```typescript
import { serializeString, deserializeString } from "@lightcone/sdk";

// Serialize string with u16 length prefix
const serialized = serializeString("Hello");  // [length (2 bytes)] + [utf-8 bytes]

// Deserialize
const [str, bytesConsumed] = deserializeString(serialized, 0);
// str = "Hello", bytesConsumed = 7
```

### Validation

```typescript
import {
  validateOutcomes,
  validateOutcomeIndex,
  validate32Bytes,
} from "@lightcone/sdk";

// Validate outcome count (2-6)
validateOutcomes(3);  // OK
validateOutcomes(7);  // throws Error

// Validate outcome index
validateOutcomeIndex(1, 3);  // OK (index 1 valid for 3 outcomes)
validateOutcomeIndex(3, 3);  // throws Error (index must be 0-2)

// Validate 32-byte buffer
validate32Bytes(Buffer.alloc(32), "questionId");  // OK
validate32Bytes(Buffer.alloc(16), "questionId");  // throws Error
```

---

## Client

### LightconePinocchioClient

```typescript
import { Connection } from "@solana/web3.js";
import { LightconePinocchioClient, PROGRAM_ID } from "@lightcone/sdk";

const connection = new Connection("https://api.devnet.solana.com");
const client = new LightconePinocchioClient(connection);

// Or with custom program ID
const client = new LightconePinocchioClient(connection, customProgramId);

// Access PDA functions
const [exchangePda] = client.pda.getExchangePda(client.programId);
```

## Account Fetchers

### getExchange

Fetch the singleton Exchange account.

```typescript
const exchange = await client.getExchange();
console.log(`Markets: ${exchange.marketCount}`);
console.log(`Paused: ${exchange.paused}`);
```

### getMarket

Fetch a Market by ID or pubkey.

```typescript
// By ID
const market = await client.getMarket(0n);
console.log(`Status: ${market.status}`);
console.log(`Outcomes: ${market.numOutcomes}`);

// By pubkey
const market = await client.getMarketByPubkey(marketPubkey);
```

### getPosition

Fetch a user's Position in a market.

```typescript
const position = await client.getPosition(userPubkey, marketPubkey);
if (position) {
  console.log(`Owner: ${position.owner.toBase58()}`);
}
```

### getOrderStatus

Fetch an OrderStatus by order hash.

```typescript
const orderStatus = await client.getOrderStatus(orderHash);
if (orderStatus) {
  console.log(`Remaining: ${orderStatus.remaining}`);
  console.log(`Cancelled: ${orderStatus.isCancelled}`);
}
```

### getUserNonce

Fetch a user's nonce.

```typescript
const nonce = await client.getUserNonce(userPubkey);
console.log(`Nonce: ${nonce}`);

// Get next nonce for order creation
const nextNonce = await client.getNextNonce(userPubkey);
```

### getNextMarketId

Get the next market ID that will be assigned.

```typescript
const nextId = await client.getNextMarketId();
```

## Transaction Builders

All transaction builders return a `BuildResult<T>`:

```typescript
interface BuildResult<T> {
  transaction: Transaction;  // Unsigned transaction
  accounts: T;               // Key accounts
  serialize: () => string;   // Serialize to base64
}
```

### initialize

Create the Exchange account (one-time protocol initialization).

```typescript
const result = await client.initialize({
  authority: adminPubkey,
});
// result.accounts: { exchange: PublicKey }
```

### createMarket

Create a new market.

```typescript
const result = await client.createMarket({
  authority: adminPubkey,
  numOutcomes: 2,
  oracle: oraclePubkey,
  questionId: Buffer.alloc(32), // 32-byte question ID
});
// result.accounts: { exchange: PublicKey, market: PublicKey }
```

### addDepositMint

Add a deposit mint to a market.

```typescript
const result = await client.addDepositMint(
  {
    authority: adminPubkey,
    marketId: 0n,
    depositMint: usdcMint,
    outcomeMetadata: [
      { name: "Yes Token", symbol: "YES", uri: "..." },
      { name: "No Token", symbol: "NO", uri: "..." },
    ],
  },
  2 // numOutcomes
);
// result.accounts: { market, vault, mintAuthority, conditionalMints[] }
```

### mintCompleteSet

Mint a complete set of outcome tokens.

```typescript
const result = await client.mintCompleteSet(
  {
    user: userPubkey,
    market: marketPubkey,
    depositMint: usdcMint,
    amount: 1_000_000n, // Raw units
  },
  2 // numOutcomes
);
// result.accounts: { position, vault, conditionalMints[] }
```

### mergeCompleteSet

Merge outcome tokens back into deposit tokens.

```typescript
const result = await client.mergeCompleteSet(
  {
    user: userPubkey,
    market: marketPubkey,
    depositMint: usdcMint,
    amount: 1_000_000n,
  },
  2 // numOutcomes
);
// result.accounts: { position, vault, conditionalMints[] }
```

### cancelOrder

Cancel an order.

```typescript
const result = await client.cancelOrder(makerPubkey, order);
// result.accounts: { orderStatus: PublicKey }
```

### incrementNonce

Increment user's nonce to invalidate all orders below it.

```typescript
const result = await client.incrementNonce(userPubkey);
// result.accounts: { userNonce: PublicKey }
```

### settleMarket

Settle a market (oracle only).

```typescript
const result = await client.settleMarket({
  oracle: oraclePubkey,
  marketId: 0n,
  winningOutcome: 0, // 0-indexed
});
// result.accounts: { exchange, market }
```

### redeemWinnings

Redeem winning outcome tokens.

```typescript
const result = await client.redeemWinnings(
  {
    user: userPubkey,
    market: marketPubkey,
    depositMint: usdcMint,
    amount: 1_000_000n,
  },
  0 // winningOutcome
);
// result.accounts: { position, vault, winningMint }
```

### setPaused

Pause or unpause the exchange.

```typescript
const result = await client.setPaused(authorityPubkey, true);
// result.accounts: { exchange: PublicKey }
```

### setOperator

Set the operator account.

```typescript
const result = await client.setOperator(authorityPubkey, newOperatorPubkey);
// result.accounts: { exchange: PublicKey }
```

### withdrawFromPosition

Withdraw tokens from a position to user's ATA.

```typescript
const result = await client.withdrawFromPosition(
  {
    user: userPubkey,
    market: marketPubkey,
    mint: tokenMint,
    amount: 1_000_000n,
  },
  true // isToken2022 (true for conditional tokens)
);
// result.accounts: { position: PublicKey }
```

### activateMarket

Activate a pending market.

```typescript
const result = await client.activateMarket({
  authority: adminPubkey,
  marketId: 0n,
});
// result.accounts: { exchange, market }
```

### matchOrdersMulti

Build match orders instruction (without Ed25519 verification).

```typescript
const result = await client.matchOrdersMulti({
  operator: operatorPubkey,
  market: marketPubkey,
  baseMint: yesMint,
  quoteMint: noMint,
  takerOrder: signedTakerOrder,
  makerOrders: [signedMakerOrder1, signedMakerOrder2],
  fillAmounts: [100_000n, 50_000n],
});
// result.accounts: { takerOrderStatus, takerPosition, makerOrderStatuses[], makerPositions[] }
```

### matchOrdersMultiWithVerify

Build complete match orders transaction with Ed25519 verification (recommended).

```typescript
const result = await client.matchOrdersMultiWithVerify({
  operator: operatorPubkey,
  market: marketPubkey,
  baseMint: yesMint,
  quoteMint: noMint,
  takerOrder: signedTakerOrder,
  makerOrders: [signedMakerOrder],
  fillAmounts: [100_000n],
});
// Transaction includes Ed25519 verify instructions for all orders
```

## Order Creation and Signing

### createBidOrder

Create a BID order (buyer wants base tokens, pays quote).

```typescript
import { Keypair } from "@solana/web3.js";

const keypair = Keypair.generate();
const nonce = await client.getNextNonce(keypair.publicKey);

const order = client.createBidOrder({
  nonce,
  maker: keypair.publicKey,
  market: marketPubkey,
  baseMint: yesMint,     // Token to buy
  quoteMint: noMint,     // Token to pay with
  makerAmount: 1_000_000n, // Quote tokens to give
  takerAmount: 500_000n,   // Base tokens to receive
  expiration: 0n,          // 0 = no expiration
});
```

### createAskOrder

Create an ASK order (seller offers base tokens, receives quote).

```typescript
const order = client.createAskOrder({
  nonce,
  maker: keypair.publicKey,
  market: marketPubkey,
  baseMint: yesMint,     // Token to sell
  quoteMint: noMint,     // Token to receive
  makerAmount: 500_000n,   // Base tokens to give
  takerAmount: 1_000_000n, // Quote tokens to receive
  expiration: 0n,
});
```

### signFullOrder

Sign an order with a Keypair.

```typescript
const signedOrder = client.signFullOrder(order, keypair);
// signedOrder.signature contains the 64-byte Ed25519 signature
```

### hashOrder

Hash an order (used for order status PDA derivation).

```typescript
const orderHash = client.hashOrder(signedOrder);
// 32-byte keccak256 hash
```

## PDA Functions

All PDA functions return `[PublicKey, number]` (address and bump).

```typescript
import {
  getExchangePda,
  getMarketPda,
  getVaultPda,
  getMintAuthorityPda,
  getConditionalMintPda,
  getAllConditionalMintPdas,
  getOrderStatusPda,
  getUserNoncePda,
  getPositionPda,
} from "@lightcone/sdk";
```

### Exchange PDA

Seeds: `["central_state"]`

```typescript
const [exchange, bump] = getExchangePda(programId);
```

### Market PDA

Seeds: `["market", market_id (u64 LE)]`

```typescript
const [market, bump] = getMarketPda(0n, programId);
```

### Vault PDA

Seeds: `["market_deposit_token_account", deposit_mint, market]`

```typescript
const [vault, bump] = getVaultPda(depositMint, market, programId);
```

### Mint Authority PDA

Seeds: `["market_mint_authority", market]`

```typescript
const [mintAuth, bump] = getMintAuthorityPda(market, programId);
```

### Conditional Mint PDA

Seeds: `["conditional_mint", market, deposit_mint, outcome_index (u8)]`

```typescript
// Single mint
const [mint, bump] = getConditionalMintPda(market, depositMint, 0, programId);

// All mints for a market
const mints = getAllConditionalMintPdas(market, depositMint, numOutcomes, programId);
// Returns [[mint0, bump0], [mint1, bump1], ...]
```

### Order Status PDA

Seeds: `["order_status", order_hash (32 bytes)]`

```typescript
const [orderStatus, bump] = getOrderStatusPda(orderHash, programId);
```

### User Nonce PDA

Seeds: `["user_nonce", user (32 bytes)]`

```typescript
const [userNonce, bump] = getUserNoncePda(user, programId);
```

### Position PDA

Seeds: `["position", owner (32 bytes), market (32 bytes)]`

```typescript
const [position, bump] = getPositionPda(owner, market, programId);
```

## Order Utilities

### Standalone Functions

```typescript
import {
  hashOrder,
  getOrderMessage,
  signOrder,
  signOrderFull,
  verifyOrderSignature,
  serializeFullOrder,
  deserializeFullOrder,
  serializeCompactOrder,
  deserializeCompactOrder,
  createBidOrder,
  createAskOrder,
  createSignedBidOrder,
  createSignedAskOrder,
  isOrderExpired,
  ordersCanCross,
  calculateTakerFill,
} from "@lightcone/sdk";
```

### Order Hashing

```typescript
const hash = hashOrder(order);        // 32-byte keccak256
const message = getOrderMessage(order); // Same as hash (message to sign)
```

### Order Signing

```typescript
// Sign and return signature
const signature = signOrder(order, keypair);  // 64-byte Buffer

// Sign and return complete order
const signedOrder = signOrderFull(unsignedOrder, keypair);

// Verify signature
const valid = verifyOrderSignature(signedOrder);
```

### Order Serialization

```typescript
// Full order (225 bytes)
const bytes = serializeFullOrder(order);
const order = deserializeFullOrder(bytes);

// Compact order (65 bytes)
const bytes = serializeCompactOrder(compactOrder);
const compactOrder = deserializeCompactOrder(bytes);
```

### Order Creation

```typescript
// Create unsigned orders
const bid = createBidOrder({ nonce, maker, market, baseMint, quoteMint, makerAmount, takerAmount });
const ask = createAskOrder({ nonce, maker, market, baseMint, quoteMint, makerAmount, takerAmount });

// Create and sign in one step
const signedBid = createSignedBidOrder(params, keypair);
const signedAsk = createSignedAskOrder(params, keypair);
```

### Order Validation

```typescript
// Check expiration
const expired = isOrderExpired(order);
const expired = isOrderExpired(order, currentTimestamp);

// Check if orders can match
const canMatch = ordersCanCross(buyOrder, sellOrder);

// Calculate fill amounts
const takerFill = calculateTakerFill(makerOrder, makerFillAmount);
```

## Ed25519 Verification

Three strategies for Ed25519 signature verification:

### Individual Instructions

Each order gets its own Ed25519 instruction.

```typescript
import {
  createEd25519VerifyInstruction,
  createOrderVerifyInstruction,
  buildMatchOrdersTransaction,
} from "@lightcone/sdk";

// Create verify instruction for an order
const verifyIx = createOrderVerifyInstruction(signedOrder);

// Build complete transaction (recommended)
const tx = buildMatchOrdersTransaction(params, programId);
// Includes: [takerVerify, makerVerify1, ..., matchOrdersMulti]
```

### Batch Verification

All signatures in one Ed25519 instruction.

```typescript
import {
  createBatchEd25519VerifyInstruction,
  buildCompactMatchOrdersTransaction,
} from "@lightcone/sdk";

// Build batch transaction
const tx = buildCompactMatchOrdersTransaction(params, programId);
// Includes: [batchVerify, matchOrdersMulti]
```

### Cross-Instruction Reference (Most Efficient)

Ed25519 instructions reference data in the match instruction.

```typescript
import {
  createCrossRefEd25519Instructions,
  buildCrossRefMatchOrdersTransaction,
} from "@lightcone/sdk";

// Build cross-ref transaction (smallest size)
const tx = buildCrossRefMatchOrdersTransaction(params, programId);
// Includes: [takerVerify(16B), makerVerify(16B), matchOrdersMulti]
```

## Account Deserialization

```typescript
import {
  deserializeExchange,
  deserializeMarket,
  deserializePosition,
  deserializeOrderStatus,
  deserializeUserNonce,
  isExchangeAccount,
  isMarketAccount,
  isPositionAccount,
  isOrderStatusAccount,
  isUserNonceAccount,
} from "@lightcone/sdk";

// Deserialize account data
const exchange = deserializeExchange(accountData);
const market = deserializeMarket(accountData);
const position = deserializePosition(accountData);
const orderStatus = deserializeOrderStatus(accountData);
const userNonce = deserializeUserNonce(accountData);

// Check account type
if (isMarketAccount(accountData)) {
  const market = deserializeMarket(accountData);
}
```

## Complete Example

```typescript
import { Connection, Keypair } from "@solana/web3.js";
import {
  LightconePinocchioClient,
  OrderSide,
  buildCrossRefMatchOrdersTransaction,
} from "@lightcone/sdk";

async function main() {
  const connection = new Connection("https://api.devnet.solana.com");
  const client = new LightconePinocchioClient(connection);

  // Create keypairs
  const maker = Keypair.generate();
  const taker = Keypair.generate();

  // Fetch market info
  const market = await client.getMarket(0n);
  const marketPda = client.pda.getMarketPda(0n, client.programId)[0];

  // Get conditional mints
  const mints = client.getConditionalMints(marketPda, usdcMint, market.numOutcomes);
  const yesMint = mints[0];
  const noMint = mints[1];

  // Create and sign orders
  const makerNonce = await client.getNextNonce(maker.publicKey);
  const makerOrder = client.signFullOrder(
    client.createAskOrder({
      nonce: makerNonce,
      maker: maker.publicKey,
      market: marketPda,
      baseMint: yesMint,
      quoteMint: noMint,
      makerAmount: 500_000n,
      takerAmount: 500_000n,
    }),
    maker
  );

  const takerNonce = await client.getNextNonce(taker.publicKey);
  const takerOrder = client.signFullOrder(
    client.createBidOrder({
      nonce: takerNonce,
      maker: taker.publicKey,
      market: marketPda,
      baseMint: yesMint,
      quoteMint: noMint,
      makerAmount: 500_000n,
      takerAmount: 500_000n,
    }),
    taker
  );

  // Build match transaction with verification
  const result = await client.matchOrdersMultiWithVerify({
    operator: operatorPubkey,
    market: marketPda,
    baseMint: yesMint,
    quoteMint: noMint,
    takerOrder,
    makerOrders: [makerOrder],
    fillAmounts: [500_000n],
  });

  // Sign and send
  result.transaction.sign(operatorKeypair);
  const signature = await connection.sendRawTransaction(
    result.transaction.serialize()
  );
  console.log(`Match tx: ${signature}`);
}
```

## Instruction Builders

Low-level instruction builders for advanced usage:

```typescript
import {
  buildInitializeIx,
  buildCreateMarketIx,
  buildAddDepositMintIx,
  buildMintCompleteSetIx,
  buildMergeCompleteSetIx,
  buildCancelOrderIx,
  buildIncrementNonceIx,
  buildSettleMarketIx,
  buildRedeemWinningsIx,
  buildSetPausedIx,
  buildSetOperatorIx,
  buildWithdrawFromPositionIx,
  buildActivateMarketIx,
  buildMatchOrdersMultiIx,
} from "@lightcone/sdk";
```
