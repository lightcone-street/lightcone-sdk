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
let toMessage: t => string
