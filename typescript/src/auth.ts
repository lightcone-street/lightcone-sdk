/**
 * Authentication module for Lightcone.
 *
 * Provides functionality for authenticating with the Lightcone API.
 * Used by both the REST API client and WebSocket client.
 *
 * # Authentication Flow
 *
 * 1. Generate a sign-in message with timestamp
 * 2. Sign the message with an Ed25519 keypair
 * 3. POST to the authentication endpoint
 * 4. Extract token from JSON response
 */

import { Keypair } from "@solana/web3.js";
import bs58 from "bs58";
import nacl from "tweetnacl";

/** Authentication API base URL */
export const AUTH_API_URL = "https://tapi.lightcone.xyz/api";

/** Authentication timeout in milliseconds */
const AUTH_TIMEOUT_MS = 10000;

/**
 * Auth error variants (mirrors Rust's AuthError enum).
 */
export type AuthErrorVariant =
  | "SystemTime"
  | "HttpError"
  | "AuthenticationFailed";

/**
 * Authentication error class.
 */
export class AuthError extends Error {
  readonly variant: AuthErrorVariant;

  constructor(variant: AuthErrorVariant, message: string) {
    super(message);
    this.name = "AuthError";
    this.variant = variant;
  }

  static systemTime(message: string): AuthError {
    return new AuthError("SystemTime", `System time error: ${message}`);
  }

  static httpError(message: string): AuthError {
    return new AuthError("HttpError", `HTTP error: ${message}`);
  }

  static authenticationFailed(message: string): AuthError {
    return new AuthError("AuthenticationFailed", `Authentication failed: ${message}`);
  }
}

/**
 * Authentication credentials returned after successful login.
 */
export interface AuthCredentials {
  /** The authentication token to use for WebSocket connection */
  authToken: string;
  /** The user's public key (Base58 encoded) */
  userPubkey: string;
  /** The user's ID */
  userId: string;
  /** Token expiration timestamp (Unix seconds) */
  expiresAt: number;
}

/**
 * Request body for login endpoint.
 */
interface LoginRequest {
  /** Raw 32-byte public key as array */
  pubkey_bytes: number[];
  /** The message that was signed */
  message: string;
  /** Base58 encoded signature */
  signature_bs58: string;
}

/**
 * Response from login endpoint.
 */
interface LoginResponse {
  /** The authentication token */
  token: string;
  /** The user's ID */
  user_id: string;
  /** The user's wallet address */
  wallet_address: string;
  /** Token expiration timestamp (Unix seconds) */
  expires_at: number;
}

/**
 * Generate the sign-in message with current timestamp.
 *
 * @returns The message to be signed
 */
export function generateSigninMessage(): string {
  const timestampMs = Date.now();
  return `Sign in to Lightcone\n\nTimestamp: ${timestampMs}`;
}

/**
 * Generate the sign-in message with a specific timestamp.
 *
 * @param timestampMs - Unix timestamp in milliseconds
 * @returns The message to be signed
 */
export function generateSigninMessageWithTimestamp(timestampMs: number): string {
  return `Sign in to Lightcone\n\nTimestamp: ${timestampMs}`;
}

/**
 * Authenticate with Lightcone and obtain credentials using a Solana Keypair.
 *
 * @param keypair - The Solana Keypair for authentication
 * @returns AuthCredentials containing the auth token and user public key
 *
 * @example
 * ```typescript
 * import { Keypair } from "@solana/web3.js";
 * import { auth } from "@lightcone/sdk";
 *
 * const keypair = Keypair.generate();
 * const credentials = await auth.authenticate(keypair);
 * console.log("Auth token:", credentials.authToken);
 * ```
 */
export async function authenticate(
  keypair: Keypair
): Promise<AuthCredentials> {
  // Generate the message
  const message = generateSigninMessage();

  // Sign the message
  const messageBytes = new TextEncoder().encode(message);
  const signature = nacl.sign.detached(messageBytes, keypair.secretKey);
  const signatureB58 = bs58.encode(signature);

  // Get the public key
  const publicKeyB58 = keypair.publicKey.toBase58();

  // Create the request body
  const request: LoginRequest = {
    pubkey_bytes: Array.from(keypair.publicKey.toBytes()),
    message,
    signature_bs58: signatureB58,
  };

  // Send the authentication request with timeout
  const url = `${AUTH_API_URL}/auth/login_or_register_with_message`;
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), AUTH_TIMEOUT_MS);

  let response: Response;
  try {
    response = await fetch(url, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify(request),
      signal: controller.signal,
    });
  } catch (error) {
    clearTimeout(timeoutId);
    if (error instanceof Error && error.name === "AbortError") {
      throw AuthError.authenticationFailed(
        "Authentication request timed out"
      );
    }
    throw AuthError.httpError(
      error instanceof Error ? error.message : String(error)
    );
  } finally {
    clearTimeout(timeoutId);
  }

  // Check for HTTP errors
  if (!response.ok) {
    throw AuthError.httpError(`HTTP ${response.status}`);
  }

  // Parse the response body
  const loginResponse = (await response.json()) as LoginResponse;

  return {
    authToken: loginResponse.token,
    userPubkey: publicKeyB58,
    userId: loginResponse.user_id,
    expiresAt: loginResponse.expires_at,
  };
}


/**
 * Sign a message with a Solana Keypair.
 *
 * @param message - The message to sign
 * @param keypair - The Solana Keypair
 * @returns The Base58-encoded signature
 */
export function signMessage(message: string, keypair: Keypair): string {
  const messageBytes = new TextEncoder().encode(message);
  const signature = nacl.sign.detached(messageBytes, keypair.secretKey);
  return bs58.encode(signature);
}
