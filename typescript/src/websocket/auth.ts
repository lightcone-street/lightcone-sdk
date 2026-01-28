/**
 * Authentication module for Lightcone WebSocket.
 *
 * Provides functionality for authenticating with the Lightcone API
 * to access private user streams (orders, balances, fills).
 *
 * # Authentication Flow
 *
 * 1. Generate a sign-in message with timestamp
 * 2. Sign the message with an Ed25519 keypair
 * 3. POST to the authentication endpoint
 * 4. Extract `auth_token` from response cookie
 * 5. Connect to WebSocket with the auth token
 */

import { Keypair } from "@solana/web3.js";
import bs58 from "bs58";
import nacl from "tweetnacl";
import { WebSocketError } from "./error";

/** Authentication API base URL */
export const AUTH_API_URL = "https://tapi.lightcone.xyz/api";

/** Authentication timeout in milliseconds */
const AUTH_TIMEOUT_MS = 10000;

/**
 * Authentication credentials returned after successful login.
 */
export interface AuthCredentials {
  /** The authentication token to use for WebSocket connection */
  authToken: string;
  /** The user's public key (Base58 encoded) */
  userPubkey: string;
}

/**
 * Request body for login endpoint.
 */
interface LoginRequest {
  /** Base58 encoded public key */
  public_key: string;
  /** The message that was signed */
  message: string;
  /** Base58 encoded signature */
  signature: string;
}

/**
 * Response from login endpoint.
 */
interface LoginResponse {
  /** Whether the login was successful */
  success: boolean;
  /** Error message if login failed */
  error?: string;
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
 * import { authenticateWithKeypair } from "@lightcone/sdk/websocket";
 *
 * const keypair = Keypair.generate();
 * const credentials = await authenticateWithKeypair(keypair);
 * console.log("Auth token:", credentials.authToken);
 * ```
 */
export async function authenticateWithKeypair(
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
    public_key: publicKeyB58,
    message,
    signature: signatureB58,
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
      credentials: "include",
      signal: controller.signal,
    });
  } catch (error) {
    clearTimeout(timeoutId);
    if (error instanceof Error && error.name === "AbortError") {
      throw WebSocketError.authenticationFailed(
        "Authentication request timed out"
      );
    }
    throw error;
  } finally {
    clearTimeout(timeoutId);
  }

  // Check for HTTP errors
  if (!response.ok) {
    throw WebSocketError.authenticationFailed(`HTTP error: ${response.status}`);
  }

  // Extract auth_token from cookies
  const cookies = response.headers.get("set-cookie");
  let authToken: string | undefined;
  if (cookies) {
    const match = cookies.match(/auth_token=([^;]+)/);
    if (match && match[1]?.trim().length > 0) {
      authToken = decodeURIComponent(match[1].trim());
    }
  }

  if (!authToken) {
    throw WebSocketError.authenticationFailed(
      "No auth_token cookie in response"
    );
  }

  // Parse the response body
  const loginResponse = (await response.json()) as LoginResponse;

  if (!loginResponse.success) {
    throw WebSocketError.authenticationFailed(
      loginResponse.error || "Unknown error"
    );
  }

  return {
    authToken,
    userPubkey: publicKeyB58,
  };
}

/**
 * Authenticate with Lightcone and obtain credentials using a secret key.
 *
 * @param secretKey - The Ed25519 secret key (64 bytes)
 * @returns AuthCredentials containing the auth token and user public key
 *
 * @example
 * ```typescript
 * import { authenticate } from "@lightcone/sdk/websocket";
 *
 * const secretKey = new Uint8Array(64); // Your secret key
 * const credentials = await authenticate(secretKey);
 * console.log("Auth token:", credentials.authToken);
 * ```
 */
export async function authenticate(
  secretKey: Uint8Array
): Promise<AuthCredentials> {
  const keypair = Keypair.fromSecretKey(secretKey);
  return authenticateWithKeypair(keypair);
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
