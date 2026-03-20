import { Connection, Keypair, PublicKey, Transaction } from "@solana/web3.js";
import { Auth, type AuthCredentials } from "./auth";
import type { ClientContext } from "./context";
import { signAndSubmitTx as signAndSubmitTxFn } from "./context";
import { Admin } from "./domain/admin";
import { Markets } from "./domain/market";
import { Notifications } from "./domain/notification";
import { Orders } from "./domain/order";
import { Orderbooks } from "./domain/orderbook";
import { Positions } from "./domain/position";
import { PriceHistoryClient } from "./domain/price_history";
import { Referrals } from "./domain/referral";
import { Trades } from "./domain/trade";
import { LightconeHttp } from "./http";
import { DEFAULT_API_URL, DEFAULT_WS_URL } from "./network";
import { Privy } from "./privy";
import { PROGRAM_ID } from "./program/constants";
import { Rpc } from "./rpc";
import { DepositSource } from "./shared";
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
  readonly connection?: Connection;
  private depositSourceValue: DepositSource;
  private signingStrategyValue?: SigningStrategy;
  private readonly wsConfigValue: WsConfig;
  private readonly authStateStore: AuthState;

  constructor(params: {
    http: LightconeHttp;
    wsConfig: WsConfig;
    programId?: PublicKey;
    connection?: Connection;
    depositSource?: DepositSource;
    signingStrategy?: SigningStrategy;
    authCredentials?: AuthCredentials;
    authState?: AuthState;
  }) {
    this.http = params.http;
    this.programId = params.programId ?? PROGRAM_ID;
    this.connection = params.connection;
    this.depositSourceValue = params.depositSource ?? DepositSource.Global;
    this.signingStrategyValue = params.signingStrategy;
    this.wsConfigValue = params.wsConfig;
    this.authStateStore =
      params.authState ??
      new AuthState(params.authCredentials);
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

  setSigningStrategy(strategy: SigningStrategy): void {
    this.signingStrategyValue = strategy;
  }

  clearSigningStrategy(): void {
    this.signingStrategyValue = undefined;
  }

  // ── Transaction signing + submission ────────────────────────────────

  async signAndSubmitTx(tx: Transaction): Promise<string> {
    return signAndSubmitTxFn(this, tx);
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

  admin(): Admin {
    return new Admin(this);
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
      connection: this.connection
        ? new Connection(this.connection.rpcEndpoint)
        : undefined,
      depositSource: this.depositSourceValue,
      signingStrategy: this.signingStrategyValue,
      authState: this.authStateStore,
    });
  }
}

export class LightconeClientBuilder {
  private baseUrlValue: string = DEFAULT_API_URL;
  private wsUrlValue: string = DEFAULT_WS_URL;
  private authCredentials?: AuthCredentials;
  private programIdValue: PublicKey = PROGRAM_ID;
  private depositSourceValue: DepositSource = DepositSource.Global;
  private signingStrategyValue?: SigningStrategy;
  private rpcUrlValue?: string;

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
    this.rpcUrlValue = url;
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
        pongTimeoutMs: 1_000,
      },
      programId: this.programIdValue,
      depositSource: this.depositSourceValue,
      signingStrategy: this.signingStrategyValue,
      connection: this.rpcUrlValue
        ? new Connection(this.rpcUrlValue)
        : undefined,
      authCredentials: this.authCredentials,
    });
  }
}
