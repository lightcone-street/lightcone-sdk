// Limit-order envelope — build + sign a limit order off-chain (align to tick →
// scale → keccak256 + ed25519 sign), optionally submitting it. The random-salt
// and error-formatting helpers are internal. `side` is 0 = Bid, 1 = Ask; `nonce`
// defaults to 0; `salt` is random when omitted.

// Build a signed limit order request (no network call).
let buildLimitOrder: (
  ~maker: SolanaKit.address,
  ~market: SolanaKit.address,
  ~baseMint: SolanaKit.address,
  ~quoteMint: SolanaKit.address,
  ~side: int,
  ~price: string,
  ~size: string,
  ~decimals: Scaling.orderbookDecimals,
  ~orderbookId: string,
  ~keypair: SolanaKit.cryptoKeyPair,
  ~nonce: bigint=?,
  ~salt: bigint=?,
  ~expiration: bigint=?,
  ~timeInForce: Shared.TimeInForce.t=?,
  ~depositSource: Shared.DepositSource.t=?,
) => promise<result<Order.submitOrderRequest, SdkError.t>>

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
  ~decimals: Scaling.orderbookDecimals,
  ~orderbookId: string,
  ~keypair: SolanaKit.cryptoKeyPair,
  ~nonce: bigint=?,
  ~salt: bigint=?,
  ~expiration: bigint=?,
  ~timeInForce: Shared.TimeInForce.t=?,
  ~depositSource: Shared.DepositSource.t=?,
) => promise<result<Order.submitOrderResponse, SdkError.t>>
