# WebSocket Module Reference

Real-time streaming client for Lightcone market data and user events.

## Client Configuration

### LightconeWebSocketClient

```typescript
import { websocket } from "@lightcone/sdk";

// Connect with defaults
const client = await websocket.LightconeWebSocketClient.connectDefault();

// Connect with custom URL
const client = await websocket.LightconeWebSocketClient.connect(
  "wss://custom.endpoint/ws",
  {
    reconnectAttempts: 10,
    baseDelayMs: 1000,
    maxDelayMs: 30000,
    pingIntervalSecs: 30,
    autoReconnect: true,
    autoResubscribe: true,
  }
);

// Authenticated connection for user streams
import { Keypair } from "@solana/web3.js";
const keypair = Keypair.generate();
const client = await websocket.LightconeWebSocketClient.connectAuthenticated(keypair);
```

| Option | Default | Description |
|--------|---------|-------------|
| `reconnectAttempts` | 10 | Maximum reconnection attempts |
| `baseDelayMs` | 1000 | Base delay for exponential backoff |
| `maxDelayMs` | 30000 | Maximum reconnection delay |
| `pingIntervalSecs` | 30 | Client ping interval |
| `autoReconnect` | true | Auto-reconnect on disconnect |
| `autoResubscribe` | true | Restore subscriptions after reconnect |
| `authToken` | - | Authentication token for user streams |

### Connection States

```typescript
type ConnectionState =
  | "Disconnected"
  | "Connecting"
  | "Connected"
  | "Reconnecting"
  | "Disconnecting";

// Check state
const state = client.connectionState();
const isConnected = client.isConnected();
const isAuthenticated = client.isAuthenticated();
```

## Subscriptions

### Book Updates

Subscribe to real-time orderbook depth updates.

```typescript
// Subscribe to orderbook updates
client.subscribeBookUpdates(["market1:orderbook1", "market2:orderbook2"]);

// Unsubscribe
client.unsubscribeBookUpdates(["market1:orderbook1"]);
```

Updates include:
- **Snapshots**: Full orderbook state on initial subscription
- **Deltas**: Incremental updates with sequence numbers
- **Resync**: Signal to re-fetch full state (sequence gap detected)

### Trades

Subscribe to trade executions.

```typescript
client.subscribeTrades(["market1:orderbook1"]);
client.unsubscribeTrades(["market1:orderbook1"]);
```

### User Events (Authenticated)

Subscribe to private user streams (requires authentication).

```typescript
// Connect with authentication
const client = await websocket.LightconeWebSocketClient.connectAuthenticated(keypair);

// Subscribe to user events
client.subscribeUser(keypair.publicKey.toBase58());

// Unsubscribe
client.unsubscribeUser(keypair.publicKey.toBase58());
```

User events include:
- Order placements and cancellations
- Fill notifications
- Balance updates

### Price History

Subscribe to historical price data with optional OHLCV candles.

```typescript
import { Resolution } from "@lightcone/sdk";

// Subscribe with OHLCV data
client.subscribePriceHistory("orderbook_id", Resolution.OneHour, true);

// Subscribe without OHLCV (midpoint only)
client.subscribePriceHistory("orderbook_id", Resolution.OneHour, false);

// Unsubscribe
client.unsubscribePriceHistory("orderbook_id", Resolution.OneHour);
```

### Market Events

Subscribe to market lifecycle events.

```typescript
client.subscribeMarket("market_pubkey");
client.unsubscribeMarket("market_pubkey");
```

Market events include:
- `OrderbookCreated`: New orderbook added
- `Opened`: Market activated for trading
- `Paused`: Trading paused
- `Settled`: Market resolved

## Event Handling

### Event Callback

```typescript
import { websocket } from "@lightcone/sdk";

client.on((event) => {
  switch (event.type) {
    case "Connected":
      console.log("Connected to server");
      break;

    case "Disconnected":
      console.log(`Disconnected: ${event.reason}`);
      break;

    case "Reconnecting":
      console.log(`Reconnecting (attempt ${event.attempt})`);
      break;

    case "BookUpdate":
      const book = client.getOrderbook(event.orderbookId);
      if (book) {
        console.log(`Best bid: ${book.bestBid()?.[0]}`);
        console.log(`Best ask: ${book.bestAsk()?.[0]}`);
      }
      break;

    case "Trade":
      console.log(`Trade: ${event.trade.size} @ ${event.trade.price}`);
      break;

    case "UserUpdate":
      const state = client.getUserState(event.user);
      if (state) {
        console.log(`Open orders: ${state.orderCount()}`);
      }
      break;

    case "PriceUpdate":
      const history = client.getPriceHistory(event.orderbookId, event.resolution);
      if (history) {
        console.log(`Midpoint: ${history.currentMidpoint()}`);
      }
      break;

    case "MarketEvent":
      console.log(`Market ${event.marketPubkey}: ${event.eventType}`);
      break;

    case "ResyncRequired":
      console.log(`Resync needed for ${event.orderbookId}`);
      break;

    case "Pong":
      // Server responded to ping
      break;

    case "Error":
      console.error(`Error: ${event.error.message}`);
      break;
  }
});

// Remove callback
client.off(callback);
```

### Event Types

| Event | Fields | Description |
|-------|--------|-------------|
| `Connected` | - | Successfully connected |
| `Disconnected` | `reason` | Connection closed |
| `Reconnecting` | `attempt` | Attempting reconnection |
| `BookUpdate` | `orderbookId`, `isSnapshot` | Orderbook updated |
| `Trade` | `orderbookId`, `trade` | Trade executed |
| `UserUpdate` | `eventType`, `user` | User state changed |
| `PriceUpdate` | `orderbookId`, `resolution` | Price history updated |
| `MarketEvent` | `eventType`, `marketPubkey` | Market lifecycle event |
| `ResyncRequired` | `orderbookId` | Sequence gap detected |
| `Pong` | - | Ping response received |
| `Error` | `error` | Error occurred |

## State Management

### LocalOrderbook

Maintains synchronized orderbook state.

```typescript
const book = client.getOrderbook("orderbook_id");
if (book) {
  // Best prices
  const [bidPrice, bidSize] = book.bestBid() ?? ["", ""];
  const [askPrice, askSize] = book.bestAsk() ?? ["", ""];

  // Spread and midpoint
  const spread = book.spread();
  const mid = book.midpoint();

  // Full depth
  const bids = book.getBids();  // Sorted descending by price
  const asks = book.getAsks();  // Sorted ascending by price

  // Top N levels
  const top5Bids = book.getTopBids(5);
  const top5Asks = book.getTopAsks(5);

  // Size at specific price
  const bidSizeAt = book.bidSizeAt("0.500000");
  const askSizeAt = book.askSizeAt("0.500000");

  // Total depth
  const totalBidDepth = book.totalBidDepth();
  const totalAskDepth = book.totalAskDepth();

  // Level counts
  const bidCount = book.bidCount();
  const askCount = book.askCount();

  // State info
  const hasSnapshot = book.hasSnapshot();
  const expectedSeq = book.expectedSequence();
  const lastUpdate = book.lastTimestamp();
}
```

### UserState

Tracks user orders and balances.

```typescript
const state = client.getUserState("user_pubkey");
if (state) {
  // Orders
  const allOrders = state.openOrders();
  const marketOrders = state.ordersForMarket("market_pubkey");
  const orderbookOrders = state.ordersForOrderbook("orderbook_id");
  const order = state.getOrder("order_hash");

  // Balances
  const allBalances = state.allBalances();
  const balance = state.getBalance("market_pubkey", "deposit_mint");
  const idle = state.idleBalanceForOutcome("market", "mint", 0);
  const onBook = state.onBookBalanceForOutcome("market", "mint", 0);

  // Counts
  const orderCount = state.orderCount();
  const hasSnapshot = state.hasSnapshot();
}
```

### PriceHistory

Maintains historical price candles.

```typescript
const history = client.getPriceHistory("orderbook_id", "1h");
if (history) {
  // All candles (newest first)
  const candles = history.candles();
  const recent = history.recentCandles(10);

  // Specific candle
  const candle = history.getCandle(timestamp);
  const latest = history.latestCandle();
  const oldest = history.oldestCandle();

  // Current prices
  const mid = history.currentMidpoint();
  const bid = history.currentBestBid();
  const ask = history.currentBestAsk();

  // Metadata
  const count = history.candleCount();
  const hasSnapshot = history.hasSnapshot();
  const serverTime = history.serverTime();
  const resolution = history.resolutionEnum();
}
```

## Authentication

### Direct Authentication

```typescript
import { websocket } from "@lightcone/sdk";
import { Keypair } from "@solana/web3.js";

// Authenticate and get credentials
const keypair = Keypair.generate();
const credentials = await websocket.authenticateWithKeypair(keypair);
console.log(`Auth token: ${credentials.authToken}`);
console.log(`User: ${credentials.userPubkey}`);

// Or use secret key directly
const secretKey = new Uint8Array(64);
const credentials = await websocket.authenticate(secretKey);
```

### Sign-in Message

```typescript
// Generate sign-in message
const message = websocket.generateSigninMessage();
// "Sign in to Lightcone\n\nTimestamp: 1234567890123"

// With specific timestamp
const message = websocket.generateSigninMessageWithTimestamp(Date.now());

// Sign message manually
const signature = websocket.signMessage(message, keypair);
```

### Auth Flow

1. Generate sign-in message with timestamp
2. Sign message with Ed25519 keypair
3. POST to `/api/auth/login_or_register_with_message`
4. Extract `auth_token` from response cookie
5. Connect WebSocket with token in Cookie header

## Message Types

### Request Types

```typescript
// Subscribe
{ method: "subscribe", params: BookUpdateParams | TradesParams | ... }

// Unsubscribe
{ method: "unsubscribe", params: BookUpdateParams | TradesParams | ... }

// Ping
{ method: "ping" }
```

### Subscription Parameters

| Type | Fields |
|------|--------|
| `book_update` | `orderbook_ids: string[]` |
| `trades` | `orderbook_ids: string[]` |
| `user` | `user: string` |
| `price_history` | `orderbook_id`, `resolution`, `include_ohlcv` |
| `market` | `market_pubkey: string` |

### Response Data Types

#### BookUpdateData

| Field | Type | Description |
|-------|------|-------------|
| `orderbook_id` | string | Orderbook identifier |
| `timestamp` | string | ISO timestamp |
| `seq` | number | Sequence number |
| `bids` | PriceLevel[] | Bid levels |
| `asks` | PriceLevel[] | Ask levels |
| `is_snapshot` | boolean | Full state or delta |
| `resync` | boolean | Resync required |

#### TradeData

| Field | Type | Description |
|-------|------|-------------|
| `orderbook_id` | string | Orderbook identifier |
| `price` | string | Trade price (decimal string) |
| `size` | string | Trade size (decimal string) |
| `side` | string | "buy" or "sell" |
| `timestamp` | string | ISO timestamp |
| `trade_id` | string | Unique trade ID |

#### UserEventData

| Field | Type | Description |
|-------|------|-------------|
| `event_type` | string | "snapshot", "order_update", "balance_update" |
| `orders` | Order[] | Open orders (snapshot) |
| `balances` | Record<string, BalanceEntry> | Balances (snapshot) |
| `order` | OrderUpdate? | Order update data |
| `balance` | Balance? | Balance update data |

#### Candle

| Field | Type | Description |
|-------|------|-------------|
| `t` | number | Timestamp (Unix ms) |
| `m` | string? | Midpoint price |
| `o` | string? | Open price (if OHLCV) |
| `h` | string? | High price (if OHLCV) |
| `l` | string? | Low price (if OHLCV) |
| `c` | string? | Close price (if OHLCV) |
| `v` | string? | Volume (if OHLCV) |
| `bb` | string? | Best bid |
| `ba` | string? | Best ask |

## Error Handling

```typescript
import { websocket } from "@lightcone/sdk";

client.on((event) => {
  if (event.type === "Error") {
    const error = event.error;
    switch (error.variant) {
      case "ConnectionFailed":
        console.log(`Connection failed: ${error.message}`);
        break;
      case "ConnectionClosed":
        console.log(`Connection closed: ${error.code}`);
        break;
      case "RateLimited":
        console.log("Rate limited by server");
        break;
      case "MessageParseError":
        console.log(`Parse error: ${error.message}`);
        break;
      case "SequenceGap":
        console.log(`Sequence gap: ${error.details}`);
        break;
      case "ResyncRequired":
        console.log(`Resync needed: ${error.details?.orderbookId}`);
        break;
      case "PingTimeout":
        console.log("Ping timeout");
        break;
      case "ServerError":
        console.log(`Server error: ${error.code} - ${error.message}`);
        break;
      case "AuthenticationFailed":
        console.log(`Auth failed: ${error.message}`);
        break;
      case "NotConnected":
        console.log("Not connected");
        break;
    }
  }
});
```

### WebSocketError Variants

| Variant | Description |
|---------|-------------|
| `ConnectionFailed` | Failed to establish connection |
| `ConnectionClosed` | Connection was closed |
| `RateLimited` | Rate limit exceeded |
| `MessageParseError` | Failed to parse message |
| `SequenceGap` | Orderbook sequence gap |
| `ResyncRequired` | Orderbook resync needed |
| `PingTimeout` | Server didn't respond to ping |
| `ServerError` | Server-side error |
| `AuthenticationFailed` | Authentication failed |
| `ChannelClosed` | Internal channel closed |
| `NotConnected` | Not connected to server |
| `InvalidUrl` | Invalid WebSocket URL |
| `Protocol` | Protocol error |
| `HttpError` | HTTP error during auth |

## Reconnection

Auto-reconnection uses exponential backoff with jitter:

```typescript
// Full jitter formula
delay = random(0, baseDelayMs * 2^attempt)
cappedDelay = min(delay, maxDelayMs)
```

Example delays with defaults:
- Attempt 1: 0-1000ms
- Attempt 2: 0-2000ms
- Attempt 3: 0-4000ms
- ...
- Capped at 30000ms

After reconnection, subscriptions are automatically restored if `autoResubscribe` is enabled.

## Complete Example

```typescript
import { websocket, Resolution } from "@lightcone/sdk";
import { Keypair } from "@solana/web3.js";

async function main() {
  // Create authenticated client
  const keypair = Keypair.generate();
  const client = await websocket.LightconeWebSocketClient.connectAuthenticated(keypair);

  // Register event handler
  client.on((event) => {
    switch (event.type) {
      case "Connected":
        console.log("Connected!");
        break;
      case "BookUpdate":
        const book = client.getOrderbook(event.orderbookId);
        if (book) {
          console.log(`${event.orderbookId}:`);
          console.log(`  Bid: ${book.bestBid()?.[0]} x ${book.bestBid()?.[1]}`);
          console.log(`  Ask: ${book.bestAsk()?.[0]} x ${book.bestAsk()?.[1]}`);
          console.log(`  Spread: ${book.spread()}`);
        }
        break;
      case "Trade":
        console.log(`Trade: ${event.trade.size} @ ${event.trade.price}`);
        break;
      case "UserUpdate":
        const state = client.getUserState(event.user);
        if (state) {
          console.log(`Orders: ${state.orderCount()}`);
          for (const order of state.openOrders()) {
            console.log(`  ${order.order_hash}: ${order.remaining} @ ${order.price}`);
          }
        }
        break;
      case "Error":
        console.error(`Error: ${event.error.message}`);
        break;
    }
  });

  // Subscribe to channels
  client.subscribeBookUpdates(["market1:orderbook1"]);
  client.subscribeTrades(["market1:orderbook1"]);
  client.subscribeUser(keypair.publicKey.toBase58());
  client.subscribePriceHistory("market1:orderbook1", Resolution.OneHour, true);

  // Keep running
  await new Promise((resolve) => setTimeout(resolve, 60000));

  // Cleanup
  await client.disconnect();
}

main().catch(console.error);
```

## Control Methods

```typescript
// Manual ping
client.ping();

// Disconnect
await client.disconnect();

// Get URL and config
const url = client.getUrl();
const config = client.getConfig();

// Get auth info
const credentials = client.getAuthCredentials();
const userPubkey = client.userPubkey();
```
