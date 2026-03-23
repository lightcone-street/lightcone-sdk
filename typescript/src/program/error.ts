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
}

export type ProgramResult<T> = Promise<T> | T;
