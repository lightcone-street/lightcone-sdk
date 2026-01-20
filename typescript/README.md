# Lightcone Pinocchio SDK

TypeScript SDK for interacting with the Lightcone Pinocchio program on Solana.

## Features

- **Full instruction coverage**: All 14 program instructions supported
- **Type-safe**: Full TypeScript support with comprehensive types
- **Efficient order matching**: Cross-instruction Ed25519 signature verification for minimal transaction size
- **Complete set operations**: Mint and merge conditional tokens
- **Order management**: Create, sign, and cancel orders
- **Position management**: Deposit, withdraw, and track positions

## Installation (TBC - Will deploy to NPM shortly)

```bash
npm install @lightcone/pinocchio-sdk
```

## Quick Start

```typescript
import {
  LightconePinocchioClient,
  createBidOrder,
  signOrderFull,
} from "@lightcone/pinocchio-sdk";
import { Connection, Keypair } from "@solana/web3.js";

// Initialize client
const connection = new Connection(process.env.RPC_URL);
const client = new LightconePinocchioClient(connection);

// Create and sign an order
const order = signOrderFull(
  createBidOrder({
    nonce: 0n,
    maker: wallet.publicKey,
    market: marketPda,
    baseMint: yesTokenMint,
    quoteMint: noTokenMint,
    makerAmount: 100_000n,
    takerAmount: 100_000n,
    expiration: BigInt(Math.floor(Date.now() / 1000) + 3600),
  }),
  wallet
);
```

## Program Instructions

The SDK supports all 14 Lightcone Pinocchio program instructions:

| # | Instruction | Description |
|---|-------------|-------------|
| 0 | `initialize` | Initialize the exchange (one-time setup) |
| 1 | `createMarket` | Create a new market |
| 2 | `addDepositMint` | Configure collateral token and create conditional mints |
| 3 | `mintCompleteSet` | Deposit collateral to mint YES + NO tokens |
| 4 | `mergeCompleteSet` | Burn YES + NO tokens to withdraw collateral |
| 5 | `cancelOrder` | Cancel an open order |
| 6 | `incrementNonce` | Increment user nonce (invalidates old orders) |
| 7 | `settleMarket` | Settle market with winning outcome |
| 8 | `redeemWinnings` | Redeem winning tokens for collateral |
| 9 | `setPaused` | Pause/unpause the exchange (admin) |
| 10 | `setOperator` | Change the operator address (admin) |
| 11 | `withdrawFromPosition` | Withdraw tokens from position to wallet |
| 12 | `activateMarket` | Activate a pending market |
| 13 | `matchOrdersMulti` | Match taker against up to 5 makers |

## Client Methods

### Exchange Management

```typescript
// Initialize exchange
const result = await client.initialize({
  authority: authorityPubkey,
  operator: operatorPubkey,
});

// Set paused state
await client.setPaused(authority, true);

// Change operator
await client.setOperator(authority, newOperatorPubkey);
```

### Market Operations

```typescript
// Create market
const result = await client.createMarket({
  authority: authorityPubkey,
  numOutcomes: 2,
  oracle: oraclePubkey,
  questionId: questionIdBuffer, // 32-byte Buffer
});

// Add deposit mint (numOutcomes required as 2nd parameter)
await client.addDepositMint(
  {
    authority: authorityPubkey,
    marketId: 0n,
    depositMint: usdcMint,
    outcomeMetadata: [
      { name: "YES", symbol: "YES", uri: "https://..." },
      { name: "NO", symbol: "NO", uri: "https://..." },
    ],
  },
  2 // numOutcomes
);

// Activate market
await client.activateMarket({
  authority: authorityPubkey,
  marketId: 0n,
});

// Settle market
await client.settleMarket({
  oracle: oraclePubkey,
  marketId: 0n,
  winningOutcome: 0,
});
```

### Token Operations

```typescript
// Mint complete set (deposit collateral, receive YES + NO)
// numOutcomes required as 2nd parameter
await client.mintCompleteSet(
  {
    user: userPubkey,
    market: marketPda,
    depositMint: usdcMint,
    amount: 1_000_000n,
  },
  2 // numOutcomes
);

// Merge complete set (burn YES + NO, receive collateral)
// numOutcomes required as 2nd parameter
await client.mergeCompleteSet(
  {
    user: userPubkey,
    market: marketPda,
    depositMint: usdcMint,
    amount: 500_000n,
  },
  2 // numOutcomes
);

// Withdraw from position
// isToken2022 required as 2nd parameter
await client.withdrawFromPosition(
  {
    user: userPubkey,
    market: marketPda,
    mint: conditionalMint,
    amount: 100_000n,
  },
  false // isToken2022
);

// Redeem winnings after settlement
// winningOutcome required as 2nd parameter
await client.redeemWinnings(
  {
    user: userPubkey,
    market: marketPda,
    depositMint: usdcMint,
    amount: 1_000_000n,
  },
  0 // winningOutcome
);
```

### Order Management

```typescript
import {
  createBidOrder,
  createAskOrder,
  signOrderFull,
  hashOrder,
} from "@lightcone/pinocchio-sdk";

// Create and sign orders
const bidOrder = signOrderFull(
  createBidOrder({
    nonce: await client.getNextNonce(buyer.publicKey),
    maker: buyer.publicKey,
    market: marketPda,
    baseMint: yesToken,
    quoteMint: noToken,
    makerAmount: 100_000n, // NO tokens to give
    takerAmount: 100_000n, // YES tokens to receive
    expiration: BigInt(Math.floor(Date.now() / 1000) + 3600),
  }),
  buyer
);

// Cancel order - takes maker pubkey and full order object
await client.cancelOrder(buyer.publicKey, bidOrder);

// Increment nonce (invalidates all pending orders with old nonce)
await client.incrementNonce(buyer.publicKey);
```

### Order Matching

```typescript
// Option 1: Use client method (recommended)
const matchResult = await client.matchOrdersMultiWithVerify({
  operator: operatorPubkey,
  market: marketPda,
  baseMint: yesToken,
  quoteMint: noToken,
  takerOrder: bidOrder,
  makerOrders: [askOrder],
  fillAmounts: [100_000n],
});
matchResult.transaction.sign(operator);
await connection.sendRawTransaction(matchResult.transaction.serialize());

// Option 2: Use standalone function
import { buildCrossRefMatchOrdersTransaction, PROGRAM_ID } from "@lightcone/pinocchio-sdk";

const matchTx = buildCrossRefMatchOrdersTransaction(
  {
    operator: operatorPubkey,
    market: marketPda,
    baseMint: yesToken,
    quoteMint: noToken,
    takerOrder: bidOrder,
    makerOrders: [askOrder],
    fillAmounts: [100_000n],
  },
  PROGRAM_ID
);
matchTx.recentBlockhash = (await connection.getLatestBlockhash()).blockhash;
matchTx.feePayer = operatorPubkey;
matchTx.sign(operator);
await connection.sendRawTransaction(matchTx.serialize());
```

## PDA Derivation

```typescript
import {
  getExchangePda,
  getMarketPda,
  getPositionPda,
  getConditionalMintPda,
  getOrderStatusPda,
  getUserNoncePda,
  getVaultPda,
  getMintAuthorityPda,
  getAllConditionalMintPdas,
} from "@lightcone/pinocchio-sdk";

// Core PDAs
const [exchange] = getExchangePda(programId);
const [market] = getMarketPda(marketId, programId);
const [position] = getPositionPda(user, market, programId);
const [conditionalMint] = getConditionalMintPda(market, depositMint, outcomeIndex, programId);
const [orderStatus] = getOrderStatusPda(orderHash, programId);
const [userNonce] = getUserNoncePda(user, programId);

// Additional PDAs
const [vault] = getVaultPda(depositMint, market, programId);
const [mintAuthority] = getMintAuthorityPda(market, programId);
const allMints = getAllConditionalMintPdas(market, depositMint, numOutcomes, programId);
```

## Account Deserialization

```typescript
import {
  deserializeExchange,
  deserializeMarket,
  deserializePosition,
  deserializeOrderStatus,
  deserializeUserNonce,
} from "@lightcone/pinocchio-sdk";

const exchangeData = await connection.getAccountInfo(exchangePda);
const exchange = deserializeExchange(exchangeData.data);

const marketData = await connection.getAccountInfo(marketPda);
const market = deserializeMarket(marketData.data);
```

## Order Utilities

```typescript
import {
  createBidOrder,
  createAskOrder,
  signOrderFull,
  hashOrder,
  ordersCanCross,
  calculateTakerFill,
} from "@lightcone/pinocchio-sdk";

// Create unsigned order
const unsignedOrder = createBidOrder({ ... });

// Sign order
const signedOrder = signOrderFull(unsignedOrder, keypair);

// Hash order (for order status lookups)
const hash = hashOrder(signedOrder);

// Check if orders can match
const canMatch = ordersCanCross(bidOrder, askOrder);

// Calculate fill amount
const fillAmount = calculateTakerFill(takerOrder, makerOrder, makerFillAmount);
```

## Client Utility Methods

```typescript
// Get all conditional mint addresses for a market
const mints = client.getConditionalMints(market, depositMint, numOutcomes);

// Derive condition ID
const conditionId = client.deriveConditionId(oracle, questionId, numOutcomes);

// Get user's current nonce (for order creation)
const nonce = await client.getNextNonce(userPubkey);

// Get next market ID
const nextId = await client.getNextMarketId();
```

## Testing

```bash
# Unit tests (no network required)
npm run test:unit

# Devnet integration tests
npm run test:devnet

# Stress test (100 concurrent order matches)
npm run test:stress

# Run all tests
npm run test:all
```

## Configuration

Create a `.env` file:

```env
RPC_URL=https://your-rpc-endpoint.com
```

Place your authority keypair at `keypairs/devnet-authority.json` for testing.

## Program ID

**Devnet**: `Aumw7EC9nnxDjQFzr1fhvXvnG3Rn3Bb5E3kbcbLrBdEk`

## Architecture

The Lightcone Pinocchio program is a high-performance market CLOB (Central Limit Order Book) built with [Pinocchio](https://github.com/febo/pinocchio) - a zero-dependency, zero-copy Solana program framework.

### Key Design Decisions

1. **Ed25519 Signature Verification**: Orders are signed off-chain and verified on-chain using the Ed25519 program
2. **Cross-instruction References**: Signature data is only stored once in the match instruction, with Ed25519 verify instructions referencing offsets
3. **Position Accounts**: User tokens are held in Position PDAs, enabling atomic matching
4. **Complete Sets**: Users can only mint/burn complete sets of conditional tokens, ensuring market integrity

### Transaction Size Optimization

The SDK uses cross-instruction Ed25519 references to minimize transaction size:

| Approach | Size | Status |
|----------|------|--------|
| Embedded Ed25519 data | ~1,386 bytes | Over limit |
| **Cross-instruction refs** | ~1,040 bytes | Under 1,232 limit |

## License

MIT
