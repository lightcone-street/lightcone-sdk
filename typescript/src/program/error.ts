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
  | "InvalidMarketStatus"
  | "MissingField"
  | "Overflow"
  | "DivisionByZero"
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
