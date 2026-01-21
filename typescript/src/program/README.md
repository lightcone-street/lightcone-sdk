# Program Module Reference

On-chain Solana program interaction for the Lightcone protocol.

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
