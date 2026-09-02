import { MAX_DEPOSIT_MINTS_PER_MARKET } from "./constants";

export type ProgramErrorVariant =
  | "InvalidDiscriminator"
  | "AccountNotFound"
  | "InvalidDataLength"
  | "InvalidOutcomeCount"
  | "InvalidOutcomeIndex"
  | "TooManyMakers"
  | "SignatureVerificationFailed"
  | "InvalidSignature"
  | "Serialization"
  | "InvalidSide"
  | "InvalidMintOrder"
  | "InvalidMarketStatus"
  | "InvalidPendingRoleKind"
  | "MissingField"
  | "Overflow"
  | "OrderbookExists"
  | "InvalidMarket"
  | "MarketSettled"
  | "InvalidProgramId"
  | "InvalidOrderbook"
  | "FullFillRequired"
  | "DivisionByZero"
  | "Reserved50"
  | "InsufficientGlobalDeposit"
  | "InvalidDepositMintOrder"
  | "ZeroAmount"
  | "InvalidAta"
  | "OrderNotFullyFilled"
  | "InvalidPayoutNumerators"
  | "PayoutVectorExceedsU32"
  | "PayoutTooSmall"
  | "TokenAccountNotEmpty"
  | "LookupTableNotClosed"
  | "InvalidManager"
  | "InvalidFeeRange"
  | "InvalidFeeSum"
  | "InvalidFeeReceiver"
  | "InvalidOracle"
  | "LookupTableDeactivated"
  | "NoPendingRoleTransfer"
  | "PendingRoleMismatch"
  | "InvalidEventAuthority"
  | "EventBatchOverflow"
  | "InvalidEventBatch"
  | "InvalidEventContract"
  | "UnsupportedEventSchema"
  | "PublicInstructionMustBeTopLevel"
  | "LookupTableCapacityExceeded"
  | "TooManyDepositMints"
  | "InvalidScalarRange"
  | "DuplicateScalarOutcomes"
  | "InvalidPubkey"
  | "UnsignedOrder";

export class ProgramSdkError extends Error {
  readonly variant: ProgramErrorVariant;

  constructor(variant: ProgramErrorVariant, message: string) {
    super(message);
    this.name = "ProgramSdkError";
    this.variant = variant;
  }

  static invalidDiscriminator(
    accountType: string,
    expected: string,
    actual: string,
  ): ProgramSdkError {
    return new ProgramSdkError(
      "InvalidDiscriminator",
      `Invalid ${accountType} discriminator. Expected ${expected}, got ${actual}`,
    );
  }

  static accountNotFound(name: string): ProgramSdkError {
    return new ProgramSdkError("AccountNotFound", `${name} not found`);
  }

  static invalidDataLength(
    name: string,
    expected: number,
    actual: number,
  ): ProgramSdkError {
    return new ProgramSdkError(
      "InvalidDataLength",
      `Invalid ${name} data length: ${actual}, expected ${expected}`,
    );
  }

  static invalidOutcomeCount(count: number): ProgramSdkError {
    return new ProgramSdkError(
      "InvalidOutcomeCount",
      `Invalid number of outcomes: ${count}. Must be between 2 and 6.`,
    );
  }

  static invalidOutcomeIndex(index: number, max: number): ProgramSdkError {
    return new ProgramSdkError(
      "InvalidOutcomeIndex",
      `Invalid outcome index: ${index}. Must be between 0 and ${max}.`,
    );
  }

  static tooManyMakers(count: number): ProgramSdkError {
    return new ProgramSdkError(
      "TooManyMakers",
      `Too many makers: ${count}`,
    );
  }

  static signatureVerificationFailed(): ProgramSdkError {
    return new ProgramSdkError(
      "SignatureVerificationFailed",
      "Signature verification failed",
    );
  }

  static invalidSignature(): ProgramSdkError {
    return new ProgramSdkError("InvalidSignature", "Invalid signature");
  }

  static serialization(message: string): ProgramSdkError {
    return new ProgramSdkError("Serialization", message);
  }

  static invalidSide(value: number): ProgramSdkError {
    return new ProgramSdkError("InvalidSide", `Invalid side: ${value}`);
  }

  static invalidMintOrder(): ProgramSdkError {
    return new ProgramSdkError(
      "InvalidMintOrder",
      "Invalid mint order: orderbook mints must be distinct",
    );
  }

  static invalidMarketStatus(value: number): ProgramSdkError {
    return new ProgramSdkError(
      "InvalidMarketStatus",
      `Unknown market status: ${value}`,
    );
  }

  static invalidPendingRoleKind(value: number): ProgramSdkError {
    return new ProgramSdkError(
      "InvalidPendingRoleKind",
      `Invalid pending role kind: ${value}`,
    );
  }

  static missingField(field: string): ProgramSdkError {
    return new ProgramSdkError(
      "MissingField",
      `Missing required field: ${field}`,
    );
  }

  static overflow(message: string): ProgramSdkError {
    return new ProgramSdkError("Overflow", message);
  }

  static orderbookExists(): ProgramSdkError {
    return new ProgramSdkError("OrderbookExists", "Orderbook already exists");
  }

  static invalidMarket(): ProgramSdkError {
    return new ProgramSdkError("InvalidMarket", "Invalid market");
  }

  static marketSettled(): ProgramSdkError {
    return new ProgramSdkError("MarketSettled", "Market already settled");
  }

  static invalidProgramId(): ProgramSdkError {
    return new ProgramSdkError("InvalidProgramId", "Invalid program ID");
  }

  static invalidOrderbook(): ProgramSdkError {
    return new ProgramSdkError("InvalidOrderbook", "Invalid orderbook");
  }

  static fullFillRequired(): ProgramSdkError {
    return new ProgramSdkError("FullFillRequired", "Full fill required");
  }

  static divisionByZero(): ProgramSdkError {
    return new ProgramSdkError("DivisionByZero", "Division by zero");
  }

  static reserved50(): ProgramSdkError {
    return new ProgramSdkError(
      "Reserved50",
      "Reserved custom error code 50",
    );
  }

  static insufficientGlobalDeposit(): ProgramSdkError {
    return new ProgramSdkError(
      "InsufficientGlobalDeposit",
      "Insufficient global deposit balance",
    );
  }

  static invalidDepositMintOrder(): ProgramSdkError {
    return new ProgramSdkError(
      "InvalidDepositMintOrder",
      "Invalid deposit mint order",
    );
  }

  static zeroAmount(): ProgramSdkError {
    return new ProgramSdkError("ZeroAmount", "Amount must be greater than zero");
  }

  static invalidAta(): ProgramSdkError {
    return new ProgramSdkError(
      "InvalidAta",
      "Invalid associated token account",
    );
  }

  static orderNotFullyFilled(): ProgramSdkError {
    return new ProgramSdkError(
      "OrderNotFullyFilled",
      "Order status is not fully filled",
    );
  }

  static invalidPayoutNumerators(): ProgramSdkError {
    return new ProgramSdkError(
      "InvalidPayoutNumerators",
      "Payout numerators must include at least one non-zero value",
    );
  }

  static payoutVectorExceedsU32(): ProgramSdkError {
    return new ProgramSdkError(
      "PayoutVectorExceedsU32",
      "Payout numerators and denominator must fit in u32",
    );
  }

  static payoutTooSmall(): ProgramSdkError {
    return new ProgramSdkError("PayoutTooSmall", "Payout too small");
  }

  static tokenAccountNotEmpty(): ProgramSdkError {
    return new ProgramSdkError(
      "TokenAccountNotEmpty",
      "Token account is not empty",
    );
  }

  static lookupTableNotClosed(): ProgramSdkError {
    return new ProgramSdkError(
      "LookupTableNotClosed",
      "Lookup table is not closed",
    );
  }

  static invalidManager(): ProgramSdkError {
    return new ProgramSdkError("InvalidManager", "Invalid manager");
  }

  static invalidFeeRange(): ProgramSdkError {
    return new ProgramSdkError(
      "InvalidFeeRange",
      "Invalid fee range: maker and taker bps must each be between -500 and 500",
    );
  }

  static invalidFeeSum(): ProgramSdkError {
    return new ProgramSdkError(
      "InvalidFeeSum",
      "Invalid fee sum: maker + taker bps must be non-negative",
    );
  }

  static invalidFeeReceiver(): ProgramSdkError {
    return new ProgramSdkError("InvalidFeeReceiver", "Invalid fee receiver");
  }

  static invalidOracle(): ProgramSdkError {
    return new ProgramSdkError("InvalidOracle", "Invalid oracle");
  }

  static lookupTableDeactivated(): ProgramSdkError {
    return new ProgramSdkError(
      "LookupTableDeactivated",
      "Lookup table is deactivated",
    );
  }

  static noPendingRoleTransfer(): ProgramSdkError {
    return new ProgramSdkError(
      "NoPendingRoleTransfer",
      "No pending role transfer",
    );
  }

  static pendingRoleMismatch(): ProgramSdkError {
    return new ProgramSdkError(
      "PendingRoleMismatch",
      "Pending role transfer mismatch",
    );
  }

  /** On-chain error 68: event-authority trailer missing, writable, or wrong. */
  static invalidEventAuthority(): ProgramSdkError {
    return new ProgramSdkError(
      "InvalidEventAuthority",
      "Invalid event authority",
    );
  }

  /** On-chain error 69: the instruction's event batch exceeds the program buffer. */
  static eventBatchOverflow(): ProgramSdkError {
    return new ProgramSdkError("EventBatchOverflow", "Event batch overflow");
  }

  /** On-chain error 70: malformed event-batch self-CPI. */
  static invalidEventBatch(): ProgramSdkError {
    return new ProgramSdkError("InvalidEventBatch", "Invalid event batch");
  }

  /** On-chain error 71: the event-batch self-CPI violates the event contract. */
  static invalidEventContract(): ProgramSdkError {
    return new ProgramSdkError("InvalidEventContract", "Invalid event contract");
  }

  /** On-chain error 72: the event batch declares an unsupported schema. */
  static unsupportedEventSchema(): ProgramSdkError {
    return new ProgramSdkError(
      "UnsupportedEventSchema",
      "Unsupported event schema",
    );
  }

  /** On-chain error 73: a public instruction was invoked through another program's CPI. */
  static publicInstructionMustBeTopLevel(): ProgramSdkError {
    return new ProgramSdkError(
      "PublicInstructionMustBeTopLevel",
      "Public instruction must be top-level",
    );
  }

  /** On-chain error 74: extending the lookup table would exceed its capacity. */
  static lookupTableCapacityExceeded(): ProgramSdkError {
    return new ProgramSdkError(
      "LookupTableCapacityExceeded",
      "Lookup table capacity exceeded",
    );
  }

  /**
   * On-chain error 75, and the client-side check that more than
   * MAX_DEPOSIT_MINTS_PER_IX groups were supplied to one instruction.
   */
  static tooManyDepositMints(count: number): ProgramSdkError {
    return new ProgramSdkError(
      "TooManyDepositMints",
      `Too many deposit mints: ${count} (max ${MAX_DEPOSIT_MINTS_PER_MARKET})`,
    );
  }

  static invalidScalarRange(): ProgramSdkError {
    return new ProgramSdkError(
      "InvalidScalarRange",
      "Scalar maxValue must be greater than minValue",
    );
  }

  static duplicateScalarOutcomes(): ProgramSdkError {
    return new ProgramSdkError(
      "DuplicateScalarOutcomes",
      "Scalar lower and upper outcome indexes must be distinct",
    );
  }

  static invalidPubkey(message: string): ProgramSdkError {
    return new ProgramSdkError("InvalidPubkey", message);
  }

  static unsignedOrder(): ProgramSdkError {
    return new ProgramSdkError(
      "UnsignedOrder",
      "Order must be signed before converting to submit request",
    );
  }
}

export type ProgramResult<T> = Promise<T> | T;
