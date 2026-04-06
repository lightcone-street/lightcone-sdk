import { SdkError } from "../error";
import { RetryPolicy, type LightconeHttp } from "../http";
import { asPubkeyStr } from "../shared";
import type { AuthCredentials, LoginRequest, LoginResponse, MeResponse, NonceResponse, User } from "./index";

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
  ): Promise<User> {
    const url = `${this.client.http.baseUrl()}/api/auth/login_or_register_with_message`;
    const body: LoginRequest = {
      message,
      signature_bs58: signatureBs58,
      pubkey_bytes: Array.from(pubkeyBytes),
      use_embedded_wallet: useEmbeddedWallet,
    };

    const response = await this.client.http.post<LoginResponse, LoginRequest>(
      url,
      body,
      RetryPolicy.None
    );

    const credentials: AuthCredentials = {
      user_id: response.user_id,
      wallet_address: asPubkeyStr(response.wallet_address),
      expires_at: parseExpiry(response.expires_at),
    };
    this.client.authState.setCredentials(credentials);

    return {
      id: response.user_id,
      wallet_address: response.wallet_address,
      linked_account: response.linked_account,
      privy_id: response.privy_id,
      embedded_wallet: response.embedded_wallet,
      x_username: response.x_username,
      x_user_id: response.x_user_id,
      x_display_name: response.x_display_name,
      google_email: response.google_email,
    };
  }

  async checkSession(): Promise<User> {
    const url = `${this.client.http.baseUrl()}/api/auth/me`;

    let response: MeResponse;
    try {
      response = await this.client.http.get<MeResponse>(url, RetryPolicy.Idempotent);
    } catch (error) {
      this.client.authState.setCredentials(undefined);
      throw SdkError.from(error);
    }

    this.client.authState.setCredentials({
      user_id: response.user_id,
      wallet_address: asPubkeyStr(response.wallet_address),
      expires_at: parseExpiry(response.expires_at),
    });

    return {
      id: response.user_id,
      wallet_address: response.wallet_address,
      linked_account: response.linked_account,
      privy_id: response.privy_id,
      embedded_wallet: response.embedded_wallet,
      x_username: response.x_username,
      x_user_id: response.x_user_id,
      x_display_name: response.x_display_name,
      google_email: response.google_email,
    };
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

function parseExpiry(timestamp: number): Date {
  if (timestamp > 1_000_000_000_000) {
    return new Date(timestamp);
  }
  return new Date(timestamp * 1000);
}
