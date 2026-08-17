import { Connection, Keypair, PublicKey, Transaction } from "@solana/web3.js";
import { Auth, type AuthCredentials } from "./auth";
import type { ClientContext } from "./context";
import {
  signAndSubmitTx as signAndSubmitTxFn,
  signAndSubmitTxConfirmed as signAndSubmitTxConfirmedFn,
  signAndSubmitTxConfirmedWithSlot as signAndSubmitTxConfirmedWithSlotFn,
} from "./context";
import type { ConfirmedTransaction } from "./context";
import type { FaucetRequest, FaucetResponse } from "./domain/faucet";
import { Markets } from "./domain/market";
import { Metrics } from "./domain/metrics";
import { Notifications } from "./domain/notification";
import { Orders } from "./domain/order";
import { Orderbooks } from "./domain/orderbook";
import { Positions } from "./domain/position";
import { PriceHistoryClient } from "./domain/price_history";
import { Referrals } from "./domain/referral";
import { Trades } from "./domain/trade";
import { LightconeHttp, RetryPolicy, type CredentialRestorer } from "./http";
import { LightconeEnv, apiUrl, wsUrl, rpcUrl, programId as envProgramId } from "./env";
import { Privy } from "./privy";
import { Rpc } from "./rpc";
import { RpcFailoverState } from "./rpcFailover";
import { DepositSource, type OrderbookRules, type PubkeyStr } from "./shared";
import { type ExternalSigner, type SigningStrategy } from "./shared/signing";
import { WsClient, type WsConfig } from "./ws";

class AuthState {
  private credentialsValue: AuthCredentials | undefined;

  constructor(initial?: AuthCredentials) {
    this.credentialsValue = initial;
  }

  getCredentials(): AuthCredentials | undefined {
    return this.credentialsValue;
  }

  setCredentials(credentials: AuthCredentials | undefined): void {
    this.credentialsValue = credentials;
  }

  async clearCaches(): Promise<void> {
    // No caches to clear — decimals are derived locally from orderbook metadata.
  }
}

export class LightconeClient implements ClientContext {
  readonly http: LightconeHttp;
  readonly programId: PublicKey;
  readonly primaryConnection?: Connection;
  readonly backupConnection?: Connection;
  readonly rpcFailoverState: RpcFailoverState;
  private depositSourceValue: DepositSource;
  private signingStrategyValue?: SigningStrategy;
  private orderNonceValue: number | undefined;
  private readonly wsConfigValue: WsConfig;
  private readonly authStateStore: AuthState;
  readonly orderbookRulesCache: Map<string, Promise<OrderbookRules>>;

  /** @deprecated Use primaryConnection — kept for ClientContext compat. */
  get connection(): Connection | undefined {
    return this.primaryConnection;
  }

  constructor(params: {
    http: LightconeHttp;
    wsConfig: WsConfig;
    programId?: PublicKey;
    primaryConnection?: Connection;
    backupConnection?: Connection;
    rpcFailoverState?: RpcFailoverState;
    depositSource?: DepositSource;
    signingStrategy?: SigningStrategy;
    orderNonce?: number;
    authCredentials?: AuthCredentials;
    authState?: AuthState;
    orderbookRulesCache?: Map<string, Promise<OrderbookRules>>;
  }) {
    this.http = params.http;
    this.programId = params.programId ?? envProgramId(LightconeEnv.Prod);
    this.primaryConnection = params.primaryConnection;
    this.backupConnection = params.backupConnection;
    this.rpcFailoverState =
      params.rpcFailoverState ?? new RpcFailoverState();
    this.depositSourceValue = params.depositSource ?? DepositSource.Global;
    this.signingStrategyValue = params.signingStrategy;
    this.orderNonceValue = params.orderNonce;
    this.wsConfigValue = params.wsConfig;
    this.authStateStore =
      params.authState ??
      new AuthState(params.authCredentials);
    this.orderbookRulesCache = params.orderbookRulesCache ?? new Map();
  }

  // ── Deposit source ──────────────────────────────────────────────────

  get depositSource(): DepositSource {
    return this.depositSourceValue;
  }

  setDepositSource(source: DepositSource): void {
    this.depositSourceValue = source;
  }

  // ── Signing strategy ────────────────────────────────────────────────

  get signingStrategy(): SigningStrategy | undefined {
    return this.signingStrategyValue;
  }

  /**
   * Cached authenticated identity shared by domain sub-clients.
   *
   * This is distinct from the HTTP cookie token and may be expired; fund-moving
   * operations must re-check its lifetime and wallet before signing.
   */
  get authCredentials(): AuthCredentials | undefined {
    return this.authStateStore.getCredentials();
  }

  setSigningStrategy(strategy: SigningStrategy): void {
    this.signingStrategyValue = strategy;
  }

  clearSigningStrategy(): void {
    this.signingStrategyValue = undefined;
  }

  // ── Nonce cache ──────────────────────────────────────────────────────

  orderNonce(): number | undefined {
    return this.orderNonceValue;
  }

  setOrderNonce(nonce: number): void {
    this.orderNonceValue = nonce;
  }

  clearOrderNonce(): void {
    this.orderNonceValue = undefined;
  }

  // ── Auth token (cookie) ─────────────────────────────────────────────

  /**
   * Get the current `lightcone-token` cookie value, if any. Populated by the SDK
   * after a successful login, then attached on every authed request. Useful
   * for forwarding the token through the `*WithCookies` methods, or
   * persisting the session across processes.
   */
  async authToken(): Promise<string | undefined> {
    return this.http.authTokenRef()();
  }

  /**
   * Clear the cached `lightcone-token`. Subsequent authed calls will go out
   * without a `Cookie` header (and 401) unless they use a
   * `*WithCookies` variant.
   */
  async clearAuthToken(): Promise<void> {
    await this.http.clearAuthToken();
  }

  /**
   * Register the credential restorer consulted when a request fails with
   * HTTP 401: it attempts to restore credentials (e.g. refresh the app's
   * auth session so the auth cookie is valid again); on success the
   * transport replays the request once IF it declared itself retry-safe
   * (RetryPolicy.None mutations are never auto-replayed). See {@link CredentialRestorer}.
   * Without a restorer, 401s propagate to callers unchanged.
   *
   * Common use: set once at app startup, alongside the signing strategy.
   */
  setCredentialRestorer(restorer: CredentialRestorer): void {
    this.http.setCredentialRestorer(restorer);
  }

  /** Remove the credential restorer (e.g. in tests); 401s propagate again. */
  clearCredentialRestorer(): void {
    this.http.clearCredentialRestorer();
  }

  // ── Transaction signing + submission ────────────────────────────────

  async signAndSubmitTx(tx: Transaction): Promise<string> {
    return signAndSubmitTxFn(this, tx);
  }

  /**
   * Sign and submit a transaction, then wait until it reaches `confirmed`
   * commitment on-chain. Prefer this over {@link signAndSubmitTx} when a
   * follow-up transaction depends on this one's state.
   */
  async signAndSubmitTxConfirmed(tx: Transaction): Promise<string> {
    return signAndSubmitTxConfirmedFn(this, tx);
  }

  /** Sign, submit, confirm, and return the transaction's processing slot. */
  async signAndSubmitTxConfirmedWithSlot(
    tx: Transaction
  ): Promise<ConfirmedTransaction> {
    return signAndSubmitTxConfirmedWithSlotFn(this, tx);
  }

  static builder(): LightconeClientBuilder {
    return new LightconeClientBuilder();
  }

  // ── Sub-client accessors ─────────────────────────────────────────────

  markets(): Markets {
    return new Markets(this);
  }

  orderbooks(): Orderbooks {
    return new Orderbooks(this);
  }

  orders(): Orders {
    return new Orders(this);
  }

  positions(): Positions {
    return new Positions(this);
  }

  trades(): Trades {
    return new Trades(this);
  }

  priceHistory(): PriceHistoryClient {
    return new PriceHistoryClient(this);
  }

  notifications(): Notifications {
    return new Notifications(this);
  }

  /**
   * Metrics sub-client — platform / market / orderbook / category / deposit-token
   * volume metrics, market leaderboard, and time-series history.
   */
  metrics(): Metrics {
    return new Metrics(this);
  }

  /**
   * Request testnet SOL and whitelisted deposit tokens for a wallet.
   *
   * Only active on environments whose backend has the faucet enabled (typically
   * local and staging).
   *
   * `POST /api/claim`
   */
  async claim(walletAddress: PubkeyStr): Promise<FaucetResponse> {
    const url = `${this.http.baseUrl()}/api/claim`;
    return this.http.post<FaucetResponse, FaucetRequest>(
      url,
      { wallet_address: walletAddress },
      RetryPolicy.None
    );
  }

  auth(): Auth {
    return new Auth({
      http: this.http,
      authState: this.authStateStore,
    });
  }

  privy(): Privy {
    return new Privy(this);
  }

  referrals(): Referrals {
    return new Referrals(this);
  }

  rpc(): Rpc {
    return new Rpc(this);
  }

  wsConfig(): WsConfig {
    return this.wsConfigValue;
  }

  ws(): WsClient {
    return new WsClient(this.wsConfigValue, this.http.authTokenRef());
  }

  clone(): LightconeClient {
    return new LightconeClient({
      http: this.http,
      wsConfig: { ...this.wsConfigValue },
      programId: this.programId,
      primaryConnection: this.primaryConnection
        ? new Connection(this.primaryConnection.rpcEndpoint, { commitment: "confirmed" })
        : undefined,
      backupConnection: this.backupConnection
        ? new Connection(this.backupConnection.rpcEndpoint, { commitment: "confirmed" })
        : undefined,
      rpcFailoverState: this.rpcFailoverState,
      depositSource: this.depositSourceValue,
      signingStrategy: this.signingStrategyValue,
      orderNonce: this.orderNonceValue,
      authState: this.authStateStore,
      orderbookRulesCache: this.orderbookRulesCache,
    });
  }
}

export class LightconeClientBuilder {
  private baseUrlValue: string = apiUrl(LightconeEnv.Prod);
  private wsUrlValue: string = wsUrl(LightconeEnv.Prod);
  private authCredentials?: AuthCredentials;
  private programIdValue: PublicKey = envProgramId(LightconeEnv.Prod);
  private depositSourceValue: DepositSource = DepositSource.Global;
  private signingStrategyValue?: SigningStrategy;
  private primaryRpcUrlValue?: string = rpcUrl(LightconeEnv.Prod);
  private backupRpcUrlValue?: string;

  /**
   * Set the deployment environment. Configures the API URL, WebSocket URL,
   * RPC URL, and program ID for the given environment.
   *
   * Individual URL overrides (e.g. `.baseUrl()`) take precedence when
   * called **after** `.env()`.
   */
  env(environment: LightconeEnv): LightconeClientBuilder {
    this.baseUrlValue = apiUrl(environment);
    this.wsUrlValue = wsUrl(environment);
    this.programIdValue = envProgramId(environment);
    this.primaryRpcUrlValue = rpcUrl(environment);
    return this;
  }

  baseUrl(url: string): LightconeClientBuilder {
    return this.withBaseUrl(url);
  }

  withBaseUrl(url: string): LightconeClientBuilder {
    this.baseUrlValue = url;
    return this;
  }

  wsUrl(url: string): LightconeClientBuilder {
    return this.withWsUrl(url);
  }

  withWsUrl(url: string): LightconeClientBuilder {
    this.wsUrlValue = url;
    return this;
  }

  auth(credentials: AuthCredentials): LightconeClientBuilder {
    return this.withAuth(credentials);
  }

  withAuth(credentials: AuthCredentials): LightconeClientBuilder {
    this.authCredentials = credentials;
    return this;
  }

  programId(id: PublicKey): LightconeClientBuilder {
    this.programIdValue = id;
    return this;
  }

  depositSource(source: DepositSource): LightconeClientBuilder {
    this.depositSourceValue = source;
    return this;
  }

  nativeSigner(keypair: Keypair): LightconeClientBuilder {
    this.signingStrategyValue = { type: "native", keypair };
    return this;
  }

  externalSigner(signer: ExternalSigner): LightconeClientBuilder {
    this.signingStrategyValue = { type: "walletAdapter", signer };
    return this;
  }

  privyWalletId(walletId: string): LightconeClientBuilder {
    this.signingStrategyValue = { type: "privy", walletId };
    return this;
  }

  rpcUrl(url: string): LightconeClientBuilder {
    this.primaryRpcUrlValue = url;
    return this;
  }

  /** Set a backup Solana RPC URL for automatic failover. */
  backupRpcUrl(url: string): LightconeClientBuilder {
    this.backupRpcUrlValue = url;
    return this;
  }

  build(): LightconeClient {
    return new LightconeClient({
      http: new LightconeHttp(this.baseUrlValue),
      wsConfig: {
        url: this.wsUrlValue,
        reconnect: true,
        maxReconnectAttempts: 10,
        baseReconnectDelayMs: 1_000,
        pingIntervalMs: 30_000,
        pongTimeoutMs: 10_000,
      },
      programId: this.programIdValue,
      depositSource: this.depositSourceValue,
      signingStrategy: this.signingStrategyValue,
      primaryConnection: this.primaryRpcUrlValue
        ? new Connection(this.primaryRpcUrlValue, { commitment: "confirmed" })
        : undefined,
      backupConnection: this.backupRpcUrlValue
        ? new Connection(this.backupRpcUrlValue, { commitment: "confirmed" })
        : undefined,
      authCredentials: this.authCredentials,
    });
  }
}
