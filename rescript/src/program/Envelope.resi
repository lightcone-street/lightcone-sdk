// Exact order construction and signing. Offline builders require server rules.
// Build a signed limit order request (no network call).
let buildLimitOrder: (
  ~maker: SolanaKit.address,
  ~market: SolanaKit.address,
  ~baseMint: SolanaKit.address,
  ~quoteMint: SolanaKit.address,
  ~side: int,
  ~price: string,
  ~size: string,
  ~rules: Scaling.OrderbookRules.t,
  ~orderbookId: string,
  ~keypair: SolanaKit.cryptoKeyPair,
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
  ~rules: Scaling.OrderbookRules.t,
  ~orderbookId: string,
  ~keypair: SolanaKit.cryptoKeyPair,
  ~nonce: bigint=?,
  ~salt: bigint=?,
  ~expiration: bigint=?,
  ~timeInForce: Shared.TimeInForce.t=?,
  ~depositSource: Shared.DepositSource.t=?,
) => promise<result<Order.Raw.SubmitResponse.t, SdkError.t>>

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
  ~orderbookId: string,
  ~nonce: bigint=?,
  ~salt: bigint=?,
  ~expiration: bigint=?,
  ~timeInForce: Shared.TimeInForce.t=?,
  ~depositSource: Shared.DepositSource.t=?,
) => promise<result<Order.Raw.SubmitResponse.t, SdkError.t>>
