import { SdkError } from "../error";
import { RetryPolicy, type LightconeHttp } from "../http";
import { asPubkeyStr } from "../shared";
import { tradingWallet } from "./index";
import type { AuthCredentials, LoginRequest, NonceResponse, SessionResponse } from "./index";

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

    const session = await this.client.http.post<SessionResponse, LoginRequest>(
      url,
      body,
      RetryPolicy.None
    );

    this.client.authState.setCredentials(credentialsFromSession(session));

    return session;
  }

  async checkSession(): Promise<SessionResponse> {
    const url = `${this.client.http.baseUrl()}/api/auth/me`;

    let session: SessionResponse;
    try {
      session = await this.client.http.get<SessionResponse>(url, RetryPolicy.Idempotent);
    } catch (error) {
      this.client.authState.setCredentials(undefined);
      throw SdkError.from(error);
    }

    this.client.authState.setCredentials(credentialsFromSession(session));

    return session;
  }

  async logout(): Promise<void> {
    const url = `${this.client.http.baseUrl()}/api/auth/logout`;
    try {
      await this.client.http.post<{ success: boolean }, Record<string, never>>(url, {}, RetryPolicy.None);
    } catch {
      // Backend cookie clear can fail in local/dev setups; still clear local state.
    }

    await this.client.http.clearAuthToken();
    this.client.authState.setCredentials(undefined);
    await this.client.authState.clearCaches();
  }

  async disconnectX(): Promise<void> {
    const url = `${this.client.http.baseUrl()}/api/auth/disconnect_x`;
    await this.client.http.post<{ success: boolean }, Record<string, never>>(url, {}, RetryPolicy.None);
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
