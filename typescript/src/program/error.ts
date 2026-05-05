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
  | "MissingField"
  | "Overflow"
  | "DivisionByZero"
  | "InvalidPayoutNumerators"
  | "PayoutVectorExceedsU32"
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

  static missingField(field: string): ProgramSdkError {
    return new ProgramSdkError(
      "MissingField",
      `Missing required field: ${field}`,
    );
  }

  static overflow(message: string): ProgramSdkError {
    return new ProgramSdkError("Overflow", message);
  }

  static divisionByZero(): ProgramSdkError {
    return new ProgramSdkError("DivisionByZero", "Division by zero");
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
