// Typed program failures shared with the pinned Rust contract.
@genType
type t =
  | InvalidPayoutNumerators
  | PayoutVectorExceedsU32
  | InvalidScalarRange
  | DuplicateScalarOutcomes
  | SignatureVerificationFailed
  | InvalidSignature
  | Overflow
  | InvalidMintOrder
  | OrderbookExists
  | InvalidMarket
  | MarketSettled
  | InvalidProgramId
  | InvalidOrderbook
  | FullFillRequired
  | DivisionByZero
  | Reserved50
  | InsufficientGlobalDeposit
  | InvalidDepositMintOrder
  | ZeroAmount
  | InvalidAta
  | OrderNotFullyFilled
  | PayoutTooSmall
  | TokenAccountNotEmpty
  | InvalidManager
  | InvalidFeeRange
  | InvalidFeeSum
  | InvalidFeeReceiver
  | InvalidOracle
  | NoPendingRoleTransfer
  | PendingRoleMismatch
  | InvalidEventAuthority
  | EventBatchOverflow
  | InvalidEventBatch
  | InvalidEventContract
  | UnsupportedEventSchema
  | PublicInstructionMustBeTopLevel
  | InactiveDepositToken
  | DepositMintMismatch
  | InvalidConditionalMint
  | UnsignedOrder
  | InvalidDiscriminator({expected: string, actual: string})
  | AccountNotFound(string)
  | InvalidDataLength({expected: int, actual: int})
  | InvalidOutcomeCount(int)
  | InvalidOutcomeIndex({index: int, max: int})
  | TooManyMakers(int)
  | InvalidSide(int)
  | InvalidMarketStatus(int)
  | InvalidPendingRoleKind(int)
  | MissingField(string)
  | TooManyDepositMints(int)
  | InvalidPubkey(string)
  | InvalidTransaction(string)
  | Serialization(string)
  | Scaling(Scaling.Error.t)
  | Other(string)
let toMessage = error =>
  switch error {
  | InvalidPayoutNumerators => "Invalid Payout Numerators"
  | PayoutVectorExceedsU32 => "Payout Vector Exceeds U32"
  | InvalidScalarRange => "Invalid Scalar Range"
  | DuplicateScalarOutcomes => "Duplicate Scalar Outcomes"
  | SignatureVerificationFailed => "Signature Verification Failed"
  | InvalidSignature => "Invalid Signature"
  | Overflow => "Overflow"
  | InvalidMintOrder => "Invalid Mint Order"
  | OrderbookExists => "Orderbook Exists"
  | InvalidMarket => "Invalid Market"
  | MarketSettled => "Market Settled"
  | InvalidProgramId => "Invalid Program Id"
  | InvalidOrderbook => "Invalid Orderbook"
  | FullFillRequired => "Full Fill Required"
  | DivisionByZero => "Division By Zero"
  | Reserved50 => "Reserved50"
  | InsufficientGlobalDeposit => "Insufficient Global Deposit"
  | InvalidDepositMintOrder => "Invalid Deposit Mint Order"
  | ZeroAmount => "Zero Amount"
  | InvalidAta => "Invalid Ata"
  | OrderNotFullyFilled => "Order Not Fully Filled"
  | PayoutTooSmall => "Payout Too Small"
  | TokenAccountNotEmpty => "Token Account Not Empty"
  | InvalidManager => "Invalid Manager"
  | InvalidFeeRange => "Invalid Fee Range"
  | InvalidFeeSum => "Invalid Fee Sum"
  | InvalidFeeReceiver => "Invalid Fee Receiver"
  | InvalidOracle => "Invalid Oracle"
  | NoPendingRoleTransfer => "No Pending Role Transfer"
  | PendingRoleMismatch => "Pending Role Mismatch"
  | InvalidEventAuthority => "Invalid Event Authority"
  | EventBatchOverflow => "Event Batch Overflow"
  | InvalidEventBatch => "Invalid Event Batch"
  | InvalidEventContract => "Invalid Event Contract"
  | UnsupportedEventSchema => "Unsupported Event Schema"
  | PublicInstructionMustBeTopLevel => "Public Instruction Must Be Top Level"
  | InactiveDepositToken => "Inactive Deposit Token"
  | DepositMintMismatch => "Deposit Mint Mismatch"
  | InvalidConditionalMint => "Invalid Conditional Mint"
  | UnsignedOrder => "Unsigned Order"
  | InvalidDiscriminator({expected, actual}) =>
    `Invalid discriminator: expected ${expected}, got ${actual}`
  | InvalidDataLength({expected, actual}) =>
    `Invalid data length: expected ${Int.toString(expected)}, got ${Int.toString(actual)}`
  | InvalidOutcomeIndex({index, max}) =>
    `Invalid outcome ${Int.toString(index)} (max ${Int.toString(max)})`
  | InvalidOutcomeCount(value)
  | TooManyMakers(value)
  | InvalidSide(value)
  | InvalidMarketStatus(value)
  | InvalidPendingRoleKind(value)
  | TooManyDepositMints(value) =>
    `Invalid program field: ${Int.toString(value)}`
  | AccountNotFound(value)
  | MissingField(value)
  | InvalidPubkey(value)
  | InvalidTransaction(value)
  | Serialization(value)
  | Other(value) => value
  | Scaling(error) => Scaling.toMessage(error)
  }
