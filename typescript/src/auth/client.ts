import { SdkError, isUnauthorized } from "../error";
import { RetryPolicy, type LightconeHttp } from "../http";
import { asPubkeyStr } from "../shared";
import { tradingWallet } from "./index";
import type {
  AuthCredentials,
  LoginRequest,
  MaxSlippagePreferenceBody,
  NonceResponse,
  SessionResponse,
} from "./index";

interface AuthState {
  getCredentials(): AuthCredentials | undefined;
  setCredentials(credentials: AuthCredentials | undefined): void;
  clearCaches(): Promise<void>;
}

interface ClientContext {
  http: LightconeHttp;
  authState: AuthState;
}

export class Auth {
  constructor(private readonly client: ClientContext) {}

  async getNonce(): Promise<string> {
    const url = `${this.client.http.baseUrl()}/api/auth/nonce`;
    const response = await this.client.http.get<NonceResponse>(url, RetryPolicy.None);
    return response.nonce;
  }

  async loginWithMessage(
    message: string,
    signatureBs58: string,
    pubkeyBytes: Uint8Array,
    useEmbeddedWallet?: boolean
  ): Promise<SessionResponse> {
    const url = `${this.client.http.baseUrl()}/api/auth/login_or_register_with_message`;
    const body: LoginRequest = {
      message,
      signature_bs58: signatureBs58,
      pubkey_bytes: Array.from(pubkeyBytes),
      use_embedded_wallet: useEmbeddedWallet,
    };

    // Credential-management endpoint: opts out of the transport's 401
    // restore-and-replay. The backend consumes the login nonce before
    // verifying the signature, so a replayed login deterministically fails —
    // and restoring credentials in order to log in is circular.
    const session = await this.client.http.postWithoutCredentialRestore<
      SessionResponse,
      LoginRequest
    >(url, body, RetryPolicy.None);
    normalizeSessionMaxSlippagePreference(session);

    this.client.authState.setCredentials(credentialsFromSession(session));

    return session;
  }

  async checkSession(): Promise<SessionResponse> {
    const url = `${this.client.http.baseUrl()}/api/auth/me`;

    let session: SessionResponse;
    try {
      session = await this.client.http.get<SessionResponse>(url, RetryPolicy.Idempotent);
      normalizeSessionMaxSlippagePreference(session);
    } catch (error) {
      this.client.authState.setCredentials(undefined);
      throw SdkError.from(error);
    }

    this.client.authState.setCredentials(credentialsFromSession(session));

    return session;
  }

  /**
   * Logout — clears the server-side cookie, internal token, and credentials.
   *
   * Local state is cleared even when the server call fails — the caller asked
   * to be signed out locally regardless — but the failure is then rethrown:
   * callers gating security decisions on teardown (e.g. whether an app may
   * restart an authenticated transport) must be able to see that the
   * server-side cookie may still be valid. A 401 counts as success: it means
   * "already logged out".
   */
  async logout(): Promise<void> {
    const url = `${this.client.http.baseUrl()}/api/auth/logout`;
    let logoutError: unknown = null;
    try {
      // Credential-management endpoint: opts out of the transport's 401
      // restore-and-replay — a 401 here means "already logged out".
      await this.client.http.postWithoutCredentialRestore<
        { success: boolean },
        Record<string, never>
      >(url, {}, RetryPolicy.None);
    } catch (error) {
      if (!isUnauthorized(error)) {
        logoutError = error;
      }
    }

    await this.client.http.clearAuthToken();
    this.client.authState.setCredentials(undefined);
    await this.client.authState.clearCaches();

    if (logoutError !== null) {
      throw logoutError;
    }
  }

  async disconnectX(): Promise<void> {
    const url = `${this.client.http.baseUrl()}/api/auth/disconnect_x`;
    await this.client.http.post<{ success: boolean }, Record<string, never>>(url, {}, RetryPolicy.None);
  }

  /** Persist an account-wide max-slippage preference strictly below 10%. */
  async updateMaxSlippagePreference(maxSlippagePreference: string): Promise<string> {
    const url = `${this.client.http.baseUrl()}/api/auth/max_slippage_preference`;
    const response = await this.client.http.post<
      MaxSlippagePreferenceBody,
      MaxSlippagePreferenceBody
    >(
      url,
      { max_slippage_preference: maxSlippagePreference },
      RetryPolicy.Idempotent
    );
    const preference = decodeMaxSlippagePreference(response, false, "update response");
    if (preference === null) {
      throw SdkError.serde("Max-slippage update response must contain a decimal string");
    }
    return preference;
  }

  connectXUrl(): string {
    return `${this.client.http.baseUrl()}/api/auth/oauth/link/x`;
  }

  credentials(): AuthCredentials | undefined {
    return this.client.authState.getCredentials();
  }

  isAuthenticated(): boolean {
    const credentials = this.credentials();
    if (!credentials) {
      return false;
    }
    return Date.now() < credentials.expires_at.getTime();
  }
}

function normalizeSessionMaxSlippagePreference(session: SessionResponse): void {
  const preference = decodeMaxSlippagePreference(
    session?.user,
    true,
    "session user",
    true
  );
  session.user.max_slippage_preference = preference;
}

/** Enforces the exact nullable/string JSON contract erased by TypeScript types. */
function decodeMaxSlippagePreference(
  payload: unknown,
  allowNull: boolean,
  context: string,
  allowMissing = false
): string | null {
  if (typeof payload !== "object" || payload === null) {
    throw SdkError.serde(`Max-slippage ${context} is malformed`);
  }
  if (!Object.prototype.hasOwnProperty.call(payload, "max_slippage_preference")) {
    if (allowMissing) {
      return null;
    }
    throw SdkError.serde(`Max-slippage ${context} is missing max_slippage_preference`);
  }

  const preference = (payload as Record<string, unknown>).max_slippage_preference;
  if (typeof preference === "string") {
    return preference;
  }
  if (allowNull && preference === null) {
    return null;
  }
  throw SdkError.serde(
    `Max-slippage ${context} must contain ${allowNull ? "a string or null" : "a string"}`
  );
}

/**
 * Derive session credentials from the envelope. The trading wallet comes from
 * the identity + auth method.
 */
function credentialsFromSession(session: SessionResponse): AuthCredentials {
  return {
    user_id: session.user.user_id,
    wallet_address: asPubkeyStr(tradingWallet(session.user, session.auth_method)),
    expires_at: parseExpiry(session.expires_at),
  };
}

function parseExpiry(timestamp: number): Date {
  if (timestamp > 1_000_000_000_000) {
    return new Date(timestamp);
  }
  return new Date(timestamp * 1000);
}
