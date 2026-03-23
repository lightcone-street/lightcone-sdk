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
}

export type ProgramResult<T> = Promise<T> | T;
