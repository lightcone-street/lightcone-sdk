// Order envelopes — build + sign an order off-chain (align to tick → scale →
// keccak256 + ed25519 sign), optionally submitting it. A trigger order is the
// same signed payload with `triggerPrice` / `triggerType` on the request. The
// random-salt and error-formatting helpers are internal. `side` is 0 = Bid,
// 1 = Ask; `nonce` defaults to 0; `salt` is random when omitted.

// Build a signed limit order request (no network call).
let buildLimitOrder: (
  ~maker: SolanaKit.address,
  ~market: SolanaKit.address,
  ~baseMint: SolanaKit.address,
  ~quoteMint: SolanaKit.address,
  ~side: int,
  ~price: string,
  ~size: string,
  ~decimals: Scaling.OrderbookDecimals.t,
  ~orderbookId: string,
  ~keypair: SolanaKit.cryptoKeyPair,
  ~nonce: bigint=?,
  ~salt: bigint=?,
  ~expiration: bigint=?,
  ~timeInForce: Shared.TimeInForce.t=?,
  ~depositSource: Shared.DepositSource.t=?,
) => promise<result<Order.Raw.SubmitRequest.t, SdkError.t>>

// Build a signed trigger (take-profit / stop-loss) order request (no network call).
let buildTriggerOrder: (
  ~maker: SolanaKit.address,
  ~market: SolanaKit.address,
  ~baseMint: SolanaKit.address,
  ~quoteMint: SolanaKit.address,
  ~side: int,
  ~price: string,
  ~size: string,
  ~decimals: Scaling.OrderbookDecimals.t,
  ~orderbookId: string,
  ~keypair: SolanaKit.cryptoKeyPair,
  ~triggerPrice: float,
  ~triggerType: Shared.TriggerType.t,
  ~nonce: bigint=?,
  ~salt: bigint=?,
  ~expiration: bigint=?,
  ~timeInForce: Shared.TimeInForce.t=?,
  ~depositSource: Shared.DepositSource.t=?,
) => promise<result<Order.Raw.SubmitRequest.t, SdkError.t>>

// Build + sign + submit in one step.
let submitLimitOrder: (
  Client.t,
  ~maker: SolanaKit.address,
  ~market: SolanaKit.address,
  ~baseMint: SolanaKit.address,
  ~quoteMint: SolanaKit.address,
  ~side: int,
  ~price: string,
  ~size: string,
  ~decimals: Scaling.OrderbookDecimals.t,
  ~orderbookId: string,
  ~keypair: SolanaKit.cryptoKeyPair,
  ~nonce: bigint=?,
  ~salt: bigint=?,
  ~expiration: bigint=?,
  ~timeInForce: Shared.TimeInForce.t=?,
  ~depositSource: Shared.DepositSource.t=?,
) => promise<result<Order.Raw.SubmitResponse.t, SdkError.t>>

// Build + sign + submit a trigger order in one step.
let submitTriggerOrder: (
  Client.t,
  ~maker: SolanaKit.address,
  ~market: SolanaKit.address,
  ~baseMint: SolanaKit.address,
  ~quoteMint: SolanaKit.address,
  ~side: int,
  ~price: string,
  ~size: string,
  ~decimals: Scaling.OrderbookDecimals.t,
  ~orderbookId: string,
  ~keypair: SolanaKit.cryptoKeyPair,
  ~triggerPrice: float,
  ~triggerType: Shared.TriggerType.t,
  ~nonce: bigint=?,
  ~salt: bigint=?,
  ~expiration: bigint=?,
  ~timeInForce: Shared.TimeInForce.t=?,
  ~depositSource: Shared.DepositSource.t=?,
) => promise<result<Order.Raw.TriggerResponse.t, SdkError.t>>

// ── Strategy-signed submits ────────────────────────────────────────────────────
// Build + sign + submit with the client's configured strategy (native keypair or
// external wallet adapter): the strategy's address is the maker, and an omitted
// nonce falls back to the client's cached order nonce (an explicit one updates
// the cache).
let submitLimitOrderSigned: (
  Client.t,
  ~market: SolanaKit.address,
  ~baseMint: SolanaKit.address,
  ~quoteMint: SolanaKit.address,
  ~side: int,
  ~price: string,
  ~size: string,
  ~decimals: Scaling.OrderbookDecimals.t,
  ~orderbookId: string,
  ~nonce: bigint=?,
  ~salt: bigint=?,
  ~expiration: bigint=?,
  ~timeInForce: Shared.TimeInForce.t=?,
  ~depositSource: Shared.DepositSource.t=?,
) => promise<result<Order.Raw.SubmitResponse.t, SdkError.t>>

let submitTriggerOrderSigned: (
  Client.t,
  ~market: SolanaKit.address,
  ~baseMint: SolanaKit.address,
  ~quoteMint: SolanaKit.address,
  ~side: int,
  ~price: string,
  ~size: string,
  ~decimals: Scaling.OrderbookDecimals.t,
  ~orderbookId: string,
  ~triggerPrice: float,
  ~triggerType: Shared.TriggerType.t,
  ~nonce: bigint=?,
  ~salt: bigint=?,
  ~expiration: bigint=?,
  ~timeInForce: Shared.TimeInForce.t=?,
  ~depositSource: Shared.DepositSource.t=?,
) => promise<result<Order.Raw.TriggerResponse.t, SdkError.t>>
