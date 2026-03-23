import { MAX_MAKERS } from "./constants";

export class ProgramSdkError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "ProgramSdkError";
  }

  static missingField(field: string): ProgramSdkError {
    return new ProgramSdkError(`Missing required field: ${field}`);
  }

  static invalidSignature(): ProgramSdkError {
    return new ProgramSdkError("Invalid signature");
  }

  static unsignedOrder(): ProgramSdkError {
    return new ProgramSdkError("Order must be signed before converting to submit request");
  }

  static invalidDiscriminator(expected: string, actual: string): ProgramSdkError {
    return new ProgramSdkError(`Invalid discriminator for ${expected}: got ${actual}`);
  }

  static accountNotFound(name: string): ProgramSdkError {
    return new ProgramSdkError(`Account not found: ${name}`);
  }

  static invalidDataLength(expected: number, actual: number): ProgramSdkError {
    return new ProgramSdkError(`Invalid data length: expected ${expected}, got ${actual}`);
  }

  static invalidOutcomeCount(count: number): ProgramSdkError {
    return new ProgramSdkError(`Invalid outcome count: ${count}`);
  }

  static invalidOutcomeIndex(index: number, max: number): ProgramSdkError {
    return new ProgramSdkError(`Invalid outcome index: ${index} (max ${max})`);
  }

  static invalidSide(value: number): ProgramSdkError {
    return new ProgramSdkError(`Invalid side: ${value}`);
  }

  static invalidMarketStatus(value: number): ProgramSdkError {
    return new ProgramSdkError(`Invalid market status: ${value}`);
  }

  static serialization(message: string): ProgramSdkError {
    return new ProgramSdkError(`Serialization error: ${message}`);
  }

  static overflow(context?: string): ProgramSdkError {
    return new ProgramSdkError(context ? `Overflow: ${context}` : "Overflow");
  }

  static invalidPubkey(message: string): ProgramSdkError {
    return new ProgramSdkError(`Invalid pubkey: ${message}`);
  }

  static tooManyMakers(count: number): ProgramSdkError {
    return new ProgramSdkError(`Too many makers: ${count} (max ${MAX_MAKERS})`);
  }

  static signatureVerificationFailed(): ProgramSdkError {
    return new ProgramSdkError("Signature verification failed");
  }

  static invalidMintOrder(): ProgramSdkError {
    return new ProgramSdkError("Invalid mint order");
  }

  static orderbookExists(): ProgramSdkError {
    return new ProgramSdkError("Orderbook already exists");
  }

  static invalidMarket(): ProgramSdkError {
    return new ProgramSdkError("Invalid market");
  }

  static marketSettled(): ProgramSdkError {
    return new ProgramSdkError("Market already settled");
  }

  static invalidProgramId(): ProgramSdkError {
    return new ProgramSdkError("Invalid program ID");
  }

  static invalidOrderbook(): ProgramSdkError {
    return new ProgramSdkError("Invalid orderbook");
  }

  static fullFillRequired(): ProgramSdkError {
    return new ProgramSdkError("Full fill required");
  }

  static divisionByZero(): ProgramSdkError {
    return new ProgramSdkError("Division by zero");
  }

  static depositTokenNotActive(): ProgramSdkError {
    return new ProgramSdkError("Deposit token not active");
  }

  static scaling(message: string): ProgramSdkError {
    return new ProgramSdkError(`Scaling error: ${message}`);
  }
}

export type ProgramResult<T> = Promise<T> | T;
