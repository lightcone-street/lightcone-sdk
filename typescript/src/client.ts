import { Connection, PublicKey } from "@solana/web3.js";
import { Auth, type AuthCredentials } from "./auth";
import type { ClientContext, DecimalsCache } from "./context";
import { Admin } from "./domain/admin";
import { Markets } from "./domain/market";
import { Notifications } from "./domain/notification";
import { Orders } from "./domain/order";
import { Orderbooks } from "./domain/orderbook";
import type { DecimalsResponse } from "./domain/orderbook";
import { Positions } from "./domain/position";
import { PriceHistoryClient } from "./domain/price_history";
import { Referrals } from "./domain/referral";
import { Trades } from "./domain/trade";
import { LightconeHttp } from "./http";
import { DEFAULT_API_URL, DEFAULT_WS_URL } from "./network";
import { Privy } from "./privy";
import { PROGRAM_ID } from "./program/constants";
import { Rpc } from "./rpc";
import { WsClient, type WsConfig } from "./ws";

class DecimalsCacheImpl implements DecimalsCache {
  private readonly map = new Map<string, DecimalsResponse>();

  get(orderbookId: string): DecimalsResponse | undefined {
    return this.map.get(orderbookId);
  }

  set(orderbookId: string, response: DecimalsResponse): void {
    this.map.set(orderbookId, response);
  }

  clear(): void {
    this.map.clear();
  }
}

class AuthState {
  private credentialsValue: AuthCredentials | undefined;

  constructor(
    private readonly clearCachesFn: () => Promise<void>,
    initial?: AuthCredentials
  ) {
    this.credentialsValue = initial;
  }

  getCredentials(): AuthCredentials | undefined {
    return this.credentialsValue;
  }

  setCredentials(credentials: AuthCredentials | undefined): void {
    this.credentialsValue = credentials;
  }

  async clearCaches(): Promise<void> {
    await this.clearCachesFn();
  }
}

export class LightconeClient implements ClientContext {
  readonly http: LightconeHttp;
  readonly programId: PublicKey;
  readonly connection?: Connection;
  private readonly wsConfigValue: WsConfig;
  private readonly decimalsCacheStore: DecimalsCacheImpl;
  private readonly authStateStore: AuthState;

  constructor(params: {
    http: LightconeHttp;
    wsConfig: WsConfig;
    programId?: PublicKey;
    connection?: Connection;
    authCredentials?: AuthCredentials;
    decimalsCache?: DecimalsCacheImpl;
    authState?: AuthState;
  }) {
    this.http = params.http;
    this.programId = params.programId ?? PROGRAM_ID;
    this.connection = params.connection;
    this.wsConfigValue = params.wsConfig;
    this.decimalsCacheStore = params.decimalsCache ?? new DecimalsCacheImpl();
    this.authStateStore =
      params.authState ??
      new AuthState(async () => this.clearDecimalsCache(), params.authCredentials);
  }

  static builder(): LightconeClientBuilder {
    return new LightconeClientBuilder();
  }

  /** Decimals cache accessor for sub-clients. */
  get decimalsCache(): DecimalsCache {
    return this.decimalsCacheStore;
  }

  // ── Sub-client accessors ─────────────────────────────────────────────

  markets(): Markets {
    return new Markets(this);
  }

  orderbooks(): Orderbooks {
    return new Orderbooks(this, this.decimalsCacheStore);
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

  async clearDecimalsCache(): Promise<void> {
    this.decimalsCacheStore.clear();
  }

  clone(): LightconeClient {
    return new LightconeClient({
      http: this.http,
      wsConfig: { ...this.wsConfigValue },
      programId: this.programId,
      connection: this.connection
        ? new Connection(this.connection.rpcEndpoint)
        : undefined,
      decimalsCache: this.decimalsCacheStore,
      authState: this.authStateStore,
    });
  }
}

export class LightconeClientBuilder {
  private baseUrlValue: string = DEFAULT_API_URL;
  private wsUrlValue: string = DEFAULT_WS_URL;
  private authCredentials?: AuthCredentials;
  private programIdValue: PublicKey = PROGRAM_ID;
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
      connection: this.rpcUrlValue
        ? new Connection(this.rpcUrlValue)
        : undefined,
      authCredentials: this.authCredentials,
    });
  }
}
