/**
 * Input validation utilities for the Lightcone REST API.
 */

import { PublicKey } from "@solana/web3.js";
import { ApiError } from "./error";

/** Maximum allowed pagination limit */
export const MAX_PAGINATION_LIMIT = 500;

/** Default request timeout in milliseconds */
export const DEFAULT_TIMEOUT_MS = 30000;

/**
 * Validate a Solana public key (Base58-encoded).
 * Uses @solana/web3.js PublicKey for proper validation.
 * @throws {ApiError} If the value is not a valid Solana pubkey
 */
export function validatePubkey(value: string, fieldName: string): void {
  if (!value || value.length === 0) {
    throw ApiError.invalidParameter(`${fieldName} cannot be empty`);
  }
  try {
    new PublicKey(value);
  } catch {
    throw ApiError.invalidParameter(`${fieldName} is not a valid Solana public key`);
  }
}

/**
 * Validate an Ed25519 signature (128 hex characters).
 * @throws {ApiError} If the signature is invalid
 */
export function validateSignature(signature: string): void {
  if (signature.length !== 128) {
    throw ApiError.invalidParameter(
      `Signature must be 128 hex characters, got ${signature.length}`
    );
  }
  if (!/^[0-9a-fA-F]+$/.test(signature)) {
    throw ApiError.invalidParameter("Signature must contain only hex characters");
  }
}

/**
 * Validate pagination limit (1-500).
 * @throws {ApiError} If the limit is out of bounds
 */
export function validateLimit(limit: number | undefined): void {
  if (limit !== undefined && (limit < 1 || limit > MAX_PAGINATION_LIMIT)) {
    throw ApiError.invalidParameter(`Limit must be 1-${MAX_PAGINATION_LIMIT}`);
  }
}
